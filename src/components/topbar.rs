use crate::actions::{connect, refresh_diff};
use crate::components::base::{KimiDropdown, KimiDropdownItem};
use crate::conversation::{export_filename, export_json, export_markdown};
use crate::state::*;
use dioxus::prelude::*;

/// Trigger a browser download of `content` as `name` (F-002.10).
fn download_file(name: &str, mime: &str, content: &str) {
    if let (Ok(n), Ok(c)) = (serde_json::to_string(name), serde_json::to_string(content)) {
        document::eval(&format!(
            "(() => {{ \
                const blob = new Blob([{c}], {{ type: '{mime}' }}); \
                const a = document.createElement('a'); \
                a.href = URL.createObjectURL(blob); \
                a.download = {n}; \
                a.click(); \
                setTimeout(() => URL.revokeObjectURL(a.href), 1000); \
            }})();"
        ));
    }
}

/// Today's date as YYYY-MM-DD from the webview clock.
fn today() -> String {
    String::from(js_sys::Date::new_0().to_iso_string()).chars().take(10).collect()
}

/// Export the current thread as Markdown or JSON.
fn export_conversation(as_markdown: bool) {
    let items = ITEMS.read().clone();
    if items.is_empty() {
        return;
    }
    let sid = SESSION_ID.read().clone().unwrap_or_default();
    let title = SESSION_TITLES
        .read()
        .get(&sid)
        .cloned()
        .unwrap_or_else(|| "Conversation".to_string());
    let date = today();
    if as_markdown {
        download_file(
            &export_filename(&sid, &date, "md"),
            "text/markdown",
            &export_markdown(&title, &items),
        );
    } else {
        download_file(
            &export_filename(&sid, &date, "json"),
            "application/json",
            &export_json(&sid, &items),
        );
    }
}

#[component]
pub fn Topbar() -> Element {
    let connected = *CONNECTED.read();
    let has_items = !ITEMS.read().is_empty();
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
                    class: if *SEARCH_OPEN.read() { "ghost active" } else { "ghost" },
                    title: "Search in conversation (⌘F)",
                    onclick: move |_| {
                        let open = !*SEARCH_OPEN.read();
                        *SEARCH_OPEN.write() = open;
                        if !open { CONVO_SEARCH.write().clear(); }
                    },
                    "Search"
                }
                if has_items {
                    KimiDropdown {
                        trigger: rsx! {
                            button { class: "ghost", title: "Export conversation", "Export" }
                        },
                        KimiDropdownItem {
                            onclick: move |_| export_conversation(true),
                            "Export as Markdown"
                        }
                        KimiDropdownItem {
                            onclick: move |_| export_conversation(false),
                            "Export as JSON"
                        }
                    }
                }
                button {
                    class: if *SHOW_CHECKPOINTS.read() { "ghost active" } else { "ghost" },
                    title: "Session checkpoints",
                    onclick: move |_| {
                        let open = !*SHOW_CHECKPOINTS.read();
                        *SHOW_CHECKPOINTS.write() = open;
                        if open { spawn(crate::actions::refresh_checkpoints()); }
                    },
                    "Checkpoints"
                }
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
                    class: if *TERMINAL_OPEN.read() { "ghost active" } else { "ghost" },
                    title: "Toggle terminal",
                    onclick: move |_| {
                        let open = !*TERMINAL_OPEN.read();
                        *TERMINAL_OPEN.write() = open;
                    },
                    "Terminal"
                }
                button {
                    class: if *SHOW_MEMORY.read() { "ghost active" } else { "ghost" },
                    title: "Project memory",
                    onclick: move |_| {
                        let open = !*SHOW_MEMORY.read();
                        *SHOW_MEMORY.write() = open;
                    },
                    "Memory"
                }
                button {
                    class: if *SHOW_BROWSER.read() { "ghost active" } else { "ghost" },
                    title: "Browser preview",
                    onclick: move |_| {
                        let open = !*SHOW_BROWSER.read();
                        *SHOW_BROWSER.write() = open;
                    },
                    "Browser"
                }
                button {
                    class: if *SHOW_MULTI_AGENT.read() { "ghost active" } else { "ghost" },
                    title: "Multi-agent worktrees",
                    onclick: move |_| {
                        let open = !*SHOW_MULTI_AGENT.read();
                        *SHOW_MULTI_AGENT.write() = open;
                    },
                    "Multi-agent"
                }
                button {
                    class: if *SHOW_AUTOMATIONS.read() { "ghost active" } else { "ghost" },
                    title: "Automations",
                    onclick: move |_| {
                        let open = !*SHOW_AUTOMATIONS.read();
                        *SHOW_AUTOMATIONS.write() = open;
                    },
                    "Automations"
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
