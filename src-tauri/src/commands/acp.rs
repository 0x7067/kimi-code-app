//! ACP client lifecycle and JSON-RPC bridging commands.

use crate::acp::AcpClient;
use serde_json::{json, Value};
use std::sync::Arc;
use tauri::{AppHandle, State};
use tokio::sync::Mutex;

#[derive(Default)]
pub struct AppState {
    pub acp: Mutex<Option<Arc<AcpClient>>>,
}

/// Connect (or reconnect) to `kimi acp` and run the initialize handshake.
#[tauri::command]
pub async fn acp_connect(app: AppHandle, state: State<'_, AppState>) -> Result<Value, Value> {
    let mut guard = state.acp.lock().await;
    if let Some(old) = guard.take() {
        old.kill().await;
    }
    let client = AcpClient::spawn(app)
        .await
        .map_err(|e| json!({"message": e}))?;
    let init = client
        .request(
            "initialize",
            json!({
                "protocolVersion": 1,
                "clientCapabilities": { "fs": { "readTextFile": true, "writeTextFile": true } },
                "clientInfo": { "name": "Kimi Code App", "version": env!("CARGO_PKG_VERSION") }
            }),
        )
        .await?;
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
pub async fn acp_request(
    state: State<'_, AppState>,
    method: String,
    params: Value,
) -> Result<Value, Value> {
    client(&state).await?.request(&method, params).await
}

/// Fire-and-forget notification (e.g. session/cancel).
#[tauri::command]
pub async fn acp_notify(
    state: State<'_, AppState>,
    method: String,
    params: Value,
) -> Result<(), Value> {
    client(&state).await?.notify(&method, params);
    Ok(())
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
