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
                for name in ["Preferences", "config.toml", "tui.toml", "mcp.json", "AGENTS.md"] {
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
            ApprovalsSection {}
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
