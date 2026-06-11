use crate::markdown::md_to_html;
use crate::state::*;
use dioxus::prelude::*;

#[component]
pub fn ThreadView() -> Element {
    use_effect(move || {
        let _ = (ITEMS.read().len(), RUNNING.read());
        document::eval("requestAnimationFrame(() => { const t = document.getElementById('thread'); if (t) t.scrollTop = t.scrollHeight; });");
    });
    rsx! {
        div { class: "thread", id: "thread",
            if ITEMS.read().is_empty() && SESSION_ID.read().is_none() {
                div { class: "empty",
                    h2 { "Welcome to Kimi Code" }
                    p { "Pick a project and start a new session, or resume one from the sidebar." }
                }
            }
            if !PLAN.read().is_empty() {
                div { class: "plan-panel",
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
            for (i, item) in ITEMS.read().iter().enumerate() {
                {render_item(i, item)}
            }
            if *RUNNING.read() {
                div { class: "working", span { class: "spinner" } "Working…" }
            }
        }
    }
}

fn render_item(i: usize, item: &Item) -> Element {
    match item {
        Item::User(text) => rsx! {
            div { key: "{i}", class: "msg user", div { class: "bubble", "{text}" } }
        },
        Item::Agent(text) => rsx! {
            div { key: "{i}", class: "msg agent",
                div { class: "bubble md", dangerous_inner_html: md_to_html(text) }
            }
        },
        Item::Thought(text) => rsx! {
            details { key: "{i}", class: "thought",
                summary { "Thinking" }
                div { class: "thought-body", "{text}" }
            }
        },
        Item::Tool(tc) => rsx! {
            details { key: "{i}", class: "tool {tc.status}",
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
