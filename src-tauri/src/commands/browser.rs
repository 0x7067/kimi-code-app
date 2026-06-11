//! F-006: Browser preview commands.

use serde_json::Value;
use tauri::{AppHandle, Manager, State};

#[tauri::command]
pub async fn start_browser_watcher(
    app: AppHandle,
    state: State<'_, crate::commands::AppState>,
    cwd: String,
) -> Result<(), String> {
    // Drop any existing watcher first.
    {
        let mut guard = state.browser_watcher.lock().map_err(|e| e.to_string())?;
        *guard = None;
    }
    let watcher = crate::browser::start_live_reload_watcher(app, cwd)?;
    let mut guard = state.browser_watcher.lock().map_err(|e| e.to_string())?;
    *guard = Some(watcher);
    Ok(())
}

#[tauri::command]
pub async fn stop_browser_watcher(
    state: State<'_, crate::commands::AppState>,
) -> Result<(), String> {
    let mut guard = state.browser_watcher.lock().map_err(|e| e.to_string())?;
    *guard = None;
    Ok(())
}
