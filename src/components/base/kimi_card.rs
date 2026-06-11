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
        "medium" => "12px",
        "large" => "16px",
        _ => "12px",
    };

    let border = if active {
        "1px solid #1E90FF"
    } else {
        "1px solid #2E2E2E"
    };

    let shadow = if active {
        "0 0 0 1px #1E90FF, 0 4px 12px rgba(30,144,255,0.1)"
    } else {
        "0 0 0 1px #2E2E2E, 0 4px 12px rgba(0,0,0,0.2)"
    };

    let hover_border = if hoverable && !active { "hover:border-[#3E3E3E]" } else { "" };
    let hover_shadow = if hoverable && !active {
        "hover:shadow-[0_0_0_1px_#3E3E3E,0_8px_24px_rgba(0,0,0,0.3)]"
    } else {
        ""
    };

    let pad = if padding { "16px" } else { "0px" };

    rsx! {
        div {
            style: "
                background: #1E1E1E;
                border: {border};
                border-radius: {radius_px};
                box-shadow: {shadow};
                padding: {pad};
                transition: all 200ms ease-out;
            ",
            class: "{hover_border} {hover_shadow}",
            {children}
        }
    }
}
