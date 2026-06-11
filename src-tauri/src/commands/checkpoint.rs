//! Tauri commands for session checkpoints (F-002.6).

use crate::acp::store::session_dir_for;
use crate::checkpoint;
use crate::paths::kimi_home;
use serde_json::{json, Value};

fn resolve_session_dir(session_id: &str) -> Result<String, String> {
    let home = kimi_home();
    let content = std::fs::read_to_string(home.join("session_index.jsonl"))
        .map_err(|e| e.to_string())?;
    session_dir_for(&content, session_id).ok_or_else(|| "session not found".into())
}

#[tauri::command]
pub fn save_checkpoint(
    session_id: String,
    name: String,
    items: Value,
    plan: Value,
) -> Result<Value, String> {
    let dir = resolve_session_dir(&session_id)?;
    let saved_name = checkpoint::save_checkpoint(&dir, &name, items, plan)?;
    Ok(json!({"name": saved_name}))
}

#[tauri::command]
pub fn list_checkpoints(session_id: String) -> Result<Vec<Value>, String> {
    let dir = resolve_session_dir(&session_id)?;
    checkpoint::list_checkpoints(&dir)
}

#[tauri::command]
pub fn load_checkpoint(session_id: String, name: String) -> Result<Value, String> {
    let dir = resolve_session_dir(&session_id)?;
    let (items, plan) = checkpoint::load_checkpoint(&dir, &name)?;
    Ok(json!({"items": items, "plan": plan}))
}

#[tauri::command]
pub fn delete_checkpoint(session_id: String, name: String) -> Result<(), String> {
    let dir = resolve_session_dir(&session_id)?;
    checkpoint::delete_checkpoint(&dir, &name)
}