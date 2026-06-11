//! ACP client lifecycle and JSON-RPC bridging commands.

use crate::acp::protocol::AgentCapabilities;
use crate::acp::queue::MessageQueue;
use crate::acp::sessions::SessionRegistry;
use crate::acp::supervisor::Supervisor;
use crate::acp::AcpClient;
use serde_json::{json, Value};
use std::sync::Arc;
use tauri::{AppHandle, State};
use tokio::sync::Mutex;

/// Outbound queue capacity while disconnected (F-001.8).
const OUTBOX_CAPACITY: usize = 256;
/// Crash auto-restart budget and base backoff (F-001.6/.10).
const MAX_RESTARTS: u32 = 5;
const BASE_BACKOFF_MS: u64 = 500;

pub struct AppState {
    pub acp: Mutex<Option<Arc<AcpClient>>>,
    pub outbox: Arc<std::sync::Mutex<MessageQueue>>,
    pub sessions: Arc<std::sync::Mutex<SessionRegistry>>,
    pub supervisor: std::sync::Mutex<Supervisor>,
    pub capabilities: std::sync::Mutex<AgentCapabilities>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            acp: Mutex::new(None),
            outbox: Arc::new(std::sync::Mutex::new(MessageQueue::new(OUTBOX_CAPACITY))),
            sessions: Arc::new(std::sync::Mutex::new(SessionRegistry::new())),
            supervisor: std::sync::Mutex::new(Supervisor::new(MAX_RESTARTS, BASE_BACKOFF_MS)),
            capabilities: std::sync::Mutex::new(AgentCapabilities::default()),
        }
    }
}

/// Connect (or reconnect) to `kimi acp` and run the initialize handshake.
#[tauri::command]
pub async fn acp_connect(app: AppHandle, state: State<'_, AppState>) -> Result<Value, Value> {
    let mut guard = state.acp.lock().await;
    if let Some(old) = guard.take() {
        old.kill().await;
    }
    if let Ok(mut sup) = state.supervisor.lock() {
        sup.reset();
    }
    let (client, init) =
        AcpClient::connect_and_init(app, state.outbox.clone(), state.sessions.clone()).await?;
    *guard = Some(client);
    Ok(init)
}

async fn client(state: &State<'_, AppState>) -> Result<Arc<AcpClient>, Value> {
    state
        .acp
        .lock()
        .await
        .clone()
        .ok_or(json!({"message": "not connected"}))
}

/// Generic JSON-RPC request to the agent (session/new, session/prompt, ...).
#[tauri::command]
pub async fn acp_request(state: State<'_, AppState>, method: String, params: Value) -> Result<Value, Value> {
    client(&state).await?.request(&method, params).await
}

/// Fire-and-forget notification (e.g. session/cancel).
#[tauri::command]
pub async fn acp_notify(state: State<'_, AppState>, method: String, params: Value) -> Result<(), Value> {
    client(&state).await?.notify(&method, params);
    Ok(())
}

/// Cancel the active turn for a session; the in-flight prompt resolves with
/// `stopReason: "cancelled"`.
#[tauri::command]
pub async fn acp_cancel(state: State<'_, AppState>, session_id: String) -> Result<(), Value> {
    client(&state).await?.cancel(&session_id);
    Ok(())
}

/// Steer the active turn (F-015): `session/cancel` → await the cancelled
/// turn's resolution (5s fallback) → immediately `session/prompt` the new
/// text. The reply is the new prompt's result.
#[tauri::command]
pub async fn acp_steer(state: State<'_, AppState>, session_id: String, text: String) -> Result<Value, Value> {
    let params = crate::acp::protocol::prompt_params(&session_id, &text);
    client(&state).await?.steer(&session_id, params).await
}

/// Resolve a pending permission request from the UI.
#[tauri::command]
pub async fn acp_respond_permission(
    state: State<'_, AppState>,
    request_id: u64,
    outcome: Value,
) -> Result<(), Value> {
    let client = client(&state).await?;
    if let Some(tx) = client.permission_waiters.lock().await.remove(&request_id) {
        let _ = tx.send(outcome);
    }
    Ok(())
}
