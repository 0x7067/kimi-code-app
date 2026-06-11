//! KimiEmptyState — Empty state for lists, search results, and error states.

use crate::design_tokens::Colors;
use dioxus::prelude::*;

#[component]
pub fn KimiEmptyState(
    icon: Element,
    title: String,
    #[props(default = "")] description: String,
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
                color: {text_secondary};
            ",
            div {
                style: "
                    width: 48px;
                    height: 48px;
                    color: {blue};
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
                    color: {text_primary};
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
                        color: {text_secondary};
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

const blue: &str = Colors::KIMI_BLUE;
const text_primary: &str = Colors::TEXT_PRIMARY;
const text_secondary: &str = Colors::TEXT_SECONDARY;
