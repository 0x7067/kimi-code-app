#!/bin/sh
set -eu

cd "$(dirname "$0")/.."

READY_PORT="${KIMI_TAURI_READY_PORT:-1420}"
DX_PORT="${KIMI_DX_PORT:-1421}"
ADDR="127.0.0.1"
DX_PID=""

cleanup() {
  if [ -n "$DX_PID" ] && kill -0 "$DX_PID" 2>/dev/null; then
    kill "$DX_PID" 2>/dev/null || true
    wait "$DX_PID" 2>/dev/null || true
  fi
}

trap cleanup INT TERM EXIT

dx serve --platform web --port "$DX_PORT" --open false &
DX_PID="$!"

attempts=0
while :; do
  if ! kill -0 "$DX_PID" 2>/dev/null; then
    wait "$DX_PID"
  fi

  html="$(curl -fsS "http://$ADDR:$DX_PORT/" 2>/dev/null || true)"
  case "$html" in
    *wasm/kimi-code-app-ui.js*)
      case "$html" in
        *"Starting the build"*) ;;
        *) break ;;
      esac
      ;;
  esac

  attempts=$((attempts + 1))
  if [ "$attempts" -gt 240 ]; then
    echo "Timed out waiting for Dioxus to serve real app HTML on port $DX_PORT" >&2
    exit 1
  fi
  sleep 0.25
done

python3 - "$READY_PORT" "$DX_PORT" <<'PY'
import http.server
import shutil
import socketserver
import sys
import urllib.error
import urllib.request

ready_port = int(sys.argv[1])
dx_port = int(sys.argv[2])
upstream = f"http://127.0.0.1:{dx_port}"


class Proxy(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        self.proxy(send_body=True)

    def do_HEAD(self):
        self.proxy(send_body=False)

    def proxy(self, send_body):
        target = upstream + self.path
        request = urllib.request.Request(target, method="GET")
        try:
            with urllib.request.urlopen(request, timeout=30) as response:
                self.send_response(response.status)
                for key, value in response.headers.items():
                    if key.lower() not in {"connection", "keep-alive", "proxy-authenticate", "proxy-authorization", "te", "trailers", "transfer-encoding", "upgrade"}:
                        self.send_header(key, value)
                self.end_headers()
                if send_body:
                    shutil.copyfileobj(response, self.wfile)
        except urllib.error.HTTPError as error:
            self.send_response(error.code)
            self.end_headers()
            if send_body:
                self.wfile.write(error.read())
        except Exception as error:
            self.send_response(502)
            self.end_headers()
            if send_body:
                self.wfile.write(f"Dioxus proxy failed: {error}".encode())

    def log_message(self, *_args):
        return


class ReusableTCPServer(socketserver.ThreadingTCPServer):
    allow_reuse_address = True


with ReusableTCPServer(("127.0.0.1", ready_port), Proxy) as server:
    print(f"Tauri dev URL ready on http://localhost:{ready_port}; proxying dx on {dx_port}", flush=True)
    server.serve_forever()
PY
