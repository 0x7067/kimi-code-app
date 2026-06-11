//! KimiCard — Elevated surface container for content sections.

use dioxus::prelude::*;

#[component]
pub fn KimiCard(
    children: Element,
    #[props(default = false)] hoverable: bool,
    #[props(default = false)] active: bool,
    #[props(default = true)] padding: bool,
    #[props(default = "medium".to_string())] radius: String,
) -> Element {
    let radius_px = match radius.as_str() {
        "small" => "8px",
        "large" => "16px",
        _ => "12px",
    };

    let hover_cls = if hoverable && !active { "hoverable" } else { "" };
    let active_cls = if active { "active" } else { "" };
    let pad = if padding { "16px" } else { "0px" };

    rsx! {
        div {
            class: "kimi-card {hover_cls} {active_cls}",
            style: "border-radius: {radius_px}; padding: {pad};",
            {children}
        }
    }
}
