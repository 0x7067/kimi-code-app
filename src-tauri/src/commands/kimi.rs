//! Commands wrapping the kimi CLI (login, version) and webview diagnostics.

use crate::paths::{kimi_bin, kimi_candidates, kimi_home};
use serde_json::{json, Value};
use std::path::PathBuf;
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

/// F-011.1: detect the kimi binary, checking the configured override, PATH,
/// and the known install locations in order. Returns the first existing
/// candidate plus its `--version` output.
#[tauri::command]
pub async fn detect_kimi_binary(override_path: Option<String>) -> Result<Value, String> {
    let override_path = override_path.filter(|p| !p.trim().is_empty()).map(PathBuf::from);
    let home = dirs::home_dir().unwrap_or_default();
    let path_var = std::env::var("PATH").ok();
    let candidates = kimi_candidates(
        override_path.as_deref(),
        path_var.as_deref(),
        &kimi_home(),
        &home,
    );
    let Some(found) = candidates.into_iter().find(|c| c.is_file()) else {
        return Ok(json!({"found": false}));
    };
    let version = tokio::process::Command::new(&found)
        .arg("--version")
        .output()
        .await
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    let is_override = override_path.as_deref() == Some(found.as_path());
    Ok(json!({
        "found": true,
        "path": found.to_string_lossy(),
        "version": version,
        "source": if is_override { "override" } else { "detected" },
    }))
}

/// F-011.2: read-only login-state check. The kimi CLI has no `auth status`
/// subcommand, so this checks whether its own credential store exists (the
/// token itself is never read — NF-022 keeps auth inside the CLI).
#[tauri::command]
pub async fn kimi_auth_status() -> Result<Value, String> {
    let cred = kimi_home().join("credentials/kimi-code.json");
    let logged_in = tokio::fs::metadata(&cred).await.is_ok();
    Ok(json!({"loggedIn": logged_in}))
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
