//! Root component: event subscriptions and top-level layout.

use crate::actions::{connect, refresh_projects, refresh_sessions};
use crate::components::*;
use crate::ipc::listen_forever;
use crate::state::*;
use dioxus::prelude::*;
use serde_json::Value;

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
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            // F-011.5/.6: approval preferences — auto-approve short-circuits
            // the modal via the normal acp_respond_permission path.
            let kind = tool.get("kind").and_then(|x| x.as_str()).unwrap_or("").to_string();
            let settings = APP_SETTINGS.read().clone();
            if let Some(option_id) = crate::conversation::auto_approve_option(
                settings.yolo,
                &settings.approvals,
                &kind,
                &title,
                &options,
            ) {
                spawn(async move {
                    let _ = crate::ipc::invoke(
                        "acp_respond_permission",
                        serde_json::json!({"requestId": request_id, "outcome": {"outcome": "selected", "optionId": option_id}}),
                    ).await;
                });
                return;
            }
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
        // F-002.8: delegated click handler for the copy buttons that
        // md_to_html injects into rendered code blocks.
        document::eval(
            "if (!window.__kimiCodeCopy) { \
                window.__kimiCodeCopy = true; \
                document.addEventListener('click', (e) => { \
                    const btn = e.target.closest('.code-copy-btn'); \
                    if (!btn) return; \
                    const pre = btn.parentElement.querySelector('pre'); \
                    navigator.clipboard.writeText(pre ? pre.innerText : '').then(() => { \
                        btn.textContent = 'Copied'; \
                        btn.classList.add('copied'); \
                        setTimeout(() => { \
                            btn.textContent = 'Copy'; \
                            btn.classList.remove('copied'); \
                        }, 1500); \
                    }); \
                }); \
            }",
        );
        // F-012: live session sync — the backend watches kimi's shared
        // session index and emits this whenever the CLI touches it.
        listen_forever("sessions:changed", |_| {
            spawn(async {
                refresh_sessions().await;
            });
        });
        spawn(async {
            // F-011.13: load persisted app settings before anything else so
            // the kimi binary override applies to the ACP connection.
            crate::actions::load_app_settings().await;
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
                            PendingQueue {}
                            Composer {}
                            if *TERMINAL_OPEN.read() {
                                TerminalPane {}
                            }
                        }
                        if *SHOW_DIFF.read() {
                            DiffPane {}
                        }
                        if *SHOW_MEMORY.read() {
                            MemoryPane {}
                        }
                        if *SHOW_BROWSER.read() {
                            BrowserPane {}
                        }
                    }
                }
                StatusBar {}
            }
            if PERMISSION.read().is_some() {
                PermissionModal {}
            }
            if *SHOW_NEW_SESSION.read() {
                NewSessionModal {}
            }
            if *SHOW_COMPACT_CONFIRM.read() {
                CompactConfirmModal {}
            }
            if RESUME_CONFLICT.read().is_some() {
                ResumeConflictModal {}
            }
            if *NEEDS_LOGIN.read() {
                LoginModal {}
            }
            if let Some(err) = ERROR.read().clone() {
                base::KimiToast {
                    key: "{err}",
                    message: err,
                    variant: "error",
                    duration: 8000_u64,
                    onclose: move |_| *ERROR.write() = None,
                }
            }
        }
    }
}
