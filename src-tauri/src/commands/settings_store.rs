//! F-011.13: lightweight app-settings store — a JSON file in the Tauri
//! app-config dir, written atomically (temp + rename). Holds GUI-side
//! preferences (binary override, thinking default, approval prefs, YOLO);
//! never tokens or secrets (NF-022).

use serde_json::Value;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|d| d.join("app_settings.json"))
        .map_err(|e| e.to_string())
}

/// Apply settings that the backend itself consumes (the kimi binary override).
fn apply_backend_side(settings: &Value) {
    let override_path = settings
        .get("kimiBinOverride")
        .and_then(|v| v.as_str())
        .map(String::from);
    crate::paths::set_kimi_override(override_path);
}

/// Read app settings; `{}` if the file does not exist yet. Also applies the
/// binary override so a plain startup load configures the backend.
#[tauri::command]
pub async fn read_app_settings(app: AppHandle) -> Result<Value, String> {
    let path = settings_path(&app)?;
    let settings = match tokio::fs::read_to_string(&path).await {
        Ok(s) => serde_json::from_str(&s).unwrap_or_else(|_| Value::Object(Default::default())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Value::Object(Default::default()),
        Err(e) => return Err(e.to_string()),
    };
    apply_backend_side(&settings);
    Ok(settings)
}

/// Persist app settings atomically (write temp file, then rename).
#[tauri::command]
pub async fn write_app_settings(app: AppHandle, settings: Value) -> Result<(), String> {
    let path = settings_path(&app)?;
    if let Some(dir) = path.parent() {
        tokio::fs::create_dir_all(dir).await.map_err(|e| e.to_string())?;
    }
    let body = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    let tmp = path.with_extension("json.tmp");
    tokio::fs::write(&tmp, body).await.map_err(|e| e.to_string())?;
    tokio::fs::rename(&tmp, &path).await.map_err(|e| e.to_string())?;
    apply_backend_side(&settings);
    Ok(())
}
