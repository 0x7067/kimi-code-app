//! StatusBar — bottom agent status bar (F-002.14) per DESIGN_SYSTEM.md §5.5:
//! connection indicator, model name, current operation, and color-coded
//! context-usage bar (F-003.12).

use crate::conversation::{can_compact, usage_color};
use crate::state::*;
use dioxus::prelude::*;

#[component]
pub fn StatusBar() -> Element {
    let connected = *CONNECTED.read();
    let usage = *CONTEXT_USAGE.read();
    let pct = (usage * 100.0).round() as u32;
    let fill = usage_color(usage);

    // Model name from the session's config options, falling back to agent info.
    let model = CONFIG_OPTIONS
        .read()
        .iter()
        .find(|o| o.id == "model")
        .map(|o| {
            o.options
                .iter()
                .find(|so| so.value == o.current)
                .map(|so| so.name.clone())
                .unwrap_or_else(|| o.current.clone())
        })
        .unwrap_or_else(|| AGENT_INFO.read().clone());

    let op = if *RUNNING.read() {
        ITEMS
            .read()
            .iter()
            .rev()
            .find_map(|item| match item {
                Item::Tool(tc) if tc.status == "in_progress" => Some(tc.title.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "Working…".to_string())
    } else if connected {
        "Ready".to_string()
    } else {
        "Disconnected".to_string()
    };

    rsx! {
        footer { class: "statusbar",
            span { class: if connected { "status-dot connected" } else { "status-dot" } }
            span { class: "status-model", "{model}" }
            // F-011.6: prominent reminder while every tool call is auto-approved.
            if APP_SETTINGS.read().yolo {
                span { class: "status-yolo", title: "YOLO mode — all tool calls auto-approved", "YOLO" }
            }
            span { class: "status-op", "{op}" }
            div { class: "status-spacer" }
            // F-003.13: manual compact trigger, gated off while a turn runs.
            button {
                class: "ghost status-compact-btn",
                title: "Summarize this session's context (/compact)",
                disabled: !can_compact(connected, SESSION_ID.read().is_some(), *RUNNING.read()),
                onclick: move |_| { *SHOW_COMPACT_CONFIRM.write() = true; },
                "Compact"
            }
            span { class: "status-ctx-label", "Context {pct}%" }
            div { class: "status-ctx-bar",
                div {
                    class: "status-ctx-fill",
                    style: "width: {pct}%; background: {fill};",
                }
            }
        }
    }
}
