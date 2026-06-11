//! Project discovery: recent projects and MCP server config merging.

use crate::paths::kimi_home;
use serde_json::{json, Value};

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
