//! Native file/folder picker commands.

use serde_json::{json, Value};
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

/// Pick an image file and return it base64-encoded for an ACP image prompt block.
#[tauri::command]
pub async fn pick_image(app: AppHandle) -> Result<Option<Value>, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .add_filter("Images", &["png", "jpg", "jpeg", "gif", "webp"])
        .pick_file(move |file| {
            let _ = tx.send(file.map(|f| f.to_string()));
        });
    let Some(path) = rx.await.map_err(|e| e.to_string())? else {
        return Ok(None);
    };
    let bytes = tokio::fs::read(&path).await.map_err(|e| e.to_string())?;
    let mime = match path.rsplit('.').next().unwrap_or("").to_ascii_lowercase().as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        _ => "image/png",
    };
    use base64::Engine;
    let data = base64::engine::general_purpose::STANDARD.encode(&bytes);
    let name = path.rsplit('/').next().unwrap_or("image").to_string();
    Ok(Some(json!({"data": data, "mimeType": mime, "name": name})))
}

/// Generic native file picker (F-011.1 Browse for the kimi binary).
#[tauri::command]
pub async fn pick_file(app: AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog().file().pick_file(move |file| {
        let _ = tx.send(file.map(|f| f.to_string()));
    });
    rx.await.map_err(|e| e.to_string())
}

/// Native folder picker.
#[tauri::command]
pub async fn pick_folder(app: AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog().file().pick_folder(move |folder| {
        let _ = tx.send(folder.map(|f| f.to_string()));
    });
    rx.await.map_err(|e| e.to_string())
}
