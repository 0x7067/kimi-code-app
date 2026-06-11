//! CLI↔app session sync commands (F-012). Sessions live in kimi's own store
//! (`~/.kimi-code/`), shared by the CLI and this app, so listing/loading here
//! is automatically in sync with `kimi` runs from a terminal.

use super::acp::AppState;
use crate::acp::{protocol, store};
use crate::paths::kimi_home;
use serde_json::{json, Value};
use std::time::Duration;
use tauri::{AppHandle, Emitter, State};

/// List sessions for a project. Prefers the agent's `session/list` extension
/// when advertised; falls back to parsing `session_index.jsonl` directly
/// (also the path when no agent is connected). Sorted by `updatedAt` desc.
#[tauri::command]
pub async fn kimi_list_sessions(
    state: State<'_, AppState>,
    work_dir: String,
) -> Result<Vec<Value>, Value> {
    let session_list = state.capabilities.lock().map(|c| c.session_list).unwrap_or(false);
    if session_list {
        if let Some(client) = state.acp.lock().await.clone() {
            if let Ok(result) = client.request("session/list", json!({ "cwd": work_dir })).await {
                if let Some(sessions) = store::sessions_from_list_result(&result) {
                    return Ok(sessions);
                }
            }
            // Unrecognized shape or agent error: fall through to the store.
        }
    }
    let home = kimi_home();
    let summaries = tokio::task::spawn_blocking(move || store::list_sessions(&home, &work_dir))
        .await
        .map_err(|e| json!({"message": e.to_string()}))?;
    summaries
        .into_iter()
        .map(|s| serde_json::to_value(s).map_err(|e| json!({"message": e.to_string()})))
        .collect()
}

/// Load an existing session via ACP `session/load`. The agent replays the
/// stored history as `session/update` notifications, which stream to the
/// frontend through the existing `acp:update` event path before this resolves.
#[tauri::command]
pub async fn kimi_load_session(
    state: State<'_, AppState>,
    session_id: String,
    cwd: String,
    mcp_servers: Option<Value>,
) -> Result<Value, Value> {
    let client = state
        .acp
        .lock()
        .await
        .clone()
        .ok_or(json!({"message": "not connected"}))?;
    if let Ok(mut reg) = state.sessions.lock() {
        reg.ensure(&session_id);
    }
    client
        .request(
            "session/load",
            json!({
                "sessionId": session_id,
                "cwd": cwd,
                "mcpServers": mcp_servers.unwrap_or_else(|| json!([])),
            }),
        )
        .await
        .map_err(|e| {
            if protocol::is_turn_busy(&e) {
                json!({
                    "code": "TURN_AGENT_BUSY",
                    "message": "This session has a turn in progress; stop it before loading.",
                })
            } else {
                e
            }
        })
}

/// Poll `session_index.jsonl` for changes (mtime+size, every 2s) and emit
/// `sessions:changed` so the UI re-lists. Spawned once from app setup; the
/// CLI appends to the index whenever a session is created or touched.
pub fn spawn_session_index_watcher(app: AppHandle) {
    let path = kimi_home().join("session_index.jsonl");
    tauri::async_runtime::spawn(async move {
        let stamp = |m: &std::fs::Metadata| (m.modified().ok(), m.len());
        let mut last = std::fs::metadata(&path).ok().as_ref().map(stamp);
        loop {
            tokio::time::sleep(Duration::from_secs(2)).await;
            let current = std::fs::metadata(&path).ok().as_ref().map(stamp);
            if current != last {
                last = current;
                let _ = app.emit("sessions:changed", ());
            }
        }
    });
}
