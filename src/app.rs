use crate::state::*;
use crate::tauri::{invoke, listen_forever};
use dioxus::prelude::*;
use serde_json::{json, Value};

fn md_to_html(text: &str) -> String {
    let parser = pulldown_cmark::Parser::new_ext(text, pulldown_cmark::Options::ENABLE_TABLES | pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, parser);
    html
}

// ---------- actions ----------

async fn connect() {
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

async fn refresh_sessions() {
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

async fn refresh_projects() {
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

async fn new_session() {
    let Some(cwd) = PROJECT.read().clone() else { return };
    reset_thread();
    *SESSION_ID.write() = None;
    let mcp = project_mcp_servers(&cwd).await;
    match invoke("acp_request", json!({"method": "session/new", "params": {"cwd": cwd, "mcpServers": mcp}})).await {
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

async fn load_session(meta: SessionMeta) {
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

async fn send_prompt(text: String) {
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

async fn cancel_turn() {
    if let Some(sid) = SESSION_ID.read().clone() {
        let _ = invoke("acp_notify", json!({"method": "session/cancel", "params": {"sessionId": sid}})).await;
    }
}

async fn set_config(config_id: String, value: String) {
    let Some(sid) = SESSION_ID.read().clone() else { return };
    let res = if config_id == "mode" {
        invoke("acp_request", json!({"method": "session/set_mode", "params": {"sessionId": sid, "modeId": value}})).await
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
            } else {
                for opt in CONFIG_OPTIONS.write().iter_mut() {
                    if opt.id == config_id {
                        opt.current = value.clone();
                    }
                }
            }
        }
        Err(e) => *ERROR.write() = Some(err_msg(&e)),
    }
}

async fn refresh_diff() {
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

// ---------- root ----------

#[component]
pub fn App() -> Element {
    use_effect(|| {
        listen_forever("acp:update", |payload| apply_update(&payload));
        listen_forever("acp:permission_request", |payload| {
            let request_id = payload.get("requestId").and_then(|x| x.as_u64()).unwrap_or(0);
            let params = payload.get("params").cloned().unwrap_or(Value::Null);
            let tool = params.get("toolCall").cloned().unwrap_or(Value::Null);
            let title = tool.get("title").and_then(|x| x.as_str()).unwrap_or("Tool call").to_string();
            let detail = tool
                .get("rawInput")
                .map(|v| serde_json::to_string_pretty(v).unwrap_or_default())
                .unwrap_or_default();
            let options = params
                .get("options")
                .and_then(|o| o.as_array())
                .map(|arr| {
                    arr.iter()
                        .map(|o| {
                            (
                                o.get("optionId").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                                o.get("name").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                                o.get("kind").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                            )
                        })
                        .collect()
                })
                .unwrap_or_default();
            *PERMISSION.write() = Some(PermissionRequest { request_id, title, detail, options });
        });
        listen_forever("acp:disconnected", |_| {
            *CONNECTED.write() = false;
            *RUNNING.write() = false;
        });
        listen_forever("login:line", |payload| {
            if let Some(line) = payload.as_str() {
                LOGIN_LINES.write().push(line.to_string());
            }
        });
        listen_forever("login:done", |payload| {
            *LOGIN_RUNNING.write() = false;
            if payload.as_i64() == Some(0) {
                *NEEDS_LOGIN.write() = false;
                spawn(async {
                    connect().await;
                    refresh_sessions().await;
                });
            }
        });
        spawn(async {
            connect().await;
            refresh_projects().await;
            refresh_sessions().await;
        });
    });

    rsx! {
        div { class: "shell",
            Sidebar {}
            main { class: "main",
                Topbar {}
                if *VIEW.read() == View::Settings {
                    SettingsView {}
                } else {
                    div { class: "workspace",
                        div { class: "thread-col",
                            ThreadView {}
                            Composer {}
                        }
                        if *SHOW_DIFF.read() {
                            DiffPane {}
                        }
                    }
                }
            }
            if PERMISSION.read().is_some() {
                PermissionModal {}
            }
            if *NEEDS_LOGIN.read() {
                LoginModal {}
            }
            if let Some(err) = ERROR.read().clone() {
                div { class: "toast",
                    span { "{err}" }
                    button { onclick: move |_| *ERROR.write() = None, "✕" }
                }
            }
        }
    }
}

// ---------- chrome ----------

#[component]
fn Topbar() -> Element {
    let connected = *CONNECTED.read();
    rsx! {
        header { class: "topbar",
            div { class: "topbar-left",
                span { class: if connected { "dot ok" } else { "dot bad" } }
                span { class: "agent-name", "{AGENT_INFO}" }
                if !connected {
                    button { class: "ghost", onclick: move |_| { spawn(connect()); }, "Reconnect" }
                }
            }
            div { class: "topbar-right",
                button {
                    class: if *SHOW_DIFF.read() { "ghost active" } else { "ghost" },
                    onclick: move |_| {
                        let now = !*SHOW_DIFF.read();
                        *SHOW_DIFF.write() = now;
                        if now { spawn(refresh_diff()); }
                    },
                    "Diff"
                }
                button {
                    class: if *VIEW.read() == View::Settings { "ghost active" } else { "ghost" },
                    onclick: move |_| {
                        let v = *VIEW.read();
                        *VIEW.write() = if v == View::Settings { View::Chat } else { View::Settings };
                    },
                    "Settings"
                }
            }
        }
    }
}

#[component]
fn Sidebar() -> Element {
    let project = PROJECT.read().clone();
    let sessions = SESSIONS.read().clone();
    let query = SESSION_SEARCH.read().to_lowercase();
    let filtered: Vec<SessionMeta> = sessions
        .iter()
        .filter(|sess| project.as_ref().map_or(true, |p| &sess.cwd == p))
        .filter(|sess| {
            query.is_empty()
                || sess.title.to_lowercase().contains(&query)
                || sess.cwd.to_lowercase().contains(&query)
        })
        .cloned()
        .collect();

    rsx! {
        aside { class: "sidebar",
            div { class: "sidebar-head",
                span { class: "brand", "Kimi Code" }
            }
            div { class: "project-picker",
                select {
                    value: project.clone().unwrap_or_default(),
                    onchange: move |e| {
                        let v = e.value();
                        *PROJECT.write() = if v.is_empty() { None } else { Some(v) };
                    },
                    option { value: "", "All projects" }
                    for p in RECENT_PROJECTS.read().iter() {
                        option { value: "{p}", selected: Some(p) == project.as_ref(),
                            {p.rsplit('/').next().unwrap_or(p).to_string()}
                        }
                    }
                }
                button {
                    class: "ghost",
                    title: "Open folder…",
                    onclick: move |_| {
                        spawn(async {
                            if let Ok(Value::String(path)) = invoke("pick_folder", json!({})).await {
                                if !RECENT_PROJECTS.read().contains(&path) {
                                    RECENT_PROJECTS.write().insert(0, path.clone());
                                }
                                *PROJECT.write() = Some(path);
                            }
                        });
                    },
                    "📁"
                }
            }
            button {
                class: "primary new-session",
                disabled: PROJECT.read().is_none() || !*CONNECTED.read(),
                onclick: move |_| { spawn(new_session()); },
                "＋ New session"
            }
            input {
                class: "session-search",
                r#type: "search",
                placeholder: "Search sessions…",
                value: "{SESSION_SEARCH}",
                oninput: move |e| *SESSION_SEARCH.write() = e.value(),
            }
            div { class: "session-list",
                for sess in filtered {
                    {
                        let active = SESSION_ID.read().as_deref() == Some(sess.id.as_str());
                        let meta = sess.clone();
                        rsx! {
                            div {
                                key: "{sess.id}",
                                class: if active { "session-item active" } else { "session-item" },
                                onclick: move |_| { spawn(load_session(meta.clone())); },
                                div { class: "session-title", "{sess.title}" }
                                div { class: "session-meta",
                                    {sess.cwd.rsplit('/').next().unwrap_or("").to_string()}
                                    " · "
                                    {sess.updated_at.get(..10).unwrap_or("").to_string()}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ---------- thread ----------

#[component]
fn ThreadView() -> Element {
    let items = ITEMS.read().clone();
    let plan = PLAN.read().clone();
    use_effect(move || {
        let _ = (ITEMS.read().len(), RUNNING.read());
        document::eval("requestAnimationFrame(() => { const t = document.getElementById('thread'); if (t) t.scrollTop = t.scrollHeight; });");
    });
    rsx! {
        div { class: "thread", id: "thread",
            if items.is_empty() && SESSION_ID.read().is_none() {
                div { class: "empty",
                    h2 { "Welcome to Kimi Code" }
                    p { "Pick a project and start a new session, or resume one from the sidebar." }
                }
            }
            if !plan.is_empty() {
                div { class: "plan-panel",
                    div { class: "plan-head", "Plan" }
                    for (i, entry) in plan.iter().enumerate() {
                        div { key: "{i}", class: "plan-entry {entry.status}",
                            span { class: "plan-status",
                                {match entry.status.as_str() {
                                    "completed" => "✓",
                                    "in_progress" => "▶",
                                    _ => "○",
                                }}
                            }
                            span { "{entry.content}" }
                        }
                    }
                }
            }
            for (i, item) in items.iter().enumerate() {
                {render_item(i, item)}
            }
            if *RUNNING.read() {
                div { class: "working", span { class: "spinner" } " Working…" }
            }
        }
    }
}

fn render_item(i: usize, item: &Item) -> Element {
    match item {
        Item::User(text) => rsx! {
            div { key: "{i}", class: "msg user", div { class: "bubble", "{text}" } }
        },
        Item::Agent(text) => rsx! {
            div { key: "{i}", class: "msg agent",
                div { class: "bubble md", dangerous_inner_html: md_to_html(text) }
            }
        },
        Item::Thought(text) => rsx! {
            details { key: "{i}", class: "thought",
                summary { "Thinking" }
                div { class: "thought-body", "{text}" }
            }
        },
        Item::Tool(tc) => rsx! {
            details { key: "{i}", class: "tool {tc.status}",
                summary {
                    span { class: "tool-badge {tc.status}",
                        {match tc.status.as_str() {
                            "completed" => "✓",
                            "failed" => "✕",
                            "in_progress" => "…",
                            _ => "·",
                        }}
                    }
                    span { class: "tool-kind", "{tc.kind}" }
                    span { class: "tool-title", "{tc.title}" }
                }
                if !tc.output.is_empty() {
                    pre { class: "tool-output", "{tc.output}" }
                }
            }
        },
    }
}

// ---------- composer ----------

#[component]
fn Composer() -> Element {
    let mut draft = use_signal(String::new);
    let running = *RUNNING.read();
    let has_session = SESSION_ID.read().is_some();
    let config_opts = CONFIG_OPTIONS.read().clone();
    let commands = COMMANDS.read().clone();
    let show_slash = draft.read().starts_with('/') && !draft.read().contains(' ');
    let filter = draft.read().trim_start_matches('/').to_string();

    let mut submit = move || {
        let text = draft.read().trim().to_string();
        if text.is_empty() || *RUNNING.read() || SESSION_ID.read().is_none() {
            return;
        }
        draft.set(String::new());
        spawn(send_prompt(text));
    };

    rsx! {
        div { class: "composer",
            if show_slash && !commands.is_empty() {
                div { class: "slash-menu",
                    for cmd in commands.iter().filter(|c| c.name.starts_with(&filter)).take(8) {
                        {
                            let name = cmd.name.clone();
                            rsx! {
                                div {
                                    key: "{cmd.name}",
                                    class: "slash-item",
                                    onclick: move |_| draft.set(format!("/{name} ")),
                                    span { class: "slash-name", "/{cmd.name}" }
                                    span { class: "slash-desc", "{cmd.description}" }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "composer-box",
                if !ATTACHMENTS.read().is_empty() {
                    div { class: "attachments",
                        for (i, a) in ATTACHMENTS.read().iter().enumerate() {
                            span { key: "{i}", class: "attachment-chip",
                                "🖼 {a.name}"
                                button {
                                    class: "chip-x",
                                    onclick: move |_| { ATTACHMENTS.write().remove(i); },
                                    "✕"
                                }
                            }
                        }
                    }
                }
                textarea {
                    placeholder: if has_session { "Message Kimi…  ( / for commands, Enter to send)" } else { "Start a session first" },
                    value: "{draft}",
                    disabled: !has_session,
                    oninput: move |e| draft.set(e.value()),
                    onkeydown: move |e| {
                        if e.key() == Key::Enter && !e.modifiers().shift() {
                            e.prevent_default();
                            submit();
                        }
                    },
                }
                div { class: "composer-controls",
                    button {
                        class: "ghost",
                        title: "Attach image",
                        disabled: !has_session,
                        onclick: move |_| {
                            spawn(async {
                                if let Ok(Value::Object(img)) = invoke("pick_image", json!({})).await {
                                    ATTACHMENTS.write().push(Attachment {
                                        name: img.get("name").and_then(|v| v.as_str()).unwrap_or("image").into(),
                                        mime: img.get("mimeType").and_then(|v| v.as_str()).unwrap_or("image/png").into(),
                                        data: img.get("data").and_then(|v| v.as_str()).unwrap_or("").into(),
                                    });
                                }
                            });
                        },
                        "🖼"
                    }
                    for opt in config_opts {
                        {
                            let id = opt.id.clone();
                            rsx! {
                                select {
                                    key: "{opt.id}",
                                    class: "cfg-select",
                                    title: "{opt.name}",
                                    value: "{opt.current}",
                                    onchange: move |e| { spawn(set_config(id.clone(), e.value())); },
                                    for so in opt.options.iter() {
                                        option { value: "{so.value}", selected: so.value == opt.current, "{so.name}" }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "spacer" }
                    if running {
                        button { class: "danger", onclick: move |_| { spawn(cancel_turn()); }, "Stop" }
                    } else {
                        button { class: "primary", disabled: !has_session, onclick: move |_| submit(), "Send" }
                    }
                }
            }
        }
    }
}

// ---------- modals & panes ----------

#[component]
fn PermissionModal() -> Element {
    let Some(req) = PERMISSION.read().clone() else { return rsx! {} };
    rsx! {
        div { class: "overlay",
            div { class: "modal",
                h3 { "Permission required" }
                p { class: "perm-title", "{req.title}" }
                if !req.detail.is_empty() {
                    pre { class: "perm-detail", "{req.detail}" }
                }
                div { class: "modal-actions",
                    for (option_id, name, kind) in req.options.clone() {
                        {
                            let rid = req.request_id;
                            let oid = option_id.clone();
                            let class = if kind.starts_with("allow") { "primary" } else { "ghost" };
                            rsx! {
                                button {
                                    key: "{option_id}",
                                    class: "{class}",
                                    onclick: move |_| {
                                        let oid = oid.clone();
                                        *PERMISSION.write() = None;
                                        spawn(async move {
                                            let _ = invoke(
                                                "acp_respond_permission",
                                                json!({"requestId": rid, "outcome": {"outcome": "selected", "optionId": oid}}),
                                            ).await;
                                        });
                                    },
                                    "{name}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn LoginModal() -> Element {
    let lines = LOGIN_LINES.read().clone();
    let running = *LOGIN_RUNNING.read();
    rsx! {
        div { class: "overlay",
            div { class: "modal",
                h3 { "Sign in to Kimi" }
                p { "Authenticate with your Kimi account using the device-code flow." }
                if !lines.is_empty() {
                    pre { class: "login-output", {lines.join("\n")} }
                }
                div { class: "modal-actions",
                    button {
                        class: "primary",
                        disabled: running,
                        onclick: move |_| {
                            LOGIN_LINES.write().clear();
                            *LOGIN_RUNNING.write() = true;
                            spawn(async {
                                let _ = invoke("kimi_login", json!({})).await;
                            });
                        },
                        if running { "Waiting for login…" } else { "Login with Kimi" }
                    }
                    button { class: "ghost", onclick: move |_| *NEEDS_LOGIN.write() = false, "Close" }
                }
            }
        }
    }
}

#[component]
fn DiffPane() -> Element {
    let diff = DIFF.read().clone();
    rsx! {
        aside { class: "diff-pane",
            div { class: "diff-head",
                span { "Working tree changes" }
                button { class: "ghost", onclick: move |_| { spawn(refresh_diff()); }, "↻" }
            }
            pre { class: "diff-body",
                for (i, line) in diff.lines().enumerate() {
                    span {
                        key: "{i}",
                        class: if line.starts_with('+') && !line.starts_with("+++") { "dl add" }
                               else if line.starts_with('-') && !line.starts_with("---") { "dl del" }
                               else if line.starts_with("@@") { "dl hunk" }
                               else { "dl" },
                        "{line}\n"
                    }
                }
            }
        }
    }
}

#[component]
fn SettingsView() -> Element {
    let mut file = use_signal(|| "config.toml".to_string());
    let mut content = use_signal(String::new);
    let mut status = use_signal(String::new);

    let mut load = move |name: String| {
        file.set(name.clone());
        status.set(String::new());
        spawn(async move {
            match invoke("read_kimi_config", json!({"name": name})).await {
                Ok(Value::String(s)) => content.set(s),
                Ok(_) => content.set(String::new()),
                Err(e) => status.set(err_msg(&e)),
            }
        });
    };

    use_effect(move || {
        load("config.toml".to_string());
    });

    rsx! {
        div { class: "settings",
            div { class: "settings-tabs",
                for name in ["config.toml", "tui.toml", "mcp.json", "AGENTS.md"] {
                    button {
                        key: "{name}",
                        class: if *file.read() == name { "tab active" } else { "tab" },
                        onclick: move |_| load(name.to_string()),
                        "{name}"
                    }
                }
            }
            textarea {
                class: "settings-editor",
                spellcheck: false,
                value: "{content}",
                oninput: move |e| content.set(e.value()),
            }
            div { class: "settings-actions",
                span { class: "settings-status", "{status}" }
                button {
                    class: "primary",
                    onclick: move |_| {
                        let name = file.read().clone();
                        let body = content.read().clone();
                        spawn(async move {
                            match invoke("write_kimi_config", json!({"name": name, "content": body})).await {
                                Ok(_) => status.set("Saved ✓".into()),
                                Err(e) => status.set(err_msg(&e)),
                            }
                        });
                    },
                    "Save"
                }
            }
        }
    }
}
