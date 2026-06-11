//! JSON-RPC 2.0 client over stdio for `kimi acp`.

use serde_json::{json, Value};
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{mpsc, oneshot, Mutex};

pub struct AcpClient {
    next_id: AtomicU64,
    pending: Arc<Mutex<HashMap<u64, oneshot::Sender<Result<Value, Value>>>>>,
    /// Pending agent->client permission requests keyed by our synthetic id.
    pub permission_waiters: Arc<Mutex<HashMap<u64, oneshot::Sender<Value>>>>,
    stdin_tx: mpsc::UnboundedSender<String>,
    child: Mutex<Option<Child>>,
}

impl AcpClient {
    pub async fn spawn(app: AppHandle) -> Result<Arc<Self>, String> {
        let mut child = Command::new(crate::commands::kimi_bin())
            .arg("acp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| format!("failed to spawn `kimi acp`: {e}"))?;

        let mut stdin = child.stdin.take().ok_or("no stdin")?;
        let stdout = child.stdout.take().ok_or("no stdout")?;

        let (stdin_tx, mut stdin_rx) = mpsc::unbounded_channel::<String>();
        tauri::async_runtime::spawn(async move {
            while let Some(line) = stdin_rx.recv().await {
                if stdin.write_all(line.as_bytes()).await.is_err() {
                    break;
                }
                let _ = stdin.write_all(b"\n").await;
                let _ = stdin.flush().await;
            }
        });

        let client = Arc::new(AcpClient {
            next_id: AtomicU64::new(1),
            pending: Arc::new(Mutex::new(HashMap::new())),
            permission_waiters: Arc::new(Mutex::new(HashMap::new())),
            stdin_tx,
            child: Mutex::new(Some(child)),
        });

        let reader_client = client.clone();
        let reader_app = app.clone();
        tauri::async_runtime::spawn(async move {
            let mut lines = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let Ok(msg) = serde_json::from_str::<Value>(&line) else {
                    continue;
                };
                reader_client.handle_message(&reader_app, msg).await;
            }
            let _ = reader_app.emit("acp:disconnected", ());
        });

        Ok(client)
    }

    async fn handle_message(self: &Arc<Self>, app: &AppHandle, msg: Value) {
        // Response to one of our requests
        if let Some(id) = msg.get("id").and_then(|v| v.as_u64()) {
            if msg.get("method").is_none() {
                if let Some(tx) = self.pending.lock().await.remove(&id) {
                    let res = if let Some(err) = msg.get("error") {
                        Err(err.clone())
                    } else {
                        Ok(msg.get("result").cloned().unwrap_or(Value::Null))
                    };
                    let _ = tx.send(res);
                }
                return;
            }
        }

        let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let params = msg.get("params").cloned().unwrap_or(Value::Null);
        let req_id = msg.get("id").cloned();

        match method {
            "session/update" => {
                let _ = app.emit("acp:update", params);
            }
            "session/request_permission" => {
                let Some(id) = req_id else { return };
                let key = id.as_u64().unwrap_or(u64::MAX);
                let (tx, rx) = oneshot::channel();
                self.permission_waiters.lock().await.insert(key, tx);
                let _ = app.emit(
                    "acp:permission_request",
                    json!({ "requestId": key, "params": params }),
                );
                let me = self.clone();
                tauri::async_runtime::spawn(async move {
                    let outcome = rx.await.unwrap_or(json!({"outcome": "cancelled"}));
                    me.reply(id, json!({ "outcome": outcome }));
                });
            }
            "fs/read_text_file" => {
                let Some(id) = req_id else { return };
                let path = params.get("path").and_then(|p| p.as_str()).unwrap_or("");
                match tokio::fs::read_to_string(path).await {
                    Ok(mut content) => {
                        if let Some(line) = params.get("line").and_then(|v| v.as_u64()) {
                            let limit = params
                                .get("limit")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(u64::MAX);
                            content = content
                                .lines()
                                .skip((line.saturating_sub(1)) as usize)
                                .take(limit as usize)
                                .collect::<Vec<_>>()
                                .join("\n");
                        }
                        self.reply(id, json!({ "content": content }));
                    }
                    Err(e) => self.reply_err(id, -32603, &e.to_string()),
                }
            }
            "fs/write_text_file" => {
                let Some(id) = req_id else { return };
                let path = params.get("path").and_then(|p| p.as_str()).unwrap_or("");
                let content = params.get("content").and_then(|c| c.as_str()).unwrap_or("");
                match tokio::fs::write(path, content).await {
                    Ok(_) => self.reply(id, Value::Null),
                    Err(e) => self.reply_err(id, -32603, &e.to_string()),
                }
            }
            _ => {
                // Unknown agent->client request: answer methodNotFound so the agent isn't stuck.
                if let Some(id) = req_id {
                    self.reply_err(id, -32601, "method not found");
                } else if !method.is_empty() {
                    let _ = app.emit("acp:notification", json!({"method": method, "params": params}));
                }
            }
        }
    }

    fn reply(&self, id: Value, result: Value) {
        let _ = self.stdin_tx.send(
            json!({"jsonrpc": "2.0", "id": id, "result": result}).to_string(),
        );
    }

    fn reply_err(&self, id: Value, code: i64, message: &str) {
        let _ = self.stdin_tx.send(
            json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
                .to_string(),
        );
    }

    pub async fn request(&self, method: &str, params: Value) -> Result<Value, Value> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = oneshot::channel();
        self.pending.lock().await.insert(id, tx);
        let payload = json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params});
        if self.stdin_tx.send(payload.to_string()).is_err() {
            self.pending.lock().await.remove(&id);
            return Err(json!({"code": -32000, "message": "agent process is not running"}));
        }
        rx.await
            .unwrap_or_else(|_| Err(json!({"code": -32000, "message": "agent disconnected"})))
    }

    pub fn notify(&self, method: &str, params: Value) {
        let _ = self.stdin_tx.send(
            json!({"jsonrpc": "2.0", "method": method, "params": params}).to_string(),
        );
    }

    pub async fn kill(&self) {
        if let Some(mut child) = self.child.lock().await.take() {
            let _ = child.kill().await;
        }
    }
}
