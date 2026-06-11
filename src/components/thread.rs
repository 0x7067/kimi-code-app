use crate::markdown::md_to_html;
use crate::state::*;
use dioxus::prelude::*;

#[component]
pub fn ThreadView() -> Element {
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
    rsx! {
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
        div { class: "thread", id: "thread",
            if ITEMS.read().is_empty() && SESSION_ID.read().is_none() {
                div { class: "empty",
                    h2 { "Welcome to Kimi Code" }
                    p { "Pick a project and start a new session, or resume one from the sidebar." }
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
            details {
                key: "{i}",
                class: "tool {tc.status}",
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
