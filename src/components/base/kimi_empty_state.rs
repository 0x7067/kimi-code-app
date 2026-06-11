//! KimiEmptyState — Empty state for lists, search results, and error states.

use crate::design_tokens::Colors;
use dioxus::prelude::*;

#[component]
pub fn KimiEmptyState(
    icon: Element,
    title: String,
    #[props(default)] description: String,
    action: Option<Element>,
) -> Element {
    rsx! {
        div {
            style: "
                display: flex;
                flex-direction: column;
                align-items: center;
                justify-content: center;
                gap: 16px;
                text-align: center;
                padding: 40px 20px;
                color: {TEXT_SECONDARY};
            ",
            div {
                style: "
                    width: 48px;
                    height: 48px;
                    color: {BLUE};
                    display: flex;
                    align-items: center;
                    justify-content: center;
                ",
                {icon}
            }
            h3 {
                style: "
                    margin: 0;
                    font-size: 20px;
                    font-weight: 600;
                    color: {TEXT_PRIMARY};
                    letter-spacing: -0.01em;
                    line-height: 1.3;
                ",
                "{title}"
            }
            if !description.is_empty() {
                p {
                    style: "
                        margin: 0;
                        font-size: 14px;
                        line-height: 1.5;
                        color: {TEXT_SECONDARY};
                        max-width: 360px;
                    ",
                    "{description}"
                }
            }
            if let Some(a) = action {
                {a}
            }
        }
    }
}

#[allow(dead_code)] // used via rsx attribute interpolation
const BLUE: &str = Colors::KIMI_BLUE;
#[allow(dead_code)] // used via rsx attribute interpolation
const TEXT_PRIMARY: &str = Colors::TEXT_PRIMARY;
#[allow(dead_code)] // used via rsx attribute interpolation
const TEXT_SECONDARY: &str = Colors::TEXT_SECONDARY;
