//! F-007: Memory management commands.

use serde_json::Value;
use tauri::AppHandle;

#[tauri::command]
pub async fn list_memories(app: AppHandle, cwd: String) -> Result<Value, String> {
    let snippets = crate::memory::list_memories(&app, &cwd)?;
    Ok(serde_json::to_value(&snippets).map_err(|e| e.to_string())?)
}

#[tauri::command]
pub async fn save_memory(
    app: AppHandle,
    cwd: String,
    content: String,
    source: String,
) -> Result<Value, String> {
    let snippet = crate::memory::save_memory(&app, &cwd, content, source)?;
    Ok(serde_json::to_value(&snippet).map_err(|e| e.to_string())?)
}

#[tauri::command]
pub async fn delete_memory(app: AppHandle, cwd: String, id: String) -> Result<(), String> {
    crate::memory::delete_memory(&app, &cwd, &id)
}

#[tauri::command]
pub async fn pin_memory(
    app: AppHandle,
    cwd: String,
    id: String,
    pinned: bool,
) -> Result<(), String> {
    crate::memory::pin_memory(&app, &cwd, &id, pinned)
}

#[tauri::command]
pub async fn retrieve_memories(
    app: AppHandle,
    cwd: String,
    query: String,
    top_k: usize,
) -> Result<Value, String> {
    let snippets = crate::memory::retrieve_memories(&app, &cwd, &query, top_k)?;
    Ok(serde_json::to_value(&snippets).map_err(|e| e.to_string())?)
}

#[tauri::command]
pub async fn build_memory_context(
    app: AppHandle,
    cwd: String,
    query: String,
    top_k: usize,
) -> Result<String, String> {
    crate::memory::build_memory_context(&app, &cwd, &query, top_k)
}
