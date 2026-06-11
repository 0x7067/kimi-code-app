//! F-005: structured MCP server management over ~/.kimi-code/mcp.json.
//!
//! Pure parse/serialize functions operate on the raw JSON text and preserve
//! unknown fields (both top-level keys next to `mcpServers` and extra keys
//! inside each server object) by editing `serde_json::Value` in place.
//!
//! Kimi's ACP capabilities advertise `mcpCapabilities {http: true, sse: false}`
//! — only stdio and HTTP transports are usable; SSE/ACP-transport servers are
//! reported with an "unsupported" status and never forwarded to the agent.

use super::config::{read_kimi_config, write_kimi_config};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// A structured view of one `mcpServers` entry.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct McpServer {
    pub name: String,
    /// "stdio" | "http" | anything else (sse/acp → unsupported).
    pub transport: String,
    pub command: String,
    pub args: Vec<String>,
    pub url: String,
    pub env: BTreeMap<String, String>,
    pub enabled: bool,
}

impl Default for McpServer {
    fn default() -> Self {
        Self {
            name: String::new(),
            transport: "stdio".into(),
            command: String::new(),
            args: Vec::new(),
            url: String::new(),
            env: BTreeMap::new(),
            enabled: true,
        }
    }
}

/// Classify a raw server object's transport: an explicit `type` field wins,
/// otherwise `url` implies http and `command` implies stdio.
pub fn transport_of(cfg: &Value) -> String {
    match cfg.get("type").and_then(|t| t.as_str()) {
        Some("streamable-http") | Some("streamable_http") => "http".into(),
        Some(t) => t.to_ascii_lowercase(),
        None if cfg.get("url").is_some() => "http".into(),
        None => "stdio".into(),
    }
}

/// Whether a server entry is enabled (absent flag = enabled).
pub fn is_enabled(cfg: &Value) -> bool {
    cfg.get("enabled").and_then(|e| e.as_bool()) != Some(false)
}

fn root_object(raw: &str) -> Result<Map<String, Value>, String> {
    if raw.trim().is_empty() {
        return Ok(Map::new());
    }
    match serde_json::from_str::<Value>(raw) {
        Ok(Value::Object(o)) => Ok(o),
        Ok(_) => Err("mcp.json is not a JSON object".into()),
        Err(e) => Err(format!("mcp.json is not valid JSON: {e}")),
    }
}

/// Parse mcp.json text into structured entries (sorted by name).
pub fn parse_mcp_json(raw: &str) -> Result<Vec<McpServer>, String> {
    let root = root_object(raw)?;
    let mut out: Vec<McpServer> = root
        .get("mcpServers")
        .and_then(|s| s.as_object())
        .map(|servers| {
            servers
                .iter()
                .map(|(name, cfg)| McpServer {
                    name: name.clone(),
                    transport: transport_of(cfg),
                    command: cfg.get("command").and_then(|c| c.as_str()).unwrap_or_default().into(),
                    args: cfg
                        .get("args")
                        .and_then(|a| a.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default(),
                    url: cfg.get("url").and_then(|u| u.as_str()).unwrap_or_default().into(),
                    env: cfg
                        .get("env")
                        .and_then(|e| e.as_object())
                        .map(|o| {
                            o.iter()
                                .map(|(k, v)| (k.clone(), v.as_str().unwrap_or_default().to_string()))
                                .collect()
                        })
                        .unwrap_or_default(),
                    enabled: is_enabled(cfg),
                })
                .collect()
        })
        .unwrap_or_default();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

fn to_json_text(root: Map<String, Value>) -> Result<String, String> {
    serde_json::to_string_pretty(&Value::Object(root))
        .map(|mut s| {
            s.push('\n');
            s
        })
        .map_err(|e| e.to_string())
}

/// Insert or update a server, preserving unknown fields on the entry and all
/// other top-level keys. For unsupported transports (sse/acp) only the
/// `enabled` flag is touched so foreign config is never destroyed.
pub fn upsert_mcp_json(raw: &str, server: &McpServer) -> Result<String, String> {
    if server.name.trim().is_empty() {
        return Err("server name is empty".into());
    }
    let mut root = root_object(raw)?;
    let servers = root
        .entry("mcpServers".to_string())
        .or_insert_with(|| json!({}));
    let Some(servers) = servers.as_object_mut() else {
        return Err("mcpServers is not a JSON object".into());
    };
    let entry = servers
        .entry(server.name.clone())
        .or_insert_with(|| json!({}));
    let Some(obj) = entry.as_object_mut() else {
        return Err(format!("server \"{}\" is not a JSON object", server.name));
    };

    match server.transport.as_str() {
        "stdio" => {
            obj.remove("type"); // stdio is the implicit default
            obj.remove("url");
            obj.remove("headers");
            obj.insert("command".into(), json!(server.command));
            if server.args.is_empty() {
                obj.remove("args");
            } else {
                obj.insert("args".into(), json!(server.args));
            }
            if server.env.is_empty() {
                obj.remove("env");
            } else {
                obj.insert("env".into(), json!(server.env));
            }
        }
        "http" => {
            obj.insert("type".into(), json!("http"));
            obj.remove("command");
            obj.remove("args");
            obj.remove("env");
            obj.insert("url".into(), json!(server.url));
        }
        _ => {} // unsupported transport: only the enabled flag below
    }
    if server.enabled {
        obj.remove("enabled");
    } else {
        obj.insert("enabled".into(), json!(false));
    }
    to_json_text(root)
}

/// Remove a server by name, preserving everything else.
pub fn remove_mcp_json(raw: &str, name: &str) -> Result<String, String> {
    let mut root = root_object(raw)?;
    if let Some(servers) = root.get_mut("mcpServers").and_then(|s| s.as_object_mut()) {
        servers.remove(name);
    }
    to_json_text(root)
}

/// Minimal well-formedness check for an MCP HTTP endpoint.
pub fn is_valid_http_url(url: &str) -> bool {
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"));
    match rest {
        Some(host) => {
            let host_part = host.split(['/', '?', '#']).next().unwrap_or("");
            !host_part.is_empty() && !host.chars().any(char::is_whitespace)
        }
        None => false,
    }
}

/// Filesystem candidates for resolving a stdio command: an explicit path is
/// itself; a bare name is looked up in each PATH directory. Pure for tests.
pub fn command_candidates(cmd: &str, path_var: Option<&str>) -> Vec<PathBuf> {
    if cmd.is_empty() {
        return Vec::new();
    }
    if cmd.contains('/') {
        return vec![PathBuf::from(cmd)];
    }
    path_var
        .map(|p| {
            std::env::split_paths(p)
                .filter(|d| !d.as_os_str().is_empty())
                .map(|d| d.join(cmd))
                .collect()
        })
        .unwrap_or_default()
}

/// Per-server validation status: "ok" | "missing-command" | "bad-url" |
/// "unsupported". `command_found` is the filesystem check result for stdio.
pub fn server_status(server: &McpServer, command_found: bool) -> &'static str {
    match server.transport.as_str() {
        "stdio" => {
            if command_found {
                "ok"
            } else {
                "missing-command"
            }
        }
        "http" => {
            if is_valid_http_url(&server.url) {
                "ok"
            } else {
                "bad-url"
            }
        }
        _ => "unsupported",
    }
}

fn command_exists(cmd: &str) -> bool {
    let path_var = std::env::var("PATH").ok();
    command_candidates(cmd, path_var.as_deref())
        .iter()
        .any(|c| c.is_file())
}

/// List user-level MCP servers with validation status (F-005.1/.2).
#[tauri::command]
pub async fn list_mcp_servers() -> Result<Value, String> {
    let raw = read_kimi_config("mcp.json".into()).await?;
    let servers = parse_mcp_json(&raw)?;
    let out: Vec<Value> = servers
        .iter()
        .map(|s| {
            let found = s.transport == "stdio" && command_exists(&s.command);
            let mut v = serde_json::to_value(s).unwrap_or_default();
            if let Some(o) = v.as_object_mut() {
                o.insert("status".into(), json!(server_status(s, found)));
            }
            v
        })
        .collect();
    Ok(json!({ "servers": out }))
}

/// Add or update an MCP server in ~/.kimi-code/mcp.json.
#[tauri::command]
pub async fn save_mcp_server(server: McpServer) -> Result<(), String> {
    let raw = read_kimi_config("mcp.json".into()).await?;
    let updated = upsert_mcp_json(&raw, &server)?;
    write_kimi_config("mcp.json".into(), updated).await
}

/// Remove an MCP server from ~/.kimi-code/mcp.json.
#[tauri::command]
pub async fn delete_mcp_server(name: String) -> Result<(), String> {
    let raw = read_kimi_config("mcp.json".into()).await?;
    let updated = remove_mcp_json(&raw, &name)?;
    write_kimi_config("mcp.json".into(), updated).await
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
      "version": 2,
      "mcpServers": {
        "files": {"command": "mcp-files", "args": ["--root", "/tmp"], "env": {"K": "v"}, "extra": true},
        "search": {"type": "http", "url": "https://mcp.example.com/v1"},
        "legacy": {"type": "sse", "url": "https://old.example.com"},
        "off": {"command": "off-bin", "enabled": false}
      }
    }"#;

    #[test]
    fn parses_entries_with_transport_detection_and_enabled_flag() {
        let servers = parse_mcp_json(SAMPLE).expect("parses");
        let names: Vec<&str> = servers.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, vec!["files", "legacy", "off", "search"]);
        let files = &servers[0];
        assert_eq!(files.transport, "stdio");
        assert_eq!(files.command, "mcp-files");
        assert_eq!(files.args, vec!["--root", "/tmp"]);
        assert_eq!(files.env.get("K").map(String::as_str), Some("v"));
        assert!(files.enabled);
        assert_eq!(servers[1].transport, "sse");
        assert!(!servers[2].enabled);
        assert_eq!(servers[3].transport, "http");
        assert_eq!(servers[3].url, "https://mcp.example.com/v1");
    }

    #[test]
    fn parses_empty_and_rejects_invalid_json() {
        assert_eq!(parse_mcp_json("").expect("empty ok"), Vec::new());
        assert!(parse_mcp_json("{nope").is_err());
        assert!(parse_mcp_json("[1,2]").is_err());
    }

    #[test]
    fn upsert_adds_server_and_preserves_unknown_fields() {
        let server = McpServer {
            name: "new".into(),
            transport: "http".into(),
            url: "https://x.dev/mcp".into(),
            ..Default::default()
        };
        let out = upsert_mcp_json(SAMPLE, &server).expect("upserts");
        let v: Value = serde_json::from_str(&out).expect("valid json");
        assert_eq!(v["version"], 2); // top-level unknown key preserved
        assert_eq!(v["mcpServers"]["files"]["extra"], true); // sibling preserved
        assert_eq!(v["mcpServers"]["new"]["type"], "http");
        assert_eq!(v["mcpServers"]["new"]["url"], "https://x.dev/mcp");
    }

    #[test]
    fn upsert_updates_in_place_and_switches_transport() {
        let server = McpServer {
            name: "files".into(),
            transport: "http".into(),
            url: "https://files.example.com".into(),
            ..Default::default()
        };
        let out = upsert_mcp_json(SAMPLE, &server).expect("upserts");
        let v: Value = serde_json::from_str(&out).expect("valid json");
        let files = &v["mcpServers"]["files"];
        assert_eq!(files["url"], "https://files.example.com");
        assert!(files.get("command").is_none());
        assert!(files.get("args").is_none());
        assert_eq!(files["extra"], true); // unknown entry key survives the edit
    }

    #[test]
    fn upsert_disable_only_touches_enabled_for_unsupported_transport() {
        let server = McpServer {
            name: "legacy".into(),
            transport: "sse".into(),
            enabled: false,
            ..Default::default()
        };
        let out = upsert_mcp_json(SAMPLE, &server).expect("upserts");
        let v: Value = serde_json::from_str(&out).expect("valid json");
        let legacy = &v["mcpServers"]["legacy"];
        assert_eq!(legacy["enabled"], false);
        assert_eq!(legacy["url"], "https://old.example.com"); // untouched
        assert_eq!(legacy["type"], "sse");
    }

    #[test]
    fn upsert_enable_removes_the_flag_and_blank_name_is_rejected() {
        let server = McpServer { name: "off".into(), command: "off-bin".into(), ..Default::default() };
        let out = upsert_mcp_json(SAMPLE, &server).expect("upserts");
        let v: Value = serde_json::from_str(&out).expect("valid json");
        assert!(v["mcpServers"]["off"].get("enabled").is_none());
        assert!(upsert_mcp_json("", &McpServer::default()).is_err());
    }

    #[test]
    fn remove_deletes_only_the_named_server() {
        let out = remove_mcp_json(SAMPLE, "files").expect("removes");
        let v: Value = serde_json::from_str(&out).expect("valid json");
        assert!(v["mcpServers"].get("files").is_none());
        assert!(v["mcpServers"].get("search").is_some());
        assert_eq!(v["version"], 2);
    }

    #[test]
    fn http_url_validation() {
        assert!(is_valid_http_url("https://mcp.example.com/v1"));
        assert!(is_valid_http_url("http://localhost:3845/mcp"));
        assert!(!is_valid_http_url("ftp://x.com"));
        assert!(!is_valid_http_url("https://"));
        assert!(!is_valid_http_url("https://bad host/x"));
        assert!(!is_valid_http_url("mcp.example.com"));
    }

    #[test]
    fn command_candidates_path_lookup_vs_explicit_path() {
        assert_eq!(command_candidates("/usr/bin/x", Some("/a:/b")), vec![PathBuf::from("/usr/bin/x")]);
        assert_eq!(
            command_candidates("npx", Some("/a:/b")),
            vec![PathBuf::from("/a/npx"), PathBuf::from("/b/npx")]
        );
        assert!(command_candidates("", Some("/a")).is_empty());
        assert!(command_candidates("npx", None).is_empty());
    }

    #[test]
    fn server_status_covers_all_transports() {
        let stdio = McpServer { transport: "stdio".into(), command: "x".into(), ..Default::default() };
        assert_eq!(server_status(&stdio, true), "ok");
        assert_eq!(server_status(&stdio, false), "missing-command");
        let http = McpServer { transport: "http".into(), url: "https://x.dev".into(), ..Default::default() };
        assert_eq!(server_status(&http, false), "ok");
        let bad = McpServer { transport: "http".into(), url: "nope".into(), ..Default::default() };
        assert_eq!(server_status(&bad, false), "bad-url");
        let sse = McpServer { transport: "sse".into(), ..Default::default() };
        assert_eq!(server_status(&sse, true), "unsupported");
    }
}
