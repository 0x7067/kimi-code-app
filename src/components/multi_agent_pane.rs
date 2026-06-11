//! F-004: Minimal multi-agent orchestration — worktree management UI.

use crate::ipc::invoke;
use crate::state::*;
use dioxus::prelude::*;
use serde_json::Value;

#[component]
pub fn MultiAgentPane() -> Element {
    let mut worktrees = use_signal(Vec::<Value>::new);
    let mut loading = use_signal(|| false);
    let mut new_name = use_signal(String::new);

    let mut refresh = move || {
        if let Some(cwd) = PROJECT.read().clone() {
            loading.set(true);
            spawn(async move {
                match invoke("list_worktrees", serde_json::json!({"cwd": cwd})).await {
                    Ok(Value::Array(list)) => worktrees.set(list),
                    _ => {}
                }
                loading.set(false);
            });
        }
    };

    use_effect(move || {
        refresh();
    });

    let create = move || {
        let name = new_name.read().trim().to_string();
        if name.is_empty() {
            return;
        }
        if let Some(cwd) = PROJECT.read().clone() {
            spawn(async move {
                match invoke("create_worktree", serde_json::json!({"cwd": cwd, "name": name})).await {
                    Ok(_) => {
                        new_name.set(String::new());
                        refresh();
                    }
                    Err(e) => *ERROR.write() = Some(format!("Create worktree failed: {e}")),
                }
            });
        }
    };

    let remove = move |path: String| {
        if let Some(cwd) = PROJECT.read().clone() {
            spawn(async move {
                match invoke("remove_worktree", serde_json::json!({"cwd": cwd, "path": path})).await {
                    Ok(_) => refresh(),
                    Err(e) => *ERROR.write() = Some(format!("Remove worktree failed: {e}")),
                }
            });
        }
    };

    let list = worktrees.read().clone();

    rsx! {
        div { class: "multi-agent-pane",
            div { class: "multi-agent-head",
                span { "Multi-agent" }
                button {
                    class: "ghost",
                    onclick: move |_| *SHOW_MULTI_AGENT.write() = false,
                    "Close"
                }
            }
            div { class: "multi-agent-body",
                div { class: "multi-agent-create",
                    input {
                        class: "prefs-input",
                        placeholder: "worktree name…",
                        value: "{new_name}",
                        oninput: move |e| new_name.set(e.value()),
                        onkeydown: move |e| {
                            if e.key() == Key::Enter { create(); }
                        },
                    }
                    button {
                        class: "primary",
                        onclick: move |_| create(),
                        "Create worktree"
                    }
                }
                if *loading.read() {
                    p { class: "memory-hint", "Loading worktrees…" }
                } else if list.is_empty() {
                    p { class: "memory-hint", "No worktrees found." }
                } else {
                    div { class: "multi-agent-list",
                        for wt in list.iter() {
                            {
                                let path = wt.get("path").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                let branch = wt.get("branch").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                let is_main = wt.get("main").and_then(|v| v.as_bool()).unwrap_or(false);
                                rsx! {
                                    div {
                                        key: "{path}",
                                        class: "multi-agent-row",
                                        div { class: "multi-agent-info",
                                            span { class: "multi-agent-branch",
                                                if is_main { "main" } else { "{branch}" }
                                            }
                                            span { class: "multi-agent-path", "{path}" }
                                        }
                                        if !is_main {
                                            button {
                                                class: "ghost danger",
                                                onclick: move |_| remove(path.clone()),
                                                "Remove"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
