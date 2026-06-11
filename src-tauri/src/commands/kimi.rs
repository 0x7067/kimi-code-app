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

/// Spawn a kimi subcommand, streaming combined stdout/stderr lines to the UI on
/// `{event}:line` and emitting `{event}:done` with the exit code. Used by the
/// long-running CLI flows (`upgrade`, `migrate`) that print progress.
async fn stream_kimi(app: AppHandle, args: Vec<String>, event: &'static str) -> Result<i32, String> {
    let mut child = tokio::process::Command::new(kimi_bin())
        .args(&args)
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
            let _ = app_out.emit(&format!("{event}:line"), line);
        }
    });
    let app_err = app.clone();
    tauri::async_runtime::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = app_err.emit(&format!("{event}:line"), line);
        }
    });

    let status = child.wait().await.map_err(|e| e.to_string())?;
    let code = status.code().unwrap_or(-1);
    let _ = app.emit(&format!("{event}:done"), code);
    Ok(code)
}

/// Run a kimi subcommand to completion and return its captured output.
async fn run_kimi(args: &[&str]) -> Result<Value, String> {
    let out = tokio::process::Command::new(kimi_bin())
        .args(args)
        .output()
        .await
        .map_err(|e| format!("failed to run kimi: {e}"))?;
    Ok(json!({
        "code": out.status.code().unwrap_or(-1),
        "stdout": String::from_utf8_lossy(&out.stdout).to_string(),
        "stderr": String::from_utf8_lossy(&out.stderr).to_string(),
    }))
}

/// Export a session as a ZIP archive via `kimi export`. `session_id` defaults to
/// the most recent session when empty; `output` is the destination ZIP path.
/// `-y` skips the interactive previous-session confirmation (non-interactive).
#[tauri::command]
pub async fn kimi_export_session(
    session_id: Option<String>,
    output: String,
) -> Result<Value, String> {
    let mut args: Vec<String> = vec!["export".into()];
    if let Some(id) = session_id.filter(|s| !s.trim().is_empty()) {
        args.push(id);
    }
    args.push("-o".into());
    args.push(output.clone());
    args.push("-y".into());
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let mut result = run_kimi(&refs).await?;
    // `kimi export` exits 0 even when the write fails (e.g. the interactive
    // "previous session" fallback under non-interactive stdin), so a zero code
    // is not proof of success. Confirm the archive actually exists and downgrade
    // the reported code when it does not, surfacing the CLI's stderr to the UI.
    if !std::path::Path::new(&output).exists() {
        if let Some(obj) = result.as_object_mut() {
            obj.insert("code".into(), json!(1));
            let stderr = obj.get("stderr").and_then(Value::as_str).unwrap_or("").trim();
            let msg = if stderr.is_empty() {
                "export produced no file (try specifying an explicit session id)".to_string()
            } else {
                stderr.to_string()
            };
            obj.insert("stderr".into(), json!(msg));
        }
    }
    Ok(result)
}

/// List configured LLM providers via `kimi provider list --json`.
#[tauri::command]
pub async fn kimi_provider_list() -> Result<Value, String> {
    let out = run_kimi(&["provider", "list", "--json"]).await?;
    // Parse the machine-readable payload when the command succeeded; fall back
    // to the raw text envelope so the UI can surface CLI errors verbatim.
    if out.get("code").and_then(Value::as_i64) == Some(0) {
        let stdout = out.get("stdout").and_then(Value::as_str).unwrap_or("");
        if let Ok(parsed) = serde_json::from_str::<Value>(stdout) {
            return Ok(json!({ "code": 0, "providers": parsed }));
        }
    }
    Ok(out)
}

/// Import providers from a custom registry URL via `kimi provider add <url>`.
#[tauri::command]
pub async fn kimi_provider_add(url: String) -> Result<Value, String> {
    run_kimi(&["provider", "add", &url]).await
}

/// Remove a provider and its model aliases via `kimi provider remove <id>`.
#[tauri::command]
pub async fn kimi_provider_remove(provider_id: String) -> Result<Value, String> {
    run_kimi(&["provider", "remove", &provider_id]).await
}

/// Discover and import providers from the public models.dev catalog via
/// `kimi provider catalog`.
#[tauri::command]
pub async fn kimi_provider_catalog() -> Result<Value, String> {
    run_kimi(&["provider", "catalog"]).await
}

/// Validate a kimi config file via `kimi doctor config|tui`. `which` is either
/// `"config"` (config.toml) or `"tui"` (tui.toml).
#[tauri::command]
pub async fn kimi_doctor(which: String) -> Result<Value, String> {
    let sub = match which.as_str() {
        "config" => "config",
        "tui" => "tui",
        other => return Err(format!("unknown doctor target: {other}")),
    };
    run_kimi(&["doctor", sub]).await
}

/// Upgrade Kimi Code to the latest version via `kimi upgrade`, streaming
/// progress lines to the UI on `upgrade:line` / `upgrade:done`.
#[tauri::command]
pub async fn kimi_upgrade(app: AppHandle) -> Result<i32, String> {
    stream_kimi(app, vec!["upgrade".into()], "upgrade").await
}

/// Migrate legacy kimi-cli data into kimi-code via `kimi migrate`, streaming
/// progress lines to the UI on `migrate:line` / `migrate:done`.
#[tauri::command]
pub async fn kimi_migrate(app: AppHandle) -> Result<i32, String> {
    stream_kimi(app, vec!["migrate".into()], "migrate").await
}
