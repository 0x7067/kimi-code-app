//! Session-management modals (F-003): the session-creation dialog
//! (F-003.11, with AGENTS.md preview per F-003.9), the manual-compact
//! confirmation (F-003.13), and the cross-process resume-conflict guard.

use crate::actions::{compact_session, create_session, load_session};
use crate::ipc::invoke;
use crate::markdown::md_to_html;
use crate::state::*;
use dioxus::prelude::*;
use serde_json::{json, Value};

/// F-003.11 — session creation dialog: optional name, working directory
/// (defaults to the selected project), optional initial prompt, and a
/// collapsible AGENTS.md preview. The preview is informational only: the kimi
/// CLI auto-injects AGENTS.md into the session context itself (F-003.9).
#[component]
pub fn NewSessionModal() -> Element {
    let mut name = use_signal(String::new);
    let mut work_dir = use_signal(|| PROJECT.read().clone().unwrap_or_default());
    let mut prompt = use_signal(String::new);
    let mut agents = use_signal(|| None::<(String, String)>); // (path, content)
    let mut agents_open = use_signal(|| false);

    // Re-detect AGENTS.md whenever the working directory changes (F-003.9).
    use_effect(move || {
        let dir = work_dir();
        if dir.is_empty() {
            agents.set(None);
            return;
        }
        spawn(async move {
            let found = invoke("read_agents_md", json!({"workDir": dir}))
                .await
                .ok()
                .and_then(|v| {
                    Some((
                        v.get("path")?.as_str()?.to_string(),
                        v.get("content")?.as_str()?.to_string(),
                    ))
                });
            agents.set(found);
        });
    });

    let can_create = !work_dir().trim().is_empty() && *CONNECTED.read();

    rsx! {
        div { class: "overlay",
            div { class: "modal session-modal",
                h3 { "New session" }
                label { class: "modal-label", "Session name (optional)" }
                input {
                    class: "kimi-input single",
                    r#type: "text",
                    placeholder: "e.g. Fix sidebar layout",
                    value: "{name}",
                    oninput: move |e| name.set(e.value()),
                }
                label { class: "modal-label", "Working directory" }
                div { class: "modal-row",
                    input {
                        class: "kimi-input single",
                        r#type: "text",
                        placeholder: "/path/to/project",
                        value: "{work_dir}",
                        oninput: move |e| work_dir.set(e.value()),
                    }
                    button {
                        class: "ghost",
                        onclick: move |_| {
                            spawn(async move {
                                if let Ok(Value::String(path)) = invoke("pick_folder", json!({})).await {
                                    work_dir.set(path);
                                }
                            });
                        },
                        "Browse…"
                    }
                }
                label { class: "modal-label", "Initial prompt (optional)" }
                textarea {
                    class: "kimi-input multi",
                    placeholder: "Sent as the first message after the session starts",
                    value: "{prompt}",
                    oninput: move |e| prompt.set(e.value()),
                }
                if let Some((path, content)) = agents() {
                    div { class: "agents-preview",
                        button {
                            class: "ghost agents-preview-toggle",
                            onclick: move |_| {
                                let open = *agents_open.read();
                                agents_open.set(!open);
                            },
                            span { class: if agents_open() { "chevron open" } else { "chevron" }, "›" }
                            "AGENTS.md detected"
                            span { class: "agents-preview-path", "{path}" }
                        }
                        if agents_open() {
                            div {
                                class: "agents-preview-body markdown",
                                dangerous_inner_html: md_to_html(&content),
                            }
                        }
                    }
                }
                div { class: "modal-actions",
                    button {
                        class: "ghost",
                        onclick: move |_| *SHOW_NEW_SESSION.write() = false,
                        "Cancel"
                    }
                    button {
                        class: "primary",
                        disabled: !can_create,
                        onclick: move |_| {
                            let cwd = work_dir().trim().to_string();
                            let name = Some(name()).filter(|n| !n.trim().is_empty());
                            let initial = Some(prompt()).filter(|p| !p.trim().is_empty());
                            *SHOW_NEW_SESSION.write() = false;
                            spawn(async move { create_session(cwd, name, initial).await });
                        },
                        "Create session"
                    }
                }
            }
        }
    }
}

/// F-003.13 — confirmation before sending `/compact`, stating its scope.
#[component]
pub fn CompactConfirmModal() -> Element {
    rsx! {
        div { class: "overlay",
            div { class: "modal",
                h3 { "Compact context" }
                p {
                    "This sends /compact to the agent, which summarizes this \
                     session's entire conversation so far into a shorter context. \
                     Older messages stay visible here but the agent will only \
                     retain the summary. This cannot be undone."
                }
                div { class: "modal-actions",
                    button {
                        class: "ghost",
                        onclick: move |_| *SHOW_COMPACT_CONFIRM.write() = false,
                        "Cancel"
                    }
                    button {
                        class: "primary",
                        onclick: move |_| {
                            *SHOW_COMPACT_CONFIRM.write() = false;
                            spawn(async { compact_session().await });
                        },
                        "Compact"
                    }
                }
            }
        }
    }
}

/// Soft cross-process conflict guard: shown when a clicked session's wire log
/// was written moments ago by another process (e.g. the kimi CLI).
#[component]
pub fn ResumeConflictModal() -> Element {
    let Some(meta) = RESUME_CONFLICT.read().clone() else { return rsx! {} };
    rsx! {
        div { class: "overlay",
            div { class: "modal",
                h3 { "Session active elsewhere" }
                p {
                    "This session looks active in another process (e.g. the CLI). \
                     Resuming here while it's being driven elsewhere can interleave \
                     history. Resume anyway?"
                }
                div { class: "modal-actions",
                    button {
                        class: "ghost",
                        onclick: move |_| *RESUME_CONFLICT.write() = None,
                        "Cancel"
                    }
                    button {
                        class: "primary",
                        onclick: move |_| {
                            let meta = meta.clone();
                            *RESUME_CONFLICT.write() = None;
                            spawn(async move { load_session(meta).await });
                        },
                        "Resume"
                    }
                }
            }
        }
    }
}
