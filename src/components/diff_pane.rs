use crate::actions::refresh_diff;
use crate::state::*;
use dioxus::prelude::*;

#[component]
pub fn DiffPane() -> Element {
    let diff = DIFF.read().clone();
    rsx! {
        aside { class: "diff-pane", "data-testid": "diff-pane",
            div { class: "diff-head",
                span { "Working tree changes" }
                button {
                    "data-testid": "diff-refresh",
                    class: "ghost icon-btn",
                    onclick: move |_| { spawn(refresh_diff()); },
                    "Refresh"
                }
            }
            pre { class: "diff-body", "data-testid": "diff-body",
                for (i, line) in diff.lines().enumerate() {
                    {
                        let delay = (i.min(24)) * 18;
                        rsx! {
                            span {
                                key: "{i}",
                                class: if line.starts_with('+') && !line.starts_with("+++") { "dl add" }
                                       else if line.starts_with('-') && !line.starts_with("---") { "dl del" }
                                       else if line.starts_with("@@") { "dl hunk" }
                                       else { "dl" },
                                style: "animation-delay: {delay}ms;",
                                "{line}\n"
                            }
                        }
                    }
                }
            }
        }
    }
}
