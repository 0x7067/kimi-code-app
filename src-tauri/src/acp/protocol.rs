//! Pure ACP protocol types and routing (F-001.4, F-001.7). No I/O here so
//! everything is unit-testable without spawning a subprocess.

use serde_json::Value;

/// Current ACP protocol revision this client targets. ACP versions are
/// integers on the wire (`kimi acp` v0.14.0 reports `1`); the date string in
/// REQUIREMENTS.md ("2025-03-26") is the MCP spec date, not the ACP version.
pub const PROTOCOL_VERSION: u64 = 1;

/// High-level classification of a `session/update` payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateKind {
    Text,
    ToolCall,
    ToolResult,
    Error,
    Status,
    Other,
}

/// Map an ACP `sessionUpdate` discriminator onto a routing kind.
pub fn classify_update(session_update: &str) -> UpdateKind {
    match session_update {
        "agent_message_chunk" | "user_message_chunk" | "agent_thought_chunk" => UpdateKind::Text,
        "tool_call" => UpdateKind::ToolCall,
        "tool_call_update" => UpdateKind::ToolResult,
        "error" => UpdateKind::Error,
        "plan" | "status" | "current_mode_update" | "available_commands_update" => UpdateKind::Status,
        _ => UpdateKind::Other,
    }
}

/// A fully routed inbound JSON-RPC message from the agent.
#[derive(Debug, Clone, PartialEq)]
pub enum Incoming {
    /// Response to one of our requests.
    Response { id: u64, result: Result<Value, Value> },
    /// `session/update` notification, classified by kind.
    SessionUpdate {
        session_id: String,
        kind: UpdateKind,
        params: Value,
    },
    /// Agent->client request (permission, fs, ...) that expects a reply.
    Request {
        id: Value,
        method: String,
        params: Value,
    },
    /// Any other notification.
    Notification { method: String, params: Value },
    /// Not a structurally valid JSON-RPC message.
    Invalid,
}

/// Route a raw JSON-RPC message into an [`Incoming`] variant.
pub fn route(msg: &Value) -> Incoming {
    let Some(obj) = msg.as_object() else {
        return Incoming::Invalid;
    };
    let id = obj.get("id");
    let method = obj.get("method").and_then(|m| m.as_str());

    match (method, id) {
        // Response to one of our requests.
        (None, Some(id)) => {
            let Some(id) = id.as_u64() else {
                return Incoming::Invalid;
            };
            if let Some(err) = obj.get("error") {
                Incoming::Response {
                    id,
                    result: Err(err.clone()),
                }
            } else if let Some(result) = obj.get("result") {
                Incoming::Response {
                    id,
                    result: Ok(result.clone()),
                }
            } else {
                Incoming::Invalid
            }
        }
        // Agent->client request expecting a reply.
        (Some(method), Some(id)) => Incoming::Request {
            id: id.clone(),
            method: method.to_string(),
            params: obj.get("params").cloned().unwrap_or(Value::Null),
        },
        // Notification.
        (Some(method), None) => {
            let params = obj.get("params").cloned().unwrap_or(Value::Null);
            if method == "session/update" {
                if let Some(session_id) = params.get("sessionId").and_then(|s| s.as_str()) {
                    let kind = params
                        .get("update")
                        .and_then(|u| u.get("sessionUpdate"))
                        .and_then(|k| k.as_str())
                        .map(classify_update)
                        .unwrap_or(UpdateKind::Other);
                    return Incoming::SessionUpdate {
                        session_id: session_id.to_string(),
                        kind,
                        params,
                    };
                }
            }
            Incoming::Notification {
                method: method.to_string(),
                params,
            }
        }
        (None, None) => Incoming::Invalid,
    }
}

/// Result of protocol version negotiation on the `initialize` response.
#[derive(Debug, Clone, PartialEq)]
pub enum VersionOutcome {
    Match(u64),
    Mismatch { ours: u64, theirs: u64 },
    Missing,
}

/// Inspect an `initialize` result and flag protocol version mismatches.
pub fn negotiate_version(init_result: &Value) -> VersionOutcome {
    match init_result.get("protocolVersion").and_then(|v| v.as_u64()) {
        None => VersionOutcome::Missing,
        Some(theirs) if theirs == PROTOCOL_VERSION => VersionOutcome::Match(theirs),
        Some(theirs) => VersionOutcome::Mismatch {
            ours: PROTOCOL_VERSION,
            theirs,
        },
    }
}

/// Agent capabilities advertised in the `initialize` response.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AgentCapabilities {
    pub load_session: bool,
    pub session_list: bool,
    pub session_resume: bool,
}

/// Extract the capabilities we care about from an `initialize` result.
pub fn parse_agent_capabilities(init_result: &Value) -> AgentCapabilities {
    let caps = &init_result["agentCapabilities"];
    AgentCapabilities {
        load_session: caps["loadSession"].as_bool().unwrap_or(false),
        session_list: caps["sessionCapabilities"]["list"].is_object(),
        session_resume: caps["sessionCapabilities"]["resume"].is_object(),
    }
}

/// Whether a JSON-RPC error means the agent rejected the call because a turn
/// is in flight (kimi reports `TURN_AGENT_BUSY` / "another turn is active";
/// `session/load` is rejected mid-turn the same way).
pub fn is_turn_busy(error: &Value) -> bool {
    let mut haystack = String::new();
    for v in [error.get("message"), error.get("code"), error.get("data")] {
        if let Some(v) = v {
            haystack.push_str(&v.to_string());
        }
    }
    haystack.contains("TURN_AGENT_BUSY") || haystack.contains("another turn is active")
}

/// Stop reason from a `session/prompt` result. Cancellation resolves the
/// in-flight prompt with `stopReason: "cancelled"` rather than a JSON-RPC
/// error, so callers must inspect this.
pub fn stop_reason(prompt_result: &Value) -> Option<&str> {
    prompt_result.get("stopReason").and_then(|s| s.as_str())
}

/// Build `session/prompt` params for a plain-text message (used by steering,
/// F-015, where the frontend hands us raw text rather than content blocks).
pub fn prompt_params(session_id: &str, text: &str) -> Value {
    serde_json::json!({
        "sessionId": session_id,
        "prompt": [{ "type": "text", "text": text }],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn update(kind: &str) -> Value {
        json!({
            "jsonrpc": "2.0",
            "method": "session/update",
            "params": { "sessionId": "s1", "update": { "sessionUpdate": kind } }
        })
    }

    #[test]
    fn routes_successful_response() {
        let msg = json!({"jsonrpc": "2.0", "id": 7, "result": {"ok": true}});
        assert_eq!(
            route(&msg),
            Incoming::Response {
                id: 7,
                result: Ok(json!({"ok": true}))
            }
        );
    }

    #[test]
    fn routes_error_response() {
        let msg = json!({"jsonrpc": "2.0", "id": 3, "error": {"code": -1, "message": "bad"}});
        assert_eq!(
            route(&msg),
            Incoming::Response {
                id: 3,
                result: Err(json!({"code": -1, "message": "bad"}))
            }
        );
    }

    #[test]
    fn routes_text_update() {
        match route(&update("agent_message_chunk")) {
            Incoming::SessionUpdate { session_id, kind, .. } => {
                assert_eq!(session_id, "s1");
                assert_eq!(kind, UpdateKind::Text);
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn classifies_all_required_kinds() {
        assert_eq!(classify_update("agent_message_chunk"), UpdateKind::Text);
        assert_eq!(classify_update("agent_thought_chunk"), UpdateKind::Text);
        assert_eq!(classify_update("tool_call"), UpdateKind::ToolCall);
        assert_eq!(classify_update("tool_call_update"), UpdateKind::ToolResult);
        assert_eq!(classify_update("error"), UpdateKind::Error);
        assert_eq!(classify_update("plan"), UpdateKind::Status);
        assert_eq!(classify_update("current_mode_update"), UpdateKind::Status);
        assert_eq!(classify_update("something_new"), UpdateKind::Other);
    }

    #[test]
    fn routes_permission_request() {
        let msg = json!({
            "jsonrpc": "2.0", "id": 42,
            "method": "session/request_permission",
            "params": {"sessionId": "s1"}
        });
        assert_eq!(
            route(&msg),
            Incoming::Request {
                id: json!(42),
                method: "session/request_permission".into(),
                params: json!({"sessionId": "s1"}),
            }
        );
    }

    #[test]
    fn routes_plain_notification() {
        let msg = json!({"jsonrpc": "2.0", "method": "agent/heartbeat", "params": {}});
        assert_eq!(
            route(&msg),
            Incoming::Notification {
                method: "agent/heartbeat".into(),
                params: json!({})
            }
        );
    }

    #[test]
    fn rejects_invalid_message() {
        assert_eq!(route(&json!("nonsense")), Incoming::Invalid);
        assert_eq!(route(&json!({"jsonrpc": "2.0"})), Incoming::Invalid);
    }

    /// Real initialize response shape from `kimi acp` v0.14.0.
    fn kimi_init() -> Value {
        json!({
            "protocolVersion": 1,
            "agentCapabilities": {
                "loadSession": true,
                "promptCapabilities": {"image": true, "audio": false, "embeddedContext": true},
                "mcpCapabilities": {"http": true, "sse": false},
                "sessionCapabilities": {"list": {}, "resume": {}}
            },
            "authMethods": [{"id": "login", "type": "terminal"}],
            "agentInfo": {"name": "Kimi Code CLI", "version": "0.14.0"}
        })
    }

    #[test]
    fn negotiates_matching_integer_version() {
        assert_eq!(negotiate_version(&kimi_init()), VersionOutcome::Match(1));
    }

    #[test]
    fn flags_version_mismatch() {
        assert_eq!(
            negotiate_version(&json!({"protocolVersion": 2})),
            VersionOutcome::Mismatch {
                ours: PROTOCOL_VERSION,
                theirs: 2
            }
        );
        // non-integer versions are not valid ACP versions
        assert_eq!(
            negotiate_version(&json!({"protocolVersion": "2025-03-26"})),
            VersionOutcome::Missing
        );
    }

    #[test]
    fn flags_missing_version() {
        assert_eq!(negotiate_version(&json!({})), VersionOutcome::Missing);
    }

    #[test]
    fn parses_agent_capabilities() {
        assert_eq!(
            parse_agent_capabilities(&kimi_init()),
            AgentCapabilities {
                load_session: true,
                session_list: true,
                session_resume: true
            }
        );
        assert_eq!(parse_agent_capabilities(&json!({})), AgentCapabilities::default());
    }

    #[test]
    fn detects_turn_busy_errors() {
        assert!(is_turn_busy(&json!({"code": -32603, "message": "TURN_AGENT_BUSY"})));
        assert!(is_turn_busy(&json!({"message": "another turn is active"})));
        assert!(is_turn_busy(&json!({"data": {"reason": "TURN_AGENT_BUSY"}})));
        assert!(!is_turn_busy(&json!({"code": -32601, "message": "method not found"})));
        assert!(!is_turn_busy(&json!({})));
    }

    #[test]
    fn extracts_stop_reason() {
        assert_eq!(
            stop_reason(&json!({"stopReason": "cancelled"})),
            Some("cancelled")
        );
        assert_eq!(stop_reason(&json!({"stopReason": "end_turn"})), Some("end_turn"));
        assert_eq!(stop_reason(&json!({})), None);
    }

    #[test]
    fn builds_text_prompt_params() {
        let p = prompt_params("s1", "do the thing");
        assert_eq!(p["sessionId"], "s1");
        assert_eq!(p["prompt"][0]["type"], "text");
        assert_eq!(p["prompt"][0]["text"], "do the thing");
        assert_eq!(p["prompt"].as_array().map(Vec::len), Some(1));
    }
}
