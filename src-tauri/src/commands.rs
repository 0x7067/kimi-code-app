use crate::acp::AcpClient;
use serde_json::{json, Value};
use std::process::Stdio;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::Mutex;

#[derive(Default)]
pub struct AppState {
    pub acp: Mutex<Option<Arc<AcpClient>>>,
}

fn kimi_home() -> std::path::PathBuf {
    std::env::var("KIMI_CODE_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| dirs::home_dir().unwrap_or_default().join(".kimi-code"))
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

/// Generic JSON-RPC request to the agent (session/new, session/prompt, ...).
#[tauri::command]
pub async fn acp_request(
    state: State<'_, AppState>,
    method: String,
    params: Value,
) -> Result<Value, Value> {
    let client = state
        .acp
        .lock()
        .await
        .clone()
        .ok_or(json!({"message": "not connected"}))?;
    client.request(&method, params).await
}

/// Fire-and-forget notification (e.g. session/cancel).
#[tauri::command]
pub async fn acp_notify(
    state: State<'_, AppState>,
    method: String,
    params: Value,
) -> Result<(), Value> {
    let client = state
        .acp
        .lock()
        .await
        .clone()
        .ok_or(json!({"message": "not connected"}))?;
    client.notify(&method, params);
    Ok(())
}

/// Resolve a pending permission request from the UI.
#[tauri::command]
pub async fn acp_respond_permission(
    state: State<'_, AppState>,
    request_id: u64,
    outcome: Value,
) -> Result<(), Value> {
    let client = state
        .acp
        .lock()
        .await
        .clone()
        .ok_or(json!({"message": "not connected"}))?;
    if let Some(tx) = client.permission_waiters.lock().await.remove(&request_id) {
        let _ = tx.send(outcome);
    }
    Ok(())
}

/// Run `kimi login` (device-code OAuth), streaming output lines to the UI.
#[tauri::command]
pub async fn kimi_login(app: AppHandle) -> Result<i32, String> {
    let mut child = tokio::process::Command::new("kimi")
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
    let out = tokio::process::Command::new("kimi")
        .arg("--version")
        .output()
        .await
        .map_err(|e| format!("kimi CLI not found: {e}"))?;
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Read a kimi config file (config.toml, tui.toml, mcp.json, AGENTS.md) from ~/.kimi-code.
#[tauri::command]
pub async fn read_kimi_config(name: String) -> Result<String, String> {
    let allowed = ["config.toml", "tui.toml", "mcp.json", "AGENTS.md"];
    if !allowed.contains(&name.as_str()) {
        return Err("not an allowed config file".into());
    }
    let path = kimi_home().join(&name);
    match tokio::fs::read_to_string(&path).await {
        Ok(s) => Ok(s),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn write_kimi_config(name: String, content: String) -> Result<(), String> {
    let allowed = ["config.toml", "tui.toml", "mcp.json", "AGENTS.md"];
    if !allowed.contains(&name.as_str()) {
        return Err("not an allowed config file".into());
    }
    let home = kimi_home();
    tokio::fs::create_dir_all(&home).await.map_err(|e| e.to_string())?;
    tokio::fs::write(home.join(&name), content)
        .await
        .map_err(|e| e.to_string())
}

/// List recently used project directories derived from the global session index.
#[tauri::command]
pub async fn recent_projects() -> Result<Vec<Value>, String> {
    let path = kimi_home().join("session_index.jsonl");
    let content = tokio::fs::read_to_string(&path).await.unwrap_or_default();
    let mut seen = std::collections::HashSet::new();
    let mut projects = Vec::new();
    for line in content.lines().rev() {
        let Ok(v) = serde_json::from_str::<Value>(line) else { continue };
        let Some(cwd) = v
            .get("cwd")
            .or_else(|| v.get("workDir"))
            .or_else(|| v.get("work_dir"))
            .and_then(|c| c.as_str())
        else {
            continue;
        };
        if seen.insert(cwd.to_string()) {
            projects.push(json!({ "path": cwd, "exists": std::path::Path::new(cwd).is_dir() }));
        }
        if projects.len() >= 30 {
            break;
        }
    }
    Ok(projects)
}

/// Merge user-level and project-level mcp.json into the ACP `mcpServers` array format.
#[tauri::command]
pub async fn mcp_servers(cwd: String) -> Result<Vec<Value>, String> {
    let mut merged: std::collections::BTreeMap<String, Value> = Default::default();
    for path in [
        kimi_home().join("mcp.json"),
        std::path::Path::new(&cwd).join(".kimi-code/mcp.json"),
    ] {
        let Ok(content) = tokio::fs::read_to_string(&path).await else { continue };
        let Ok(v) = serde_json::from_str::<Value>(&content) else { continue };
        if let Some(servers) = v.get("mcpServers").and_then(|s| s.as_object()) {
            for (name, cfg) in servers {
                merged.insert(name.clone(), cfg.clone());
            }
        }
    }
    let to_pairs = |v: Option<&Value>| -> Vec<Value> {
        v.and_then(|x| x.as_object())
            .map(|o| {
                o.iter()
                    .map(|(k, val)| json!({"name": k, "value": val.as_str().unwrap_or_default()}))
                    .collect()
            })
            .unwrap_or_default()
    };
    Ok(merged
        .into_iter()
        .filter_map(|(name, cfg)| {
            if let Some(url) = cfg.get("url").and_then(|u| u.as_str()) {
                Some(json!({
                    "type": "http",
                    "name": name,
                    "url": url,
                    "headers": to_pairs(cfg.get("headers")),
                }))
            } else {
                cfg.get("command").and_then(|c| c.as_str()).map(|command| {
                    json!({
                        "name": name,
                        "command": command,
                        "args": cfg.get("args").cloned().unwrap_or(json!([])),
                        "env": to_pairs(cfg.get("env")),
                    })
                })
            }
        })
        .collect())
}

/// Pick an image file and return it base64-encoded for an ACP image prompt block.
#[tauri::command]
pub async fn pick_image(app: AppHandle) -> Result<Option<Value>, String> {
    use tauri_plugin_dialog::DialogExt;
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

/// Native folder picker.
#[tauri::command]
pub async fn pick_folder(app: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog().file().pick_folder(move |folder| {
        let _ = tx.send(folder.map(|f| f.to_string()));
    });
    rx.await.map_err(|e| e.to_string())
}

/// Git diff of working tree for the review pane.
#[tauri::command]
pub async fn git_diff(cwd: String) -> Result<Value, String> {
    let status = tokio::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&cwd)
        .output()
        .await
        .map_err(|e| e.to_string())?;
    if !status.status.success() {
        return Ok(json!({ "isRepo": false, "files": [], "diff": "" }));
    }
    let diff = tokio::process::Command::new("git")
        .args(["diff", "HEAD"])
        .current_dir(&cwd)
        .output()
        .await
        .map_err(|e| e.to_string())?;
    Ok(json!({
        "isRepo": true,
        "files": String::from_utf8_lossy(&status.stdout).lines().map(|l| l.to_string()).collect::<Vec<_>>(),
        "diff": String::from_utf8_lossy(&diff.stdout),
    }))
}
