use crate::actions::refresh_diff;
use crate::state::*;
use dioxus::prelude::*;

#[component]
pub fn DiffPane() -> Element {
    let diff = DIFF.read().clone();
    rsx! {
        aside { class: "diff-pane",
            div { class: "diff-head",
                span { "Working tree changes" }
                button { class: "ghost icon-btn", onclick: move |_| { spawn(refresh_diff()); }, "Refresh" }
            }
            pre { class: "diff-body",
                for (i, line) in diff.lines().enumerate() {
                    span {
                        key: "{i}",
                        class: if line.starts_with('+') && !line.starts_with("+++") { "dl add" }
                               else if line.starts_with('-') && !line.starts_with("---") { "dl del" }
                               else if line.starts_with("@@") { "dl hunk" }
                               else { "dl" },
                        "{line}\n"
                    }
                }
            }
        }
    }
}
