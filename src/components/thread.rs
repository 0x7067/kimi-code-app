use crate::conversation::{item_matches, item_plain_text};
use crate::markdown::md_to_html;
use crate::state::*;
use dioxus::prelude::*;
use serde_json::Value;
use std::collections::HashSet;

/// Copy text to the system clipboard (F-002.8).
pub(crate) fn copy_text(text: &str) {
    if let Ok(js) = serde_json::to_string(text) {
        document::eval(&format!("navigator.clipboard.writeText({js});"));
    }
}

/// Mark message `i` as copied, then clear the flag after a short delay.
fn flash_copied(mut copied: Signal<Option<usize>>, i: usize) {
    copied.set(Some(i));
    let handle = gloo_timers::callback::Timeout::new(1500, move || {
        if *copied.peek() == Some(i) {
            copied.set(None);
        }
    });
    handle.forget();
}

#[component]
pub fn ThreadView() -> Element {
    let copied = use_signal(|| None::<usize>);
    let collapsed = use_signal(|| HashSet::<usize>::new());
    use_effect(move || {
        let items = ITEMS.read();
        let running = *RUNNING.read();
        // Track total output length of in-progress tools so auto-scroll fires
        // when CLI output streams in in real time.
        let cli_output_len: usize = items
            .iter()
            .filter_map(|item| {
                if let Item::Tool(tc) = item {
                    if tc.status == "in_progress" {
                        return Some(tc.output.len());
                    }
                }
                None
            })
            .sum();
        let _ = (items.len(), running, cli_output_len);
        document::eval(
            "requestAnimationFrame(() => { \
                const t = document.getElementById('thread'); \
                if (t) { \
                    const near = t.scrollHeight - t.scrollTop - t.clientHeight < 120; \
                    if (near) t.scrollTop = t.scrollHeight; \
                } \
            });",
        );
    });
    let query = CONVO_SEARCH.read().trim().to_string();
    let hits = if query.is_empty() {
        0
    } else {
        ITEMS.read().iter().filter(|item| item_matches(item, &query)).count()
    };
    rsx! {
        if *SEARCH_OPEN.read() {
            div { class: "convo-search",
                input {
                    class: "convo-search-input",
                    r#type: "text",
                    placeholder: "Search in conversation…",
                    value: "{CONVO_SEARCH}",
                    autofocus: true,
                    oninput: move |e| *CONVO_SEARCH.write() = e.value(),
                    onkeydown: move |e| {
                        if e.key() == Key::Escape {
                            e.prevent_default();
                            *SEARCH_OPEN.write() = false;
                            CONVO_SEARCH.write().clear();
                        }
                    },
                }
                if !query.is_empty() {
                    span { class: "convo-search-count",
                        if hits == 1 { "1 match" } else { "{hits} matches" }
                    }
                }
                button {
                    class: "ghost",
                    onclick: move |_| {
                        *SEARCH_OPEN.write() = false;
                        CONVO_SEARCH.write().clear();
                    },
                    "Close"
                }
            }
        }
        if !PLAN.read().is_empty() {
            div { class: "plan-sticky",
                div { class: "plan-head", "Plan" }
                for (i, entry) in PLAN.read().iter().enumerate() {
                    div { key: "{i}", class: "plan-entry {entry.status}",
                        span { class: "plan-status {entry.status}",
                            {match entry.status.as_str() {
                                "completed" => "✓",
                                "in_progress" => "▶",
                                _ => "○",
                            }}
                        }
                        span { class: "plan-content", "{entry.content}" }
                    }
                }
            }
        }
        if *SHOW_CHECKPOINTS.read() {
            {checkpoint_panel()}
        }
        div { class: "thread", id: "thread",
            if ITEMS.read().is_empty() && SESSION_ID.read().is_none() {
                div { class: "thread-hero",
                    div { class: "thread-hero-icon", "✦" }
                    h2 { "Welcome to Kimi Code" }
                    p { "Pick a project and start a new session, or resume one from the sidebar." }
                }
            }
            for (i, item) in ITEMS.read().iter().enumerate() {
                {render_item(i, item, copied, &query, collapsed)}
            }
            if *RUNNING.read() {
                div { class: "working", span { class: "spinner" } "Working…" }
            }
        }
    }
}

/// Search modifier class for an item: highlight hits, dim everything else.
fn search_class(item: &Item, query: &str) -> &'static str {
    if query.is_empty() {
        ""
    } else if item_matches(item, query) {
        " search-hit"
    } else {
        " search-dim"
    }
}

/// Per-message copy button (F-002.8).
fn copy_button(i: usize, item: &Item, copied: Signal<Option<usize>>) -> Element {
    let text = item_plain_text(item);
    let is_copied = *copied.read() == Some(i);
    rsx! {
        button {
            class: if is_copied { "msg-copy copied" } else { "msg-copy" },
            title: "Copy message",
            onclick: move |_| {
                copy_text(&text);
                flash_copied(copied, i);
            },
            if is_copied { "Copied" } else { "Copy" }
        }
    }
}

/// Edit button for user messages (F-002.7).
fn edit_button(i: usize, text: &str) -> Element {
    let text = text.to_string();
    rsx! {
        button {
            class: "msg-copy",
            title: "Edit and resend",
            onclick: move |_| {
                *COMPOSER_EDIT_INDEX.write() = Some(i);
                *COMPOSER_PREFILL.write() = Some(text.clone());
            },
            "Edit"
        }
    }
}

/// Action bar below agent messages.
fn action_bar(i: usize, item: &Item, copied: Signal<Option<usize>>) -> Element {
    let text = item_plain_text(item);
    let is_copied = *copied.read() == Some(i);
    rsx! {
        div { class: "msg-actions",
            button {
                class: "msg-action-btn",
                title: "Copy message",
                onclick: move |_| {
                    copy_text(&text);
                    flash_copied(copied, i);
                },
                if is_copied { "Copied" } else { "Copy" }
            }
        }
    }
}

fn render_item(
    i: usize,
    item: &Item,
    copied: Signal<Option<usize>>,
    query: &str,
    mut collapsed: Signal<HashSet<usize>>,
) -> Element {
    let sc = search_class(item, query);
    match item {
        Item::User(text) => rsx! {
            div { key: "{i}", class: "msg user{sc}",
                {copy_button(i, item, copied)}
                {edit_button(i, text)}
                div { class: "bubble", "{text}" }
            }
        },
        Item::Agent(text) => {
            let is_collapsed = collapsed.read().contains(&i);
            let sid = SESSION_ID.read().clone();
            let title = sid
                .as_ref()
                .and_then(|sid| SESSION_TITLES.read().get(sid).cloned())
                .unwrap_or_else(|| "Agent response".to_string());
            rsx! {
                div { key: "{i}", class: "msg agent{sc}",
                    div { class: "agent-header",
                        span { class: "agent-header-title", "{title}" }
                        span { class: "agent-header-duration", "Worked for 2m 34s" }
                        button {
                            class: "agent-header-expand",
                            onclick: move |_| {
                                let mut set = collapsed.read().clone();
                                if set.contains(&i) {
                                    set.remove(&i);
                                } else {
                                    set.insert(i);
                                }
                                collapsed.set(set);
                            },
                            if is_collapsed { "Show more" } else { "Show less" }
                        }
                    }
                    if !is_collapsed {
                        div { class: "bubble md", dangerous_inner_html: md_to_html(text) }
                        {action_bar(i, item, copied)}
                    }
                }
            }
        }
        Item::Thought(text) => rsx! {
            details { key: "{i}", class: "thought{sc}",
                summary { "Thinking" }
                div { class: "thought-body", "{text}" }
            }
        },
        Item::Cancelled => rsx! {
            div { key: "{i}", class: "turn-cancelled", span { "cancelled" } }
        },
        Item::Tool(tc) => rsx! {
            details {
                key: "{i}",
                class: "tool {tc.status}{sc}",
                open: tc.status == "in_progress",
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

/// File attachment card (placeholder for future wiring).
#[component]
pub fn FileCard(name: String, mime: String, meta: String) -> Element {
    let icon = if mime.starts_with("image/") {
        "🖼"
    } else if mime.contains("pdf") {
        "📄"
    } else {
        "📎"
    };
    rsx! {
        div { class: "file-card",
            div { class: "file-card-icon", "{icon}" }
            div { class: "file-card-info",
                div { class: "file-card-name", "{name}" }
                div { class: "file-card-meta", "{meta}" }
            }
        }
    }
}

/// F-002.6: checkpoint panel rendered above the thread when SHOW_CHECKPOINTS is true.
fn checkpoint_panel() -> Element {
    let mut name_input = use_signal(String::new);
    let has_session = SESSION_ID.read().is_some();
    rsx! {
        div { class: "checkpoint-panel",
            div { class: "checkpoint-head",
                span { "Checkpoints" }
                button {
                    class: "ghost",
                    onclick: move |_| { *SHOW_CHECKPOINTS.write() = false; },
                    "Close"
                }
            }
            if has_session {
                div { class: "checkpoint-save-row",
                    input {
                        class: "checkpoint-input",
                        r#type: "text",
                        placeholder: "Checkpoint name…",
                        value: "{name_input}",
                        oninput: move |e| name_input.set(e.value()),
                    }
                    button {
                        class: "primary",
                        onclick: move |_| {
                            let text = name_input.read().trim().to_string();
                            if !text.is_empty() {
                                spawn(async move {
                                    crate::actions::save_checkpoint(&text).await;
                                });
                                name_input.set(String::new());
                            }
                        },
                        "Save"
                    }
                }
            }
            div { class: "checkpoint-list",
                for cp in CHECKPOINTS.read().clone() {
                    {
                        let name = cp.get("name").and_then(|v: &Value| v.as_str()).unwrap_or("").to_string();
                        let saved_at = cp.get("savedAt").and_then(|v: &Value| v.as_str()).unwrap_or("").to_string();
                        let name_clone = name.clone();
                        rsx! {
                            div { key: "{name}", class: "checkpoint-item",
                                div { class: "checkpoint-meta",
                                    span { class: "checkpoint-name", "{name}" }
                                    span { class: "checkpoint-time", "{saved_at}" }
                                }
                                div { class: "checkpoint-actions",
                                    button {
                                        class: "ghost",
                                        onclick: move |_| {
                                            let n = name_clone.clone();
                                            spawn(async move {
                                                crate::actions::load_checkpoint(&n).await;
                                            });
                                        },
                                        "Restore"
                                    }
                                    button {
                                        class: "ghost danger",
                                        onclick: move |_| {
                                            let n = name.clone();
                                            spawn(async move {
                                                crate::actions::delete_checkpoint(&n).await;
                                            });
                                        },
                                        "Delete"
                                    }
                                }
                            }
                        }
                    }
                }
                if CHECKPOINTS.read().is_empty() {
                    div { class: "checkpoint-empty", "No checkpoints saved yet." }
                }
            }
        }
    }
}
