//! KimiBadge — Status and label badges.

use crate::design_tokens::Colors;
use dioxus::prelude::*;

#[component]
pub fn KimiBadge(
    children: Element,
    #[props(default = "default".to_string())] variant: String,
    #[props(default = "small".to_string())] size: String,
) -> Element {
    let (bg, text, border) = match variant.as_str() {
        "blue" => (Colors::KIMI_BLUE_MUTED, Colors::KIMI_BLUE, "none"),
        "green" => (Colors::SUCCESS_MUTED, Colors::SUCCESS, "none"),
        "yellow" => (Colors::WARNING_MUTED, Colors::WARNING, "none"),
        "red" => (Colors::ERROR_MUTED, Colors::ERROR, "none"),
        "gray" => (Colors::BG_SURFACE, Colors::TEXT_TERTIARY, Colors::BORDER_SUBTLE),
        _ => (Colors::BG_HOVER, Colors::TEXT_PRIMARY, "none"),
    };

    let (pad, font_size, radius) = match size.as_str() {
        "medium" => ("4px 12px", "13px", "6px"),
        _ => ("2px 8px", "12px", "4px"),
    };

    let border_style = if border != "none" {
        format!("1px solid {border}")
    } else {
        "none".to_string()
    };

    rsx! {
        span {
            style: "
                display: inline-flex;
                align-items: center;
                background: {bg};
                color: {text};
                padding: {pad};
                font-size: {font_size};
                border-radius: {radius};
                border: {border_style};
                line-height: 1.4;
                font-weight: 500;
                white-space: nowrap;
            ",
            {children}
        }
    }
}
