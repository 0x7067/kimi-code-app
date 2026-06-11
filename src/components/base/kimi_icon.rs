//! KimiIcon — Brand logo component with blue dot.

use crate::design_tokens::Colors;
use dioxus::prelude::*;

#[component]
pub fn KimiIcon(
    #[props(default = 32)] size: u32,
    #[props(default = "rounded".to_string())] variant: String,
    #[props(default = false)] animate_dot: bool,
) -> Element {
    let dot_size = size / 4;
    let dot_offset = size / 8;
    let font_size = size as f32 * 0.55;
    let k_y_offset = size as f32 / 2.0 + font_size / 3.5;

    let rx = if variant == "round" {
        size / 2
    } else if variant == "rounded" {
        size / 5
    } else {
        0
    };

    rsx! {
        svg {
            width: "{size}px",
            height: "{size}px",
            view_box: "0 0 {size} {size}",
            fill: "none",
            xmlns: "http://www.w3.org/2000/svg",

            if variant != "k-only" {
                rect {
                    x: "0",
                    y: "0",
                    width: "{size}",
                    height: "{size}",
                    rx: "{rx}",
                    fill: Colors::BG_DARK,
                }
            }

            text {
                x: "{size / 2}",
                y: "{k_y_offset as u32}",
                "text-anchor": "middle",
                fill: Colors::TEXT_PRIMARY,
                "font-family": Typography::FONT_UI,
                "font-weight": Typography::WEIGHT_BOLD,
                "font-size": "{font_size as u32}px",
                "K"
            }

            circle {
                cx: "{size - dot_offset - dot_size / 2}",
                cy: "{dot_offset + dot_size / 2}",
                r: "{dot_size / 2}",
                fill: Colors::KIMI_BLUE,
                class: if animate_dot { "kimi-pulse" } else { "" },
            }
        }
    }
}

use crate::design_tokens::Typography;
