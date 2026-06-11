//! Read/write access to the kimi config files in ~/.kimi-code.

use crate::paths::kimi_home;

const ALLOWED_CONFIGS: &[&str] = &["config.toml", "tui.toml", "mcp.json", "AGENTS.md"];

/// Read a kimi config file (config.toml, tui.toml, mcp.json, AGENTS.md) from ~/.kimi-code.
#[tauri::command]
pub async fn read_kimi_config(name: String) -> Result<String, String> {
    if !ALLOWED_CONFIGS.contains(&name.as_str()) {
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
    if !ALLOWED_CONFIGS.contains(&name.as_str()) {
        return Err("not an allowed config file".into());
    }
    let home = kimi_home();
    tokio::fs::create_dir_all(&home).await.map_err(|e| e.to_string())?;
    tokio::fs::write(home.join(&name), content)
        .await
        .map_err(|e| e.to_string())
}
