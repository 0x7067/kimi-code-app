//! KimiToggle — iOS-style toggle switch.

use crate::design_tokens::Colors;
use dioxus::prelude::*;

#[component]
pub fn KimiToggle(
    #[props(default = false)] checked: bool,
    #[props(default = false)] disabled: bool,
    onchange: EventHandler<bool>,
) -> Element {
    let mut is_on = use_signal(|| checked);

    let track_bg = if is_on() {
        Colors::KIMI_BLUE
    } else {
        Colors::BG_HOVER
    };

    let thumb_translate = if is_on() { "16px" } else { "0px" };
    let opacity = if disabled { "0.5" } else { "1.0" };
    let cursor = if disabled { "not-allowed" } else { "pointer" };

    rsx! {
        div {
            role: "switch",
            aria_checked: "{is_on()}",
            style: "
                width: 32px;
                height: 16px;
                border-radius: 8px;
                background: {track_bg};
                position: relative;
                cursor: {cursor};
                opacity: {opacity};
                transition: background 150ms ease-out;
                flex-shrink: 0;
            ",
            onclick: move |_| {
                if !disabled {
                    let new = !is_on();
                    is_on.set(new);
                    onchange.call(new);
                }
            },
            div {
                style: "
                    width: 14px;
                    height: 14px;
                    border-radius: 50%;
                    background: #FFFFFF;
                    position: absolute;
                    top: 1px;
                    left: 1px;
                    transform: translateX({thumb_translate});
                    transition: transform 150ms ease-out;
                    box-shadow: 0 1px 3px rgba(0,0,0,0.3);
                ",
            }
        }
    }
}
