//! Commands wrapping the kimi CLI (login, version) and webview diagnostics.

use crate::paths::kimi_bin;
use std::process::Stdio;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};

/// Run `kimi login` (device-code OAuth), streaming output lines to the UI.
#[tauri::command]
pub async fn kimi_login(app: AppHandle) -> Result<i32, String> {
    let mut child = tokio::process::Command::new(kimi_bin())
        .arg("login")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    let stdout = child.stdout.take().ok_or("no stdout")?;
    let stderr = child.stderr.take().ok_or("no stderr")?;

    let app_out = app.clone();
    tauri::async_runtime::spawn(async move {
        let mut lines = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = app_out.emit("login:line", line);
        }
    });
    let app_err = app.clone();
    tauri::async_runtime::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = app_err.emit("login:line", line);
        }
    });

    let status = child.wait().await.map_err(|e| e.to_string())?;
    let code = status.code().unwrap_or(-1);
    let _ = app.emit("login:done", code);
    Ok(code)
}

/// Forward frontend console errors to stderr (diagnostics).
#[tauri::command]
pub fn js_log(msg: String) {
    eprintln!("[webview] {msg}");
}

/// Check kimi availability and version.
#[tauri::command]
pub async fn kimi_version() -> Result<String, String> {
    let out = tokio::process::Command::new(kimi_bin())
        .arg("--version")
        .output()
        .await
        .map_err(|e| format!("kimi CLI not found: {e}"))?;
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}
