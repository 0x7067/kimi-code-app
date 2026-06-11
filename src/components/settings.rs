use crate::ipc::invoke;
use crate::state::err_msg;
use dioxus::prelude::*;
use serde_json::{json, Value};

#[component]
pub fn SettingsView() -> Element {
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
                        onclick: |_| load(name.to_string()),
                        "{name}"
                    }
                }
            }
            textarea {
                class: "settings-editor",
                spellcheck: false,
                value: "{content}",
                oninput: |e| content.set(e.value()),
            }
            div { class: "settings-actions",
                span { class: "settings-status", "{status}" }
                button {
                    class: "primary",
                    onclick: |_| {
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
