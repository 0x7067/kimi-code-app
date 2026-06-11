//! KimiLoading — Loading indicators (spinner, dots, skeleton).

use crate::design_tokens::Colors;
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub enum LoadingVariant {
    Spinner,
    Dots,
    Skeleton { width: String, height: String },
}

#[component]
pub fn KimiLoading(
    #[props(default = LoadingVariant::Spinner)] variant: LoadingVariant,
    #[props(default = 16)] size: u32,
) -> Element {
    match variant {
        LoadingVariant::Spinner => rsx! {
            svg {
                class: "animate-spin",
                width: "{size}",
                height: "{size}",
                view_box: "0 0 24 24",
                fill: "none",
                xmlns: "http://www.w3.org/2000/svg",
                circle {
                    cx: "12",
                    cy: "12",
                    r: "10",
                    stroke: "#2E2E2E",
                    "stroke-width": "2",
                }
                path {
                    d: "M22 12a10 10 0 0 0-10-10",
                    stroke: Colors::KIMI_BLUE,
                    "stroke-width": "2",
                    "stroke-linecap": "round",
                }
            }
        },
        LoadingVariant::Dots => {
            let dot = dot_size(size);
            rsx! {
            div {
                style: "
                    display: flex;
                    align-items: center;
                    gap: 6px;
                    height: {size}px;
                ",
                for i in 0..3 {
                    div {
                        key: "{i}",
                        style: "
                            width: {dot}px;
                            height: {dot}px;
                            border-radius: 50%;
                            background: {BLUE};
                            animation: kimi-dot-pulse 1.4s ease-in-out infinite;
                            animation-delay: {delay_ms(i)}ms;
                        ",
                    }
                }
            }
        }},
        LoadingVariant::Skeleton { width, height } => rsx! {
            div {
                style: "
                    width: {width};
                    height: {height};
                    border-radius: 6px;
                    background: linear-gradient(90deg, {BG1} 25%, {BG2} 50%, {BG1} 75%);
                    background-size: 200% 100%;
                    animation: kimi-skeleton 1.5s linear infinite;
                ",
            }
        },
    }
}

#[allow(dead_code)] // used via rsx attribute interpolation
const BLUE: &str = Colors::KIMI_BLUE;
#[allow(dead_code)] // used via rsx attribute interpolation
const BG1: &str = Colors::BG_HOVER;
#[allow(dead_code)] // used via rsx attribute interpolation
const BG2: &str = "#333333";

#[allow(dead_code)] // used via rsx attribute interpolation
fn dot_size(size: u32) -> u32 {
    (size as f32 * 0.5) as u32
}

#[allow(dead_code)] // used via rsx attribute interpolation
fn delay_ms(i: usize) -> u32 {
    (i as u32) * 160
}
