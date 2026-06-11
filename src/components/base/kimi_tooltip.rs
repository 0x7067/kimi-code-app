//! KimiTooltip — Hover tooltip for icon buttons and truncated text.

use crate::design_tokens::Colors;
use dioxus::prelude::*;

#[component]
pub fn KimiTooltip(
    children: Element,
    content: String,
    #[props(default = "top".to_string())] position: String,
) -> Element {
    let mut show = use_signal(|| false);

    let (arrow_offset, transform) = match position.as_str() {
        "bottom" => ("top: -4px;", "translateY(4px)"),
        "left" => ("right: -4px;", "translateX(-4px)"),
        "right" => ("left: -4px;", "translateX(4px)"),
        _ => ("bottom: -4px;", "translateY(-4px)"),
    };

    rsx! {
        div {
            class: "relative inline-flex items-center",
            onmouseenter: move |_| show.set(true),
            onmouseleave: move |_| show.set(false),
            {children}
            if show() {
                div {
                    style: "
                        position: absolute;
                        {arrow_offset}
                        left: 50%;
                        transform: translateX(-50%) {transform};
                        background: {BG};
                        color: {TEXT};
                        padding: 4px 8px;
                        border-radius: 8px;
                        font-size: 12px;
                        line-height: 1.4;
                        white-space: nowrap;
                        z-index: 50;
                        box-shadow: 0 4px 12px rgba(0,0,0,0.3);
                        pointer-events: none;
                        animation: fade-in 150ms ease-out;
                    ",
                    "{content}"
                }
            }
        }
    }
}

// Helper constants for the style string above
#[allow(dead_code)] // used via rsx attribute interpolation
const BG: &str = Colors::BG_HOVER;
#[allow(dead_code)] // used via rsx attribute interpolation
const TEXT: &str = Colors::TEXT_PRIMARY;
