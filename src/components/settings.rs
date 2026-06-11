//! SettingsView (F-011): Preferences pane (binary, auth, model, thinking,
//! approvals, YOLO) plus the raw kimi config-file editor tabs. All
//! preferences live in `APP_SETTINGS` and persist via the backend
//! app-settings store, applying immediately without restart (F-011.13).

use crate::actions::{save_app_settings, set_config};
use crate::ipc::invoke;
use crate::state::*;
use dioxus::prelude::*;
use serde_json::{json, Value};

/// Mutate the in-memory settings and persist them (F-011.13).
fn update_settings(f: impl FnOnce(&mut AppSettings)) {
    f(&mut APP_SETTINGS.write());
    spawn(async { save_app_settings().await });
}

#[component]
pub fn SettingsView() -> Element {
    let mut file = use_signal(|| "Preferences".to_string());
    let mut content = use_signal(String::new);
    let mut status = use_signal(String::new);

    let mut load = move |name: String| {
        file.set(name.clone());
        status.set(String::new());
        if name == "Preferences" {
            return;
        }
        spawn(async move {
            match invoke("read_kimi_config", json!({"name": name})).await {
                Ok(Value::String(s)) => content.set(s),
                Ok(_) => content.set(String::new()),
                Err(e) => status.set(err_msg(&e)),
            }
        });
    };

    let prefs_open = *file.read() == "Preferences";

    rsx! {
        div { class: "settings",
            div { class: "settings-tabs",
                for name in ["Preferences", "MCP Servers", "config.toml", "tui.toml", "mcp.json", "AGENTS.md"] {
                    button {
                        key: "{name}",
                        class: if *file.read() == name { "tab active" } else { "tab" },
                        onclick: move |_| load(name.to_string()),
                        "{name}"
                    }
                }
            }
            if prefs_open {
                PreferencesPane {}
            } else if *file.read() == "MCP Servers" {
                McpServersPane {}
            } else {
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
                                    Ok(_) => status.set("Saved".into()),
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
}

#[component]
fn PreferencesPane() -> Element {
    rsx! {
        div { class: "prefs",
            BinarySection {}
            AuthSection {}
            ModelSection {}
            ThinkingSection {}
            ContextSection {}
            ApprovalsSection {}
            MemorySection {}
        }
    }
}

// ---------- F-011.1: kimi binary ----------

#[component]
fn BinarySection() -> Element {
    let mut detected = use_signal(|| None::<Value>);

    let redetect = move || {
        let override_path = APP_SETTINGS.read().kimi_bin_override.clone();
        spawn(async move {
            let res = invoke("detect_kimi_binary", json!({"overridePath": override_path}))
                .await
                .unwrap_or(json!({"found": false}));
            detected.set(Some(res));
        });
    };

    use_effect(move || {
        redetect();
    });

    let info = detected.read().clone();
    let (found, path, version, source) = match &info {
        Some(v) => (
            v.get("found").and_then(|x| x.as_bool()).unwrap_or(false),
            v.get("path").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            v.get("version").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            v.get("source").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        ),
        None => (false, String::new(), String::new(), String::new()),
    };
    let override_value = APP_SETTINGS.read().kimi_bin_override.clone().unwrap_or_default();

    rsx! {
        section { class: "prefs-section",
            h3 { "Kimi binary" }
            if info.is_none() {
                p { class: "prefs-hint", "Detecting…" }
            } else if found {
                p { class: "prefs-detected",
                    code { "{path}" }
                    span { class: "prefs-version", " v{version}" }
                    if source == "override" {
                        span { class: "prefs-tag", "manual override" }
                    }
                }
            } else {
                p { class: "prefs-warning", "kimi binary not found — install Kimi Code or set a path below." }
            }
            div { class: "prefs-row",
                input {
                    class: "prefs-input",
                    placeholder: "Manual path override (blank = auto-detect)",
                    value: "{override_value}",
                    onchange: move |e| {
                        let v = e.value().trim().to_string();
                        update_settings(|s| {
                            s.kimi_bin_override = if v.is_empty() { None } else { Some(v) };
                        });
                        redetect();
                    },
                }
                button {
                    class: "ghost",
                    onclick: move |_| {
                        spawn(async move {
                            if let Ok(Value::String(p)) = invoke("pick_file", json!({})).await {
                                update_settings(|s| s.kimi_bin_override = Some(p));
                                redetect();
                            }
                        });
                    },
                    "Browse…"
                }
                button { class: "ghost", onclick: move |_| redetect(), "Re-detect" }
            }
        }
    }
}

// ---------- F-011.2: authentication ----------

#[component]
fn AuthSection() -> Element {
    let mut logged_in = use_signal(|| None::<bool>);

    let refresh = move || {
        spawn(async move {
            let res = invoke("kimi_auth_status", json!({})).await;
            logged_in.set(Some(
                res.ok()
                    .and_then(|v| v.get("loggedIn").and_then(|x| x.as_bool()))
                    .unwrap_or(false),
            ));
        });
    };
    // Initial check + re-check whenever the login modal finishes running.
    use_effect(move || {
        let _running = *LOGIN_RUNNING.read(); // subscribe
        refresh();
    });

    let state = *logged_in.read();
    rsx! {
        section { class: "prefs-section",
            h3 { "Authentication" }
            div { class: "prefs-row",
                match state {
                    None => rsx! { span { class: "prefs-hint", "Checking…" } },
                    Some(true) => rsx! { span { class: "prefs-auth ok", "Signed in to Kimi" } },
                    Some(false) => rsx! { span { class: "prefs-auth missing", "Not signed in" } },
                }
                button {
                    class: "primary",
                    onclick: move |_| *NEEDS_LOGIN.write() = true,
                    if state == Some(true) { "Re-authenticate" } else { "Login" }
                }
            }
            p { class: "prefs-hint",
                "Tokens are stored by the kimi CLI itself; this app never stores credentials. The CLI has no logout command — remove ~/.kimi-code/credentials to sign out."
            }
        }
    }
}

// ---------- F-011.3: model selection ----------

#[component]
fn ModelSection() -> Element {
    let mut models = use_signal(Vec::<(String, String)>::new);
    let mut current = use_signal(String::new);
    let mut saved = use_signal(String::new);

    use_effect(move || {
        spawn(async move {
            if let Ok(v) = invoke("list_kimi_models", json!({})).await {
                let list = v
                    .get("models")
                    .and_then(|m| m.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|m| {
                                Some((
                                    m.get("id")?.as_str()?.to_string(),
                                    m.get("name")?.as_str()?.to_string(),
                                ))
                            })
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                models.set(list);
                if let Some(d) = v.get("default").and_then(|d| d.as_str()) {
                    current.set(d.to_string());
                }
            }
        });
    });

    let mut apply = move |value: String| {
        let value = value.trim().to_string();
        if value.is_empty() {
            return;
        }
        current.set(value.clone());
        spawn(async move {
            // Persist as the CLI default (config.toml)…
            match invoke("set_default_model", json!({"model": value.clone()})).await {
                Ok(_) => saved.set("Saved as default model".into()),
                Err(e) => {
                    saved.set(String::new());
                    *ERROR.write() = Some(err_msg(&e));
                    return;
                }
            }
            // …and switch the live session too when the agent exposes a model
            // config option, keeping the status bar's model display in sync.
            let has_model_opt = CONFIG_OPTIONS.read().iter().any(|o| o.id == "model");
            if has_model_opt {
                set_config("model".into(), value).await;
            }
        });
    };

    let known = models.read().clone();
    let cur = current.read().clone();
    let is_known = known.iter().any(|(id, _)| *id == cur);

    rsx! {
        section { class: "prefs-section",
            h3 { "Model" }
            div { class: "prefs-row",
                if !known.is_empty() {
                    select {
                        class: "cfg-select",
                        value: if is_known { "{cur}" } else { "" },
                        onchange: move |e| apply(e.value()),
                        if !is_known {
                            option { value: "", selected: true, disabled: true, "Custom: {cur}" }
                        }
                        for (id, name) in known.iter() {
                            option { key: "{id}", value: "{id}", selected: *id == cur, "{name} ({id})" }
                        }
                    }
                }
                input {
                    class: "prefs-input",
                    placeholder: "Custom model id (e.g. kimi-code/kimi-for-coding)",
                    value: "{cur}",
                    onchange: move |e| apply(e.value()),
                }
            }
            if !saved.read().is_empty() {
                p { class: "prefs-hint", "{saved}" }
            }
        }
    }
}

// ---------- F-011.4: thinking-mode default ----------

#[component]
fn ThinkingSection() -> Element {
    let mode = APP_SETTINGS.read().thinking_default.clone();
    rsx! {
        section { class: "prefs-section",
            h3 { "Thinking mode default" }
            div { class: "prefs-col",
                for (value, label) in [
                    ("always", "Always — every send uses thinking"),
                    ("never", "Never — only ⌘⇧⏎ uses thinking"),
                    ("ask", "Ask — decide per message (⏎ vs ⌘⇧⏎)"),
                ] {
                    label { key: "{value}", class: "prefs-radio",
                        input {
                            r#type: "radio",
                            name: "thinking-default",
                            value: "{value}",
                            checked: mode == value,
                            onchange: move |_| update_settings(|s| s.thinking_default = value.into()),
                        }
                        "{label}"
                    }
                }
            }
        }
    }
}

// ---------- F-011.5 / F-011.6: approvals & YOLO ----------

#[component]
fn ApprovalsSection() -> Element {
    let settings = APP_SETTINGS.read().clone();
    let yolo = settings.yolo;
    let rows: [(&'static str, &'static str, String); 4] = [
        ("shell", "Shell commands", settings.approvals.shell),
        ("file-edit", "File edits", settings.approvals.file_edit),
        ("mcp", "MCP tools", settings.approvals.mcp),
        ("git", "Git operations", settings.approvals.git),
    ];
    rsx! {
        section { class: "prefs-section",
            h3 { "Tool approvals" }
            for (id, label, value) in rows {
                div { key: "{id}", class: "prefs-row prefs-approval",
                    span { class: "prefs-approval-label", "{label}" }
                    select {
                        class: "cfg-select",
                        disabled: yolo,
                        value: "{value}",
                        onchange: move |e| {
                            let v = e.value();
                            update_settings(|s| {
                                match id {
                                    "shell" => s.approvals.shell = v,
                                    "file-edit" => s.approvals.file_edit = v,
                                    "git" => s.approvals.git = v,
                                    _ => s.approvals.mcp = v,
                                }
                            });
                        },
                        option { value: "ask", selected: value == "ask", "Ask every time" }
                        option { value: "auto", selected: value == "auto", "Auto-approve" }
                    }
                }
            }
            div { class: if yolo { "prefs-yolo active" } else { "prefs-yolo" },
                label { class: "prefs-radio",
                    input {
                        r#type: "checkbox",
                        checked: yolo,
                        onchange: move |e| update_settings(|s| s.yolo = e.checked()),
                    }
                    strong { "YOLO mode — auto-approve everything" }
                }
                p { class: "prefs-warning",
                    "Danger: the agent will run shell commands, edit files, and call tools without asking. Use only in throwaway environments."
                }
            }
        }
    }
}

// ---------- F-011.7: context limit & auto-compact ----------

#[component]
fn ContextSection() -> Element {
    let settings = APP_SETTINGS.read().clone();
    rsx! {
        section { class: "prefs-section",
            h3 { "Context limit" }
            div { class: "prefs-row prefs-approval",
                label { class: "prefs-label",
                    input {
                        r#type: "checkbox",
                        checked: settings.auto_compact,
                        onchange: move |e| update_settings(|s| s.auto_compact = e.checked()),
                    }
                    "Auto-compact when usage exceeds threshold"
                }
            }
            div { class: "prefs-row",
                label { class: "prefs-label", "Compact threshold" }
                select {
                    class: "cfg-select",
                    disabled: !settings.auto_compact,
                    value: "{(settings.auto_compact_threshold * 100.0).round() as u32}",
                    onchange: move |e| {
                        if let Ok(pct) = e.value().parse::<f64>() {
                            update_settings(|s| s.auto_compact_threshold = pct / 100.0);
                        }
                    },
                    option { value: "60", "60%" }
                    option { value: "70", "70%" }
                    option { value: "80", "80%" }
                    option { value: "85", "85%" }
                    option { value: "90", "90%" }
                }
            }
        }
    }
}

// ---------- F-007.1/11: memory preferences ----------

#[component]
fn MemorySection() -> Element {
    let settings = APP_SETTINGS.read().clone();
    rsx! {
        section { class: "prefs-section",
            h3 { "Memory preferences" }
            p { class: "prefs-hint",
                "These preferences are saved to your app settings and passed to the agent as context."
            }
            div { class: "prefs-row",
                label { class: "prefs-label", "Tech stack" }
                input {
                    class: "prefs-input",
                    placeholder: "e.g. Rust + Dioxus + Tauri, TypeScript + React…",
                    value: "{settings.tech_stack}",
                    onchange: move |e| update_settings(|s| s.tech_stack = e.value()),
                }
            }
            div { class: "prefs-row",
                label { class: "prefs-label", "Coding style" }
                input {
                    class: "prefs-input",
                    placeholder: "e.g. Prefer functional, minimal mutability, type-driven design…",
                    value: "{settings.coding_style}",
                    onchange: move |e| update_settings(|s| s.coding_style = e.value()),
                }
            }
            div { class: "prefs-row",
                label { class: "prefs-label", "Naming conventions" }
                input {
                    class: "prefs-input",
                    placeholder: "e.g. snake_case for Rust, camelCase for JS, PascalCase for components…",
                    value: "{settings.naming_conventions}",
                    onchange: move |e| update_settings(|s| s.naming_conventions = e.value()),
                }
            }
        }
    }
}

// ---------- F-005: MCP server management UI ----------

#[component]
pub fn McpServersPane() -> Element {
    let mut servers = use_signal(Vec::<Value>::new);
    let mut loading = use_signal(|| true);
    let mut editing = use_signal(|| None::<Value>);
    let mut status = use_signal(String::new);

    let mut refresh = move || {
        loading.set(true);
        spawn(async move {
            match invoke("list_mcp_servers", json!({})).await {
                Ok(Value::Object(mut o)) => {
                    let list = o.remove("servers").and_then(|v| v.as_array().cloned()).unwrap_or_default();
                    servers.set(list);
                    status.set(String::new());
                }
                Ok(_) => servers.set(Vec::new()),
                Err(e) => status.set(err_msg(&e)),
            }
            loading.set(false);
        });
    };

    use_effect(move || {
        refresh();
    });

    rsx! {
        div { class: "mcp-pane",
            div { class: "mcp-head",
                h3 { "MCP Servers" }
                button {
                    class: "primary",
                    onclick: move |_| {
                        editing.set(Some(json!({
                            "name": "",
                            "transport": "stdio",
                            "command": "",
                            "args": [],
                            "url": "",
                            "env": {},
                            "enabled": true
                        })));
                    },
                    "+ Add server"
                }
            }
            if *loading.read() {
                p { class: "prefs-hint", "Loading…" }
            } else if servers.read().is_empty() {
                div { class: "mcp-empty",
                    p { "No MCP servers configured." }
                    p { class: "prefs-hint", "Add a server to extend the agent with external tools." }
                }
            } else {
                div { class: "mcp-list",
                    for srv in servers.read().iter().cloned() {
                        {
                            let key = srv.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            rsx! {
                                McpServerCard {
                                    key: "{key}",
                                    server: srv,
                                    on_edit: move |s| editing.set(Some(s)),
                                    on_delete: move |name| {
                                        spawn(async move {
                                            match invoke("delete_mcp_server", json!({"name": name})).await {
                                                Ok(_) => refresh(),
                                                Err(e) => status.set(err_msg(&e)),
                                            }
                                        });
                                    },
                                }
                            }
                        }
                    }
                }
            }
            if !status.read().is_empty() {
                p { class: "prefs-warning", "{status}" }
            }
            if let Some(srv) = editing.read().clone() {
                McpServerEditModal {
                    server: srv,
                    on_save: move |s| {
                        spawn(async move {
                            match invoke("save_mcp_server", json!({"server": s})).await {
                                Ok(_) => {
                                    editing.set(None);
                                    refresh();
                                }
                                Err(e) => status.set(err_msg(&e)),
                            }
                        });
                    },
                    on_cancel: move || editing.set(None),
                }
            }
        }
    }
}

#[component]
fn McpServerCard(server: Value, on_edit: EventHandler<Value>, on_delete: EventHandler<String>) -> Element {
    let name = server.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let transport = server.get("transport").and_then(|v| v.as_str()).unwrap_or("stdio").to_string();
    let enabled = server.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);
    let status = server.get("status").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let name_for_delete = name.clone();
    let server_for_toggle = server.clone();
    let server_for_edit = server.clone();

    rsx! {
        div { class: "mcp-card",
            div { class: "mcp-card-main",
                div { class: "mcp-card-top",
                    span { class: "mcp-name", "{name}" }
                    span { class: "mcp-badge mcp-badge-{transport}", "{transport}" }
                    span { class: "mcp-status mcp-status-{status}", "{status}" }
                }
                div { class: "mcp-card-meta",
                    if transport == "stdio" {
                        if let Some(cmd) = server.get("command").and_then(|v| v.as_str()) {
                            code { "{cmd}" }
                        }
                    } else {
                        if let Some(url) = server.get("url").and_then(|v| v.as_str()) {
                            span { "{url}" }
                        }
                    }
                }
            }
            div { class: "mcp-card-actions",
                label { class: "mcp-toggle",
                    input {
                        r#type: "checkbox",
                        checked: enabled,
                        onchange: move |e| {
                            let mut s = server_for_toggle.clone();
                            if let Some(o) = s.as_object_mut() {
                                o.insert("enabled".into(), json!(e.checked()));
                            }
                            on_edit.call(s);
                        },
                    }
                    span { class: "mcp-toggle-track",
                        span { class: "mcp-toggle-thumb" }
                    }
                }
                button {
                    class: "ghost icon-btn",
                    onclick: move |_| on_edit.call(server_for_edit.clone()),
                    "Edit"
                }
                button {
                    class: "danger icon-btn",
                    onclick: move |_| on_delete.call(name_for_delete.clone()),
                    "Delete"
                }
            }
        }
    }
}

#[component]
fn McpServerEditModal(server: Value, on_save: EventHandler<Value>, on_cancel: EventHandler<()>) -> Element {
    let mut draft = use_signal(|| server.clone());

    let name = draft.read().get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let transport = draft.read().get("transport").and_then(|v| v.as_str()).unwrap_or("stdio").to_string();
    let is_new = name.is_empty();
    let cmd_val = draft.read().get("command").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let args_val = draft.read().get("args").and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>().join("\n")).unwrap_or_default();
    let url_val = draft.read().get("url").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let env_val = draft.read().get("env").and_then(|v| v.as_object()).map(|o| o.iter().map(|(k,v)| format!("{}={}", k, v.as_str().unwrap_or(""))).collect::<Vec<_>>().join("\n")).unwrap_or_default();

    rsx! {
        div { class: "overlay",
            div { class: "modal mcp-modal",
                h3 { if is_new { "Add MCP server" } else { "Edit MCP server" } }
                div { class: "modal-label", "Name" }
                input {
                    class: "prefs-input",
                    placeholder: "e.g. filesystem",
                    value: "{name}",
                    disabled: !is_new,
                    oninput: move |e| {
                        let mut d = draft.write().clone();
                        if let Some(o) = d.as_object_mut() {
                            o.insert("name".into(), json!(e.value()));
                        }
                        draft.set(d);
                    },
                }
                div { class: "modal-label", "Transport" }
                select {
                    class: "cfg-select",
                    value: "{transport}",
                    onchange: move |e| {
                        let mut d = draft.write().clone();
                        if let Some(o) = d.as_object_mut() {
                            o.insert("transport".into(), json!(e.value()));
                        }
                        draft.set(d);
                    },
                    option { value: "stdio", selected: transport == "stdio", "stdio (local command)" }
                    option { value: "http", selected: transport == "http", "HTTP (remote endpoint)" }
                }
                if transport == "stdio" {
                    div { class: "modal-label", "Command" }
                    input {
                        class: "prefs-input",
                        placeholder: "e.g. npx -y @modelcontextprotocol/server-filesystem",
                        value: "{cmd_val}",
                        oninput: move |e| {
                            let mut d = draft.write().clone();
                            if let Some(o) = d.as_object_mut() {
                                o.insert("command".into(), json!(e.value()));
                            }
                            draft.set(d);
                        },
                    }
                    div { class: "modal-label", "Arguments (one per line)" }
                    textarea {
                        class: "settings-editor",
                        style: "min-height: 60px;",
                        value: "{args_val}",
                        oninput: move |e| {
                            let lines: Vec<String> = e.value().lines().map(|s| s.to_string()).collect();
                            let mut d = draft.write().clone();
                            if let Some(o) = d.as_object_mut() {
                                o.insert("args".into(), json!(lines));
                            }
                            draft.set(d);
                        },
                    }
                } else {
                    div { class: "modal-label", "URL" }
                    input {
                        class: "prefs-input",
                        placeholder: "https://mcp.example.com/v1",
                        value: "{url_val}",
                        oninput: move |e| {
                            let mut d = draft.write().clone();
                            if let Some(o) = d.as_object_mut() {
                                o.insert("url".into(), json!(e.value()));
                            }
                            draft.set(d);
                        },
                    }
                }
                div { class: "modal-label", "Environment variables (KEY=VALUE, one per line)" }
                textarea {
                    class: "settings-editor",
                    style: "min-height: 60px;",
                    value: "{env_val}",
                    oninput: move |e| {
                        let mut map = serde_json::Map::new();
                        for line in e.value().lines() {
                            if let Some((k, v)) = line.split_once('=') {
                                map.insert(k.trim().to_string(), json!(v.trim().to_string()));
                            }
                        }
                        let mut d = draft.write().clone();
                        if let Some(o) = d.as_object_mut() {
                            o.insert("env".into(), Value::Object(map));
                        }
                        draft.set(d);
                    },
                }
                div { class: "modal-actions",
                    button { class: "ghost", onclick: move |_| on_cancel.call(()), "Cancel" }
                    button {
                        class: "primary",
                        disabled: name.trim().is_empty(),
                        onclick: move |_| on_save.call(draft.read().clone()),
                        "Save"
                    }
                }
            }
        }
    }
}
