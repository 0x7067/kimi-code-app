//! Root component: event subscriptions and top-level layout.

use crate::actions::{connect, refresh_projects, refresh_sessions};
use crate::components::*;
use crate::ipc::listen_forever;
use crate::state::*;
use dioxus::prelude::*;
use serde_json::Value;
use wasm_bindgen::prelude::*;

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

        // Poll for new sessions started outside the app
        let window = web_sys::window().unwrap();
        let cb = Closure::wrap(Box::new(move || {
            wasm_bindgen_futures::spawn_local(async {
                refresh_sessions().await;
            });
        }) as Box<dyn FnMut()>);
        let _ = window.set_interval_with_callback_and_timeout_and_arguments_0(
            cb.as_ref().unchecked_ref(),
            3000,
        );
        cb.forget();
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
                    button { onclick: move |_| *ERROR.write() = None, "Dismiss" }
                }
            }
        }
    }
}
