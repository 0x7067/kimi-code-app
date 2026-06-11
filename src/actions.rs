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

fn parse_session_meta(v: &Value) -> SessionMeta {
    let str_of = |key: &str| -> String {
        match v.get(key) {
            Some(Value::String(s)) => s.clone(),
            Some(Value::Number(n)) => n.to_string(),
            _ => String::new(),
        }
    };
    SessionMeta {
        id: str_of("sessionId"),
        // kimi's store reports `workDir`; the ACP session/list shape uses `cwd`.
        cwd: if v.get("workDir").is_some() { str_of("workDir") } else { str_of("cwd") },
        title: {
            let t = str_of("title");
            if t.is_empty() { "Untitled session".into() } else { t }
        },
        updated_at: str_of("updatedAt"),
    }
}

/// Refresh the sidebar session list (F-012). With a project selected this
/// reads kimi's shared store via `kimi_list_sessions` (so CLI sessions appear
/// too); with no project it falls back to the agent's global `session/list`.
pub async fn refresh_sessions() {
    let project = PROJECT.read().clone();
    let res = match &project {
        Some(cwd) => invoke("kimi_list_sessions", json!({"workDir": cwd})).await,
        None => invoke("acp_request", json!({"method": "session/list", "params": {}}))
            .await
            .map(|r| r.get("sessions").cloned().unwrap_or_else(|| json!([]))),
    };
    if let Ok(value) = res {
        let sessions = value
            .as_array()
            .map(|arr| arr.iter().map(parse_session_meta).collect())
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

fn semantic_title(text: &str) -> String {
    let trimmed = text.trim();
    let first = trimmed.lines().next().unwrap_or(trimmed);
    let cleaned = first.trim_start_matches('/').trim();
    if cleaned.len() > 40 {
        format!("{}…", &cleaned[..39])
    } else {
        cleaned.to_string()
    }
}

/// F-003.1/F-003.11: create a session in `cwd`, optionally naming it and
/// sending an initial prompt. AGENTS.md is NOT injected here — the kimi CLI
/// picks it up itself from `cwd` (the dialog only previews it, F-003.9).
pub async fn create_session(cwd: String, name: Option<String>, initial_prompt: Option<String>) {
    cache_current_scrollback();
    PENDING_QUEUE.write().clear(); // F-014: the queue is per-session
    reset_thread();
    *SESSION_ID.write() = None;
    *PROJECT.write() = Some(cwd.clone());
    let mcp = project_mcp_servers(&cwd).await;
    match invoke("acp_request", json!({"method": "session/new", "params": {"cwd": cwd, "mcpServers": mcp}}))
        .await
    {
        Ok(res) => {
            handle_session_result(&res);
            if let (Some(name), Some(sid)) = (name.filter(|n| !n.trim().is_empty()), SESSION_ID.read().clone()) {
                SESSION_TITLES.write().insert(sid, name.trim().to_string());
            }
            refresh_sessions().await;
            if let Some(prompt) = initial_prompt.filter(|p| !p.trim().is_empty()) {
                send_prompt(prompt, false).await;
            }
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

/// Sidebar click path: soft cross-process conflict guard. If the session's
/// wire log was touched in the last ~30s by something other than this app's
/// current session (e.g. the kimi CLI), ask before resuming; otherwise load.
pub async fn request_load_session(meta: SessionMeta) {
    let is_own = SESSION_ID.read().as_deref() == Some(meta.id.as_str());
    let age = invoke("kimi_session_activity", json!({"sessionId": meta.id}))
        .await
        .ok()
        .and_then(|v| v.as_u64());
    if crate::conversation::should_warn_resume(age, is_own) {
        *RESUME_CONFLICT.write() = Some(meta);
    } else {
        load_session(meta).await;
    }
}

pub async fn load_session(meta: SessionMeta) {
    cache_current_scrollback();
    // F-014: the pending queue is per-session — drop it unless we are
    // reloading the session it belongs to.
    if SESSION_ID.read().as_deref() != Some(meta.id.as_str()) {
        PENDING_QUEUE.write().clear();
    }
    reset_thread();
    *SESSION_ID.write() = Some(meta.id.clone());
    *PROJECT.write() = Some(meta.cwd.clone());
    // Restore cached scrollback if present
    if let Some((items, plan)) = SCROLLBACK_CACHE.read().get(&meta.id).cloned() {
        *ITEMS.write() = items;
        *PLAN.write() = plan;
    }
    let mcp = project_mcp_servers(&meta.cwd).await;
    // F-012: the backend replays stored history as acp:update events before
    // this resolves; mid-turn loads come back as a TURN_AGENT_BUSY error.
    match invoke(
        "kimi_load_session",
        json!({"sessionId": meta.id, "cwd": meta.cwd, "mcpServers": mcp}),
    )
    .await
    {
        Ok(res) => handle_session_result(&res),
        Err(e) => *ERROR.write() = Some(err_msg(&e)),
    }
}

pub async fn send_prompt(text: String, thinking: bool) {
    let Some(sid) = SESSION_ID.read().clone() else { return };
    let attachments = ATTACHMENTS.write().drain(..).collect::<Vec<_>>();
    let label = if attachments.is_empty() {
        text.clone()
    } else {
        format!("{text}\n[{} image(s) attached]", attachments.len())
    };
    let is_first = ITEMS.read().is_empty();
    ITEMS.write().push(Item::User(label));
    if is_first {
        SESSION_TITLES.write().insert(sid.clone(), semantic_title(&text));
    }
    let mut blocks = vec![json!({"type": "text", "text": text})];
    for a in attachments {
        blocks.push(json!({"type": "image", "data": a.data, "mimeType": a.mime}));
    }
    let mut params = json!({"sessionId": sid, "prompt": blocks});
    if thinking {
        // F-002.13: flag the prompt for thinking mode so the backend can map it
        // to the agent's thinking toggle when forwarding the request.
        params["_meta"] = json!({"thinking": true});
    }
    let epoch = begin_turn(&sid);
    let res = invoke("acp_request", json!({"method": "session/prompt", "params": params})).await;
    finish_turn(&sid, epoch, res);
    refresh_sessions().await;
}

/// Claim a new turn epoch (F-013/F-015), mark the turn as running, and track
/// the session in RUNNING_SESSIONS (F-003.14).
fn begin_turn(session_id: &str) -> u64 {
    let epoch = {
        let mut epoch = TURN_EPOCH.write();
        *epoch += 1;
        *epoch
    };
    *RUNNING.write() = true;
    crate::conversation::turn_started(
        &mut RUNNING_SESSIONS.write(),
        session_id,
        epoch,
        crate::conversation::now_epoch(),
    );
    epoch
}

/// Handle a resolved `session/prompt`: mark cancellations (F-013), and — if
/// this turn was not superseded by a steer — clear the running flag and
/// dispatch the next queued message (F-014).
fn finish_turn(session_id: &str, epoch: u64, res: Result<Value, Value>) {
    // F-003.14: clear the running marker unless a newer turn (steer) on this
    // session has since claimed it.
    crate::conversation::turn_finished(&mut RUNNING_SESSIONS.write(), session_id, epoch);
    match res {
        Ok(v) => {
            // A steer pushes its own marker before the replacement message,
            // so only mark here when this turn is still the latest.
            if v.get("stopReason").and_then(|s| s.as_str()) == Some("cancelled")
                && *TURN_EPOCH.read() == epoch
            {
                ITEMS.write().push(Item::Cancelled);
            }
        }
        Err(e) => *ERROR.write() = Some(err_msg(&e)),
    }
    if *TURN_EPOCH.read() == epoch {
        *RUNNING.write() = false;
        dispatch_pending();
    }
}

/// F-015: cancel the running turn and immediately send `text` in its place.
pub async fn steer_prompt(text: String) {
    let Some(sid) = SESSION_ID.read().clone() else { return };
    let epoch = begin_turn(&sid);
    {
        let mut items = ITEMS.write();
        items.push(Item::Cancelled);
        items.push(Item::User(text.clone()));
    }
    let res = invoke("acp_steer", json!({"sessionId": sid, "text": text})).await;
    finish_turn(&sid, epoch, res);
    refresh_sessions().await;
}

/// F-014: queue a message to send after the current turn ends.
pub fn enqueue_prompt(text: &str) {
    crate::conversation::queue_push(&mut PENDING_QUEUE.write(), text);
}

/// Dispatch the oldest queued message, if any (F-014). Boxed so the
/// send → finish → dispatch → send recursion has a finite future type.
fn dispatch_pending() {
    let next = crate::conversation::queue_pop_front(&mut PENDING_QUEUE.write());
    if let Some(text) = next {
        dioxus::prelude::spawn(async move {
            let fut: std::pin::Pin<Box<dyn std::future::Future<Output = ()>>> =
                Box::pin(send_prompt(text, false));
            fut.await;
        });
    }
}

/// F-003.13: manual context compaction. `/compact` is a kimi slash command
/// handled CLI-side, so it travels as an ordinary prompt for this session.
pub async fn compact_session() {
    send_prompt("/compact".to_string(), false).await;
}

pub async fn cancel_turn() {
    if let Some(sid) = SESSION_ID.read().clone() {
        let _ = invoke("acp_cancel", json!({"sessionId": sid})).await;
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

/// F-011.13: load persisted app settings from the backend store on startup.
/// The backend also applies the kimi binary override as a side effect.
pub async fn load_app_settings() {
    if let Ok(v) = invoke("read_app_settings", json!({})).await {
        if let Ok(settings) = serde_json::from_value::<AppSettings>(v) {
            *APP_SETTINGS.write() = settings;
        }
    }
}

/// F-011.13: persist the current app settings (atomic write backend-side).
pub async fn save_app_settings() {
    let settings = APP_SETTINGS.read().clone();
    let payload = serde_json::to_value(&settings).unwrap_or_else(|_| json!({}));
    if let Err(e) = invoke("write_app_settings", json!({"settings": payload})).await {
        *ERROR.write() = Some(err_msg(&e));
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

// ---------- F-002.6: checkpoint save/restore ----------

pub async fn save_checkpoint(name: &str) {
    let Some(session_id) = SESSION_ID.read().clone() else { return };
    let items = serde_json::to_value(&*ITEMS.read()).unwrap_or_else(|_| json!([]));
    let plan = serde_json::to_value(&*PLAN.read()).unwrap_or_else(|_| json!([]));
    match invoke("save_checkpoint", json!({"sessionId": session_id, "name": name, "items": items, "plan": plan})).await {
        Ok(_) => { refresh_checkpoints().await; }
        Err(e) => *ERROR.write() = Some(err_msg(&e)),
    }
}

pub async fn refresh_checkpoints() {
    let Some(session_id) = SESSION_ID.read().clone() else { return };
    match invoke("list_checkpoints", json!({"sessionId": session_id})).await {
        Ok(Value::Array(list)) => *CHECKPOINTS.write() = list,
        _ => {}
    }
}

pub async fn load_checkpoint(name: &str) {
    let Some(session_id) = SESSION_ID.read().clone() else { return };
    match invoke("load_checkpoint", json!({"sessionId": session_id, "name": name})).await {
        Ok(v) => {
            if let Ok(items) = serde_json::from_value::<Vec<Item>>(v.get("items").cloned().unwrap_or_else(|| json!([]))) {
                *ITEMS.write() = items;
            }
            if let Ok(plan) = serde_json::from_value::<Vec<PlanEntry>>(v.get("plan").cloned().unwrap_or_else(|| json!([]))) {
                *PLAN.write() = plan;
            }
        }
        Err(e) => *ERROR.write() = Some(err_msg(&e)),
    }
}

pub async fn delete_checkpoint(name: &str) {
    let Some(session_id) = SESSION_ID.read().clone() else { return };
    if let Err(e) = invoke("delete_checkpoint", json!({"sessionId": session_id, "name": name})).await {
        *ERROR.write() = Some(err_msg(&e));
    } else {
        refresh_checkpoints().await;
    }
}
