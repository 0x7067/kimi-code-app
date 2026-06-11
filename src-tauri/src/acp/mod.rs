//! JSON-RPC 2.0 client over stdio for `kimi acp`.

pub mod protocol;
pub mod queue;
pub mod sessions;
pub mod store;
pub mod supervisor;

use queue::{MessageQueue, TurnQueue};
use serde_json::{json, Value};
use sessions::SessionRegistry;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
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
    /// Outbound notifications queued while disconnected; flushed on reconnect.
    outbox: Arc<std::sync::Mutex<MessageQueue>>,
    /// Per-session update history shared across restarts.
    sessions: Arc<std::sync::Mutex<SessionRegistry>>,
    /// Set by `kill()` so an intentional shutdown doesn't trigger auto-restart.
    shutting_down: AtomicBool,
    /// Per-session prompt serialization: kimi rejects a second `session/prompt`
    /// while a turn is active, so later prompts wait their turn here.
    turns: std::sync::Mutex<TurnQueue<oneshot::Sender<()>>>,
}

impl AcpClient {
    /// Spawn `kimi acp` and connect to the shared outbox / session registry
    /// (both owned by `AppState` so they survive process restarts).
    pub async fn spawn(
        app: AppHandle,
        outbox: Arc<std::sync::Mutex<MessageQueue>>,
        sessions: Arc<std::sync::Mutex<SessionRegistry>>,
    ) -> Result<Arc<Self>, String> {
        let mut child = Command::new(crate::paths::kimi_bin())
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
        // Flush messages queued while we were disconnected (F-001.8).
        if let Ok(mut q) = outbox.lock() {
            for line in q.drain() {
                let _ = stdin_tx.send(line);
            }
        }
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
            outbox,
            sessions,
            shutting_down: AtomicBool::new(false),
            turns: std::sync::Mutex::new(TurnQueue::new()),
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
            // stdout EOF without kill() means the subprocess crashed (F-001.6/.10).
            if !reader_client.shutting_down.load(Ordering::SeqCst) {
                let _ = reader_app.emit("acp:crashed", ());
                AcpClient::auto_restart(reader_app, reader_client).await;
            }
        });

        Ok(client)
    }

    /// Spawn the subprocess and run the `initialize` handshake, flagging
    /// protocol version mismatches to the frontend (F-001.7).
    pub async fn connect_and_init(
        app: AppHandle,
        outbox: Arc<std::sync::Mutex<MessageQueue>>,
        sessions: Arc<std::sync::Mutex<SessionRegistry>>,
    ) -> Result<(Arc<Self>, Value), Value> {
        let client = Self::spawn(app.clone(), outbox, sessions)
            .await
            .map_err(|e| json!({"message": e}))?;
        let init = client
            .request(
                "initialize",
                json!({
                    "protocolVersion": protocol::PROTOCOL_VERSION,
                    "clientCapabilities": { "fs": { "readTextFile": true, "writeTextFile": true } },
                    "clientInfo": { "name": "Kimi Code App", "version": env!("CARGO_PKG_VERSION") }
                }),
            )
            .await?;
        match protocol::negotiate_version(&init) {
            protocol::VersionOutcome::Match(_) => {}
            protocol::VersionOutcome::Mismatch { ours, theirs } => {
                let _ = app.emit("acp:version_mismatch", json!({"ours": ours, "theirs": theirs}));
            }
            protocol::VersionOutcome::Missing => {
                let _ = app.emit(
                    "acp:version_mismatch",
                    json!({"ours": protocol::PROTOCOL_VERSION, "theirs": Value::Null}),
                );
            }
        }
        // Remember agent capabilities (loadSession, session list/resume) for
        // session sync features.
        let caps = protocol::parse_agent_capabilities(&init);
        if let Ok(mut slot) = app.state::<crate::commands::AppState>().capabilities.lock() {
            *slot = caps;
        }
        Ok((client, init))
    }

    /// Restart the crashed subprocess with exponential backoff, then hand the
    /// frontend the session ids to replay (F-001.10).
    ///
    /// Returns a boxed future to break the async type cycle
    /// (spawn -> reader task -> auto_restart -> spawn).
    fn auto_restart(
        app: AppHandle,
        dead: Arc<Self>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> {
        Box::pin(async move {
            let state = app.state::<crate::commands::AppState>();
            loop {
                let backoff = state
                    .supervisor
                    .lock()
                    .map(|mut s| s.next_backoff())
                    .ok()
                    .flatten();
                let Some(ms) = backoff else {
                    let _ = app.emit("acp:restart_failed", ());
                    return;
                };
                tokio::time::sleep(Duration::from_millis(ms)).await;
                // Bail if a manual reconnect already replaced us.
                {
                    let guard = state.acp.lock().await;
                    match guard.as_ref() {
                        Some(current) if Arc::ptr_eq(current, &dead) => {}
                        _ => return,
                    }
                }
                match Self::connect_and_init(app.clone(), dead.outbox.clone(), dead.sessions.clone()).await {
                    Ok((client, _)) => {
                        *state.acp.lock().await = Some(client);
                        if let Ok(mut s) = state.supervisor.lock() {
                            s.reset();
                        }
                        let replay = dead.sessions.lock().map(|s| s.session_ids()).unwrap_or_default();
                        let _ = app.emit("acp:restarted", json!({"replaySessions": replay}));
                        return;
                    }
                    Err(_) => continue,
                }
            }
        })
    }

    async fn handle_message(self: &Arc<Self>, app: &AppHandle, msg: Value) {
        match protocol::route(&msg) {
            // Response to one of our requests
            protocol::Incoming::Response { id, result } => {
                if let Some(tx) = self.pending.lock().await.remove(&id) {
                    let _ = tx.send(result);
                }
            }
            // Streamed session update: record per-session, then forward (F-001.4/.9)
            protocol::Incoming::SessionUpdate { params, .. } => {
                if let Ok(mut reg) = self.sessions.lock() {
                    reg.record_update(&params);
                }
                let _ = app.emit("acp:update", params);
            }
            protocol::Incoming::Notification { method, params } => {
                if method == "session/update" {
                    // Malformed update without a sessionId: still forward as before.
                    let _ = app.emit("acp:update", params);
                } else {
                    let _ = app.emit("acp:notification", json!({"method": method, "params": params}));
                }
            }
            protocol::Incoming::Invalid => {}
            // Agent->client request expecting a reply
            protocol::Incoming::Request { id, method, params } => {
                self.handle_request(app, id, &method, params).await;
            }
        }
    }

    async fn handle_request(self: &Arc<Self>, app: &AppHandle, req_id: Value, method: &str, params: Value) {
        match method {
            "session/request_permission" => {
                let id = req_id;
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
                let id = req_id;
                let path = params.get("path").and_then(|p| p.as_str()).unwrap_or("");
                match tokio::fs::read_to_string(path).await {
                    Ok(mut content) => {
                        if let Some(line) = params.get("line").and_then(|v| v.as_u64()) {
                            let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(u64::MAX);
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
                let id = req_id;
                let path = params.get("path").and_then(|p| p.as_str()).unwrap_or("");
                let content = params.get("content").and_then(|c| c.as_str()).unwrap_or("");
                match tokio::fs::write(path, content).await {
                    Ok(_) => self.reply(id, Value::Null),
                    Err(e) => self.reply_err(id, -32603, &e.to_string()),
                }
            }
            _ => {
                // Unknown agent->client request: answer methodNotFound so the agent isn't stuck.
                let _ = app;
                self.reply_err(req_id, -32601, "method not found");
            }
        }
    }

    fn reply(&self, id: Value, result: Value) {
        let _ = self
            .stdin_tx
            .send(json!({"jsonrpc": "2.0", "id": id, "result": result}).to_string());
    }

    fn reply_err(&self, id: Value, code: i64, message: &str) {
        let _ = self.stdin_tx.send(
            json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}}).to_string(),
        );
    }

    pub async fn request(&self, method: &str, params: Value) -> Result<Value, Value> {
        // Only one prompt may be in flight per session; queue later ones.
        if method == "session/prompt" {
            if let Some(sid) = params.get("sessionId").and_then(|s| s.as_str()) {
                let sid = sid.to_string();
                let waiter = {
                    let mut turns = self.turns.lock().expect("turn queue poisoned");
                    if turns.try_begin(&sid) {
                        None
                    } else {
                        let (tx, rx) = oneshot::channel();
                        turns.enqueue_waiter(&sid, tx);
                        Some(rx)
                    }
                };
                if let Some(rx) = waiter {
                    let _ = rx.await;
                }
                let res = self.send_request(method, params).await;
                if let Ok(mut turns) = self.turns.lock() {
                    if let Some(next) = turns.end_turn(&sid) {
                        let _ = next.send(());
                    }
                }
                return res;
            }
        }
        self.send_request(method, params).await
    }

    /// Cancel the active turn for a session. `session/cancel` is a
    /// notification; the in-flight `session/prompt` then resolves normally
    /// with `stopReason: "cancelled"` (see [`protocol::stop_reason`]).
    pub fn cancel(&self, session_id: &str) {
        self.notify("session/cancel", json!({ "sessionId": session_id }));
    }

    async fn send_request(&self, method: &str, params: Value) -> Result<Value, Value> {
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
        let payload = json!({"jsonrpc": "2.0", "method": method, "params": params}).to_string();
        if self.stdin_tx.send(payload.clone()).is_err() {
            // Disconnected: queue for flush on reconnect (F-001.8).
            if let Ok(mut q) = self.outbox.lock() {
                q.push(payload);
            }
        }
    }

    pub async fn kill(&self) {
        self.shutting_down.store(true, Ordering::SeqCst);
        if let Some(mut child) = self.child.lock().await.take() {
            let _ = child.kill().await;
        }
    }
}
