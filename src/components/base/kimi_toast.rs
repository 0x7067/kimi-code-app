//! KimiToast — Notification toast for errors, success, and info.

use crate::design_tokens::Colors;
use dioxus::prelude::*;

#[component]
pub fn KimiToast(
    message: String,
    #[props(default = "info".to_string())] variant: String,
    #[props(default = 5000)] duration: u64,
    onclose: EventHandler<()>,
) -> Element {
    let mut visible = use_signal(|| true);

    let border_color = match variant.as_str() {
        "success" => Colors::SUCCESS,
        "warning" => Colors::WARNING,
        "error" => Colors::ERROR,
        _ => Colors::INFO,
    };

    use_effect(move || {
        if duration > 0 {
            let handle = gloo_timers::callback::Timeout::new(duration as u32, move || {
                visible.set(false);
                onclose.call(());
            });
            std::mem::forget(handle);
        }
    });

    if !visible() {
        return rsx! {};
    }

    rsx! {
        div {
            class: "kimi-slide-in",
            style: "
                position: fixed;
                top: 16px;
                right: 16px;
                background: {BG};
                color: {TEXT};
                border-left: 3px solid {border_color};
                border-radius: 12px;
                padding: 12px 16px;
                box-shadow: 0 4px 12px rgba(0,0,0,0.3);
                max-width: 400px;
                z-index: 100;
                display: flex;
                align-items: center;
                gap: 12px;
                font-size: 14px;
                line-height: 1.4;
            ",
            span { style: "flex: 1;", "{message}" }
            button {
                class: "kimi-toast-close",
                onclick: move |_| {
                    visible.set(false);
                    onclose.call(());
                },
                svg {
                    width: "16",
                    height: "16",
                    view_box: "0 0 24 24",
                    fill: "none",
                    stroke: "currentColor",
                    "stroke-width": "2",
                    "stroke-linecap": "round",
                    "stroke-linejoin": "round",
                    path { d: "M18 6 6 18" }
                    path { d: "m6 6 12 12" }
                }
            }
        }
    }
}

#[allow(dead_code)] // used via rsx attribute interpolation
const BG: &str = Colors::BG_SURFACE;
#[allow(dead_code)] // used via rsx attribute interpolation
const TEXT: &str = Colors::TEXT_PRIMARY;
