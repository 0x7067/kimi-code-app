//! F-009: Headless ACP execution for automations.
//!
//! Spawns a short-lived `kimi acp` process, drives a single prompt through
//! to completion, and returns the agent's text response. No Tauri events,
//! no UI — plain stdout/stdin JSON-RPC.

use serde_json::{json, Value};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::time::timeout;

const HEADLESS_TIMEOUT_SECS: u64 = 300;

pub struct HeadlessResult {
    pub text: String,
    pub tool_calls: Vec<String>,
    pub stop_reason: String,
}

/// Run a single prompt headlessly in `cwd` and return the agent response.
pub async fn run_prompt(cwd: &str, prompt: &str) -> Result<HeadlessResult, String> {
    let mut child = Command::new(crate::paths::kimi_bin())
        .arg("acp")
        .current_dir(cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("failed to spawn `kimi acp`: {e}"))?;

    let mut stdin = child.stdin.take().ok_or("no stdin")?;
    let stdout = child.stdout.take().ok_or("no stdout")?;

    // 1. Initialize
    let init_id = send_request(&mut stdin, "initialize", json!({"protocolVersion": 1})).await?;
    let mut lines = BufReader::new(stdout).lines();
    wait_for_response(&mut lines, init_id, HEADLESS_TIMEOUT_SECS).await?;

    // 2. Create session
    let new_id = send_request(&mut stdin, "session/new", session_new_params(cwd)).await?;
    let new_res = wait_for_response(&mut lines, new_id, HEADLESS_TIMEOUT_SECS).await?;
    let session_id = new_res
        .get("result")
        .and_then(|r| r.get("sessionId"))
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();
    if session_id.is_empty() {
        return Err("session/new returned no sessionId".into());
    }

    // 3. Send prompt
    let prompt_id = send_request(
        &mut stdin,
        "session/prompt",
        prompt_params(&session_id, prompt),
    )
    .await?;

    // 4. Collect streamed updates until turn completes.
    let result = collect_turn(&mut lines, prompt_id, &session_id).await;

    // 5. Tear down: close stdin and let the process exit (ACP has no
    // session/exit method; dropping stdin signals end-of-input).
    drop(stdin);
    let _ = timeout(Duration::from_secs(2), child.wait()).await;

    result
}

async fn send_request(stdin: &mut tokio::process::ChildStdin, method: &str, params: Value) -> Result<u64, String> {
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    let id = COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let msg = json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params});
    let line = serde_json::to_string(&msg).map_err(|e| e.to_string())?;
    stdin
        .write_all(line.as_bytes())
        .await
        .map_err(|e| e.to_string())?;
    stdin.write_all(b"\n").await.map_err(|e| e.to_string())?;
    stdin.flush().await.map_err(|e| e.to_string())?;
    Ok(id)
}

async fn wait_for_response(
    lines: &mut tokio::io::Lines<BufReader<tokio::process::ChildStdout>>,
    expected_id: u64,
    secs: u64,
) -> Result<Value, String> {
    let deadline = Duration::from_secs(secs);
    let start = std::time::Instant::now();
    loop {
        if start.elapsed() > deadline {
            return Err("headless response timeout".into());
        }
        let line = timeout(Duration::from_secs(5), lines.next_line())
            .await
            .map_err(|_| "read timeout")?
            .map_err(|e| e.to_string())?
            .ok_or("stdout closed")?;
        let msg: Value = serde_json::from_str(&line).map_err(|e| e.to_string())?;
        if msg.get("id").and_then(|v| v.as_u64()) == Some(expected_id) {
            return Ok(msg);
        }
        // Ignore other messages (e.g. notifications).
    }
}

async fn collect_turn(
    lines: &mut tokio::io::Lines<BufReader<tokio::process::ChildStdout>>,
    prompt_id: u64,
    session_id: &str,
) -> Result<HeadlessResult, String> {
    let deadline = Duration::from_secs(HEADLESS_TIMEOUT_SECS);
    let start = std::time::Instant::now();
    let mut text = String::new();
    let mut tool_calls = Vec::new();
    let mut stop_reason = String::new();

    loop {
        if start.elapsed() > deadline {
            break;
        }
        let line = timeout(Duration::from_secs(5), lines.next_line())
            .await
            .map_err(|_| "read timeout")?
            .map_err(|e| e.to_string())?
            .ok_or("stdout closed")?;
        let msg: Value = serde_json::from_str(&line).unwrap_or(Value::Null);

        // Check for the response to our prompt request.
        if msg.get("id").and_then(|v| v.as_u64()) == Some(prompt_id) {
            if let Some(result) = msg.get("result") {
                if let Some(reason) = result.get("stopReason").and_then(|s| s.as_str()) {
                    stop_reason = reason.to_string();
                }
                if stop_reason.is_empty() || stop_reason == "finished" {
                    // Some implementations return the full text here.
                    if let Some(t) = result.get("text").and_then(|s| s.as_str()) {
                        text.push_str(t);
                    }
                }
            }
            if !stop_reason.is_empty() {
                break;
            }
            continue;
        }

        // Handle streaming updates.
        if let Some(params) = msg.get("params") {
            let sid = params.get("sessionId").and_then(|s| s.as_str()).unwrap_or("");
            if sid != session_id {
                continue;
            }
            let update = params.get("update").unwrap_or(params);
            let kind = update
                .get("sessionUpdate")
                .and_then(|s| s.as_str())
                .unwrap_or("");
            match kind {
                "agent_message_chunk" => {
                    if let Some(t) = content_text(update.get("content")) {
                        text.push_str(&t);
                    }
                }
                "tool_call" => {
                    if let Some(title) = update.get("title").and_then(|s| s.as_str()) {
                        tool_calls.push(title.to_string());
                    }
                }
                _ => {}
            }
            // Also check nested stopReason in update payload.
            if let Some(reason) = update.get("stopReason").and_then(|s| s.as_str()) {
                stop_reason = reason.to_string();
                if !stop_reason.is_empty() && stop_reason != "in_progress" {
                    break;
                }
            }
        }
    }

    Ok(HeadlessResult {
        text,
        tool_calls,
        stop_reason: if stop_reason.is_empty() {
            "unknown".into()
        } else {
            stop_reason
        },
    })
}

fn content_text(v: Option<&Value>) -> Option<String> {
    let v = v?;
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Array(blocks) => Some(
            blocks
                .iter()
                .filter_map(|b| content_text(Some(b)))
                .collect::<Vec<_>>()
                .join(""),
        ),
        Value::Object(_) => v
            .get("text")
            .and_then(|t| t.as_str())
            .map(String::from)
            .or_else(|| content_text(v.get("content"))),
        _ => None,
    }
}

fn session_new_params(cwd: &str) -> Value {
    json!({ "cwd": cwd, "mcpServers": [] })
}

fn prompt_params(session_id: &str, prompt: &str) -> Value {
    json!({
        "sessionId": session_id,
        "prompt": [{ "type": "text", "text": prompt }],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_new_params_include_empty_mcp_servers() {
        let params = session_new_params("/tmp/project");

        assert_eq!(params["cwd"], "/tmp/project");
        assert_eq!(params["mcpServers"].as_array().map(Vec::len), Some(0));
    }

    #[test]
    fn prompt_params_use_acp_content_blocks() {
        let params = prompt_params("session_1", "Reply with exactly OK.");

        assert_eq!(params["sessionId"], "session_1");
        assert!(params.get("text").is_none(), "headless prompt must not use the old text field");
        assert_eq!(params["prompt"][0]["type"], "text");
        assert_eq!(params["prompt"][0]["text"], "Reply with exactly OK.");
    }
}
