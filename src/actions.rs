//! Async actions: everything that talks to the backend and mutates state.

use crate::ipc::invoke;
use crate::state::*;
use dioxus::prelude::ReadableExt;
use serde_json::{json, Value};

pub async fn connect() {
    match invoke("acp_connect", json!({})).await {
        Ok(init) => {
            *CONNECTED.write() = true;
            let name = init.pointer("/agentInfo/name").and_then(|v| v.as_str()).unwrap_or("Kimi Code");
            let ver = init.pointer("/agentInfo/version").and_then(|v| v.as_str()).unwrap_or("");
            *AGENT_INFO.write() = format!("{name} {ver}");
        }
        Err(e) => *ERROR.write() = Some(err_msg(&e)),
    }
}

pub async fn refresh_sessions() {
    if let Ok(res) = invoke("acp_request", json!({"method": "session/list", "params": {}})).await {
        let sessions = res
            .get("sessions")
            .and_then(|x| x.as_array())
            .map(|arr| {
                arr.iter()
                    .map(|v| SessionMeta {
                        id: v.get("sessionId").and_then(|x| x.as_str()).unwrap_or("").into(),
                        cwd: v.get("cwd").and_then(|x| x.as_str()).unwrap_or("").into(),
                        title: v
                            .get("title")
                            .and_then(|x| x.as_str())
                            .unwrap_or("Untitled session")
                            .into(),
                        updated_at: v.get("updatedAt").and_then(|x| x.as_str()).unwrap_or("").into(),
                    })
                    .collect()
            })
            .unwrap_or_default();
        *SESSIONS.write() = sessions;
    }
}

pub async fn refresh_projects() {
    if let Ok(res) = invoke("recent_projects", json!({})).await {
        let projects = res
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter(|p| p.get("exists").and_then(|e| e.as_bool()).unwrap_or(false))
                    .filter_map(|p| p.get("path").and_then(|x| x.as_str()).map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        *RECENT_PROJECTS.write() = projects;
    }
}

fn handle_session_result(res: &Value) {
    if let Some(sid) = res.get("sessionId").and_then(|x| x.as_str()) {
        *SESSION_ID.write() = Some(sid.to_string());
    }
    if let Some(opts) = res.get("configOptions") {
        set_config_options(opts);
    }
}

async fn project_mcp_servers(cwd: &str) -> Value {
    invoke("mcp_servers", json!({"cwd": cwd})).await.unwrap_or(json!([]))
}

pub async fn new_session() {
    let Some(cwd) = PROJECT.read().clone() else { return };
    reset_thread();
    *SESSION_ID.write() = None;
    let mcp = project_mcp_servers(&cwd).await;
    match invoke("acp_request", json!({"method": "session/new", "params": {"cwd": cwd, "mcpServers": mcp}}))
        .await
    {
        Ok(res) => {
            handle_session_result(&res);
            refresh_sessions().await;
        }
        Err(e) => {
            let msg = err_msg(&e);
            if e.get("code").and_then(|c| c.as_i64()) == Some(-32000) || msg.contains("auth") {
                *NEEDS_LOGIN.write() = true;
            } else {
                *ERROR.write() = Some(msg);
            }
        }
    }
}

pub async fn load_session(meta: SessionMeta) {
    reset_thread();
    *SESSION_ID.write() = Some(meta.id.clone());
    *PROJECT.write() = Some(meta.cwd.clone());
    let mcp = project_mcp_servers(&meta.cwd).await;
    match invoke(
        "acp_request",
        json!({"method": "session/load", "params": {"sessionId": meta.id, "cwd": meta.cwd, "mcpServers": mcp}}),
    )
    .await
    {
        Ok(res) => handle_session_result(&res),
        Err(e) => *ERROR.write() = Some(err_msg(&e)),
    }
}

pub async fn send_prompt(text: String) {
    let Some(sid) = SESSION_ID.read().clone() else { return };
    let attachments = ATTACHMENTS.write().drain(..).collect::<Vec<_>>();
    let label = if attachments.is_empty() {
        text.clone()
    } else {
        format!("{text}\n[{} image(s) attached]", attachments.len())
    };
    ITEMS.write().push(Item::User(label));
    *RUNNING.write() = true;
    let mut blocks = vec![json!({"type": "text", "text": text})];
    for a in attachments {
        blocks.push(json!({"type": "image", "data": a.data, "mimeType": a.mime}));
    }
    let res = invoke(
        "acp_request",
        json!({"method": "session/prompt", "params": {"sessionId": sid, "prompt": blocks}}),
    )
    .await;
    *RUNNING.write() = false;
    if let Err(e) = res {
        *ERROR.write() = Some(err_msg(&e));
    }
    refresh_sessions().await;
}

pub async fn cancel_turn() {
    if let Some(sid) = SESSION_ID.read().clone() {
        let _ = invoke("acp_notify", json!({"method": "session/cancel", "params": {"sessionId": sid}})).await;
    }
}

pub async fn set_config(config_id: String, value: String) {
    let Some(sid) = SESSION_ID.read().clone() else { return };
    let res = if config_id == "mode" {
        invoke("acp_request", json!({"method": "session/set_mode", "params": {"sessionId": sid, "modeId": value}}))
            .await
    } else {
        invoke(
            "acp_request",
            json!({"method": "session/set_config_option", "params": {"sessionId": sid, "configId": config_id, "value": value}}),
        )
        .await
    };
    match res {
        Ok(r) => {
            if let Some(opts) = r.get("configOptions") {
                set_config_options(opts);
            } else if let Some(opt) = CONFIG_OPTIONS.write().iter_mut().find(|o| o.id == config_id) {
                opt.current = value.clone();
            }
        }
        Err(e) => *ERROR.write() = Some(err_msg(&e)),
    }
}

pub async fn refresh_diff() {
    if let Some(cwd) = PROJECT.read().clone() {
        if let Ok(res) = invoke("git_diff", json!({"cwd": cwd})).await {
            let diff = res.get("diff").and_then(|d| d.as_str()).unwrap_or("").to_string();
            let files = res
                .get("files")
                .and_then(|f| f.as_array())
                .map(|a| a.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>().join("\n"))
                .unwrap_or_default();
            *DIFF.write() = if diff.is_empty() && files.is_empty() {
                "No uncommitted changes.".to_string()
            } else {
                format!("{files}\n\n{diff}")
            };
        }
    }
}
