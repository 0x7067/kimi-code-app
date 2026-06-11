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

/// Resolve the project-instructions file for a project root (F-003.9).
///
/// Checks `AGENTS.md` first, then `CLAUDE.md` (commonly a symlink to
/// AGENTS.md). Symlinks are followed and the resolved target path is
/// returned, so a `CLAUDE.md -> AGENTS.md` link reports the real AGENTS.md.
pub(crate) fn resolve_agents_md(root: &std::path::Path) -> Option<std::path::PathBuf> {
    for name in ["AGENTS.md", "CLAUDE.md"] {
        let candidate = root.join(name);
        // `is_file` follows symlinks, so a CLAUDE.md link to AGENTS.md counts.
        if candidate.is_file() {
            return Some(std::fs::canonicalize(&candidate).unwrap_or(candidate));
        }
    }
    None
}

/// F-003.9 — AGENTS.md auto-detection for the session-creation dialog.
///
/// NOTE: the kimi CLI auto-injects AGENTS.md into the session context itself
/// (server-side) when a session starts in `work_dir`, so the app's job here
/// is DETECTION + PREVIEW only — we never inject the content into prompts.
#[tauri::command]
pub async fn read_agents_md(work_dir: String) -> Result<Option<Value>, String> {
    let Some(path) = resolve_agents_md(std::path::Path::new(&work_dir)) else {
        return Ok(None);
    };
    let content = tokio::fs::read_to_string(&path).await.map_err(|e| e.to_string())?;
    Ok(Some(json!({ "path": path.to_string_lossy(), "content": content })))
}

/// Convert merged mcp.json entries into the ACP `mcpServers` array format.
///
/// F-005: disabled servers and unsupported transports are dropped here —
/// kimi's mcpCapabilities advertise stdio + HTTP only (no SSE, no ACP
/// transport), so anything else must never reach `session/new`.
pub(crate) fn to_acp_servers(merged: std::collections::BTreeMap<String, Value>) -> Vec<Value> {
    let to_pairs = |v: Option<&Value>| -> Vec<Value> {
        v.and_then(|x| x.as_object())
            .map(|o| {
                o.iter()
                    .map(|(k, val)| json!({"name": k, "value": val.as_str().unwrap_or_default()}))
                    .collect()
            })
            .unwrap_or_default()
    };
    merged
        .into_iter()
        .filter_map(|(name, cfg)| {
            if !super::mcp::is_enabled(&cfg) {
                return None;
            }
            match super::mcp::transport_of(&cfg).as_str() {
                "http" => cfg.get("url").and_then(|u| u.as_str()).map(|url| {
                    json!({
                        "type": "http",
                        "name": name,
                        "url": url,
                        "headers": to_pairs(cfg.get("headers")),
                    })
                }),
                "stdio" => cfg.get("command").and_then(|c| c.as_str()).map(|command| {
                    json!({
                        "name": name,
                        "command": command,
                        "args": cfg.get("args").cloned().unwrap_or(json!([])),
                        "env": to_pairs(cfg.get("env")),
                    })
                }),
                _ => None, // sse / acp transports: not supported by kimi
            }
        })
        .collect()
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
    Ok(to_acp_servers(merged))
}

#[cfg(test)]
mod acp_servers_tests {
    use super::to_acp_servers;
    use serde_json::{json, Value};
    use std::collections::BTreeMap;

    fn merged(entries: &[(&str, Value)]) -> BTreeMap<String, Value> {
        entries.iter().map(|(k, v)| (k.to_string(), v.clone())).collect()
    }

    #[test]
    fn stdio_and_http_servers_pass_through() {
        let out = to_acp_servers(merged(&[
            ("files", json!({"command": "mcp-files", "args": ["-r"], "env": {"K": "v"}})),
            ("web", json!({"url": "https://mcp.example.com"})),
        ]));
        assert_eq!(out.len(), 2);
        assert_eq!(out[0]["command"], "mcp-files");
        assert_eq!(out[1]["type"], "http");
        assert_eq!(out[1]["url"], "https://mcp.example.com");
    }

    #[test]
    fn disabled_servers_are_dropped() {
        let out = to_acp_servers(merged(&[
            ("off", json!({"command": "x", "enabled": false})),
            ("on", json!({"command": "y", "enabled": true})),
        ]));
        assert_eq!(out.len(), 1);
        assert_eq!(out[0]["name"], "on");
    }

    #[test]
    fn sse_and_acp_transports_are_dropped() {
        let out = to_acp_servers(merged(&[
            ("sse", json!({"type": "sse", "url": "https://old.example.com"})),
            ("acp", json!({"type": "acp", "command": "agent"})),
            ("ok", json!({"type": "http", "url": "https://x.dev"})),
        ]));
        assert_eq!(out.len(), 1);
        assert_eq!(out[0]["name"], "ok");
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_agents_md;
    use std::fs;
    use std::path::PathBuf;

    fn temp_root(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("kimi-agents-md-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn resolve_agents_md_returns_none_when_absent() {
        let root = temp_root("none");
        assert_eq!(resolve_agents_md(&root), None);
    }

    #[test]
    fn resolve_agents_md_prefers_agents_md() {
        let root = temp_root("prefers");
        fs::write(root.join("AGENTS.md"), "# agents").unwrap();
        fs::write(root.join("CLAUDE.md"), "# claude").unwrap();
        let found = resolve_agents_md(&root).unwrap();
        assert!(found.ends_with("AGENTS.md"), "got {found:?}");
    }

    #[test]
    fn resolve_agents_md_falls_back_to_claude_md() {
        let root = temp_root("fallback");
        fs::write(root.join("CLAUDE.md"), "# claude").unwrap();
        let found = resolve_agents_md(&root).unwrap();
        assert!(found.ends_with("CLAUDE.md"), "got {found:?}");
    }

    #[cfg(unix)]
    #[test]
    fn resolve_agents_md_follows_claude_md_symlink_to_target() {
        let root = temp_root("symlink");
        fs::write(root.join("instructions.md"), "# real").unwrap();
        std::os::unix::fs::symlink(root.join("instructions.md"), root.join("CLAUDE.md")).unwrap();
        let found = resolve_agents_md(&root).unwrap();
        assert!(found.ends_with("instructions.md"), "got {found:?}");
    }

    #[cfg(unix)]
    #[test]
    fn resolve_agents_md_ignores_dangling_symlink() {
        let root = temp_root("dangling");
        std::os::unix::fs::symlink(root.join("missing.md"), root.join("AGENTS.md")).unwrap();
        assert_eq!(resolve_agents_md(&root), None);
    }
}
