//! KimiButton — Primary, secondary, ghost, and danger buttons.

use dioxus::prelude::*;

#[component]
pub fn KimiButton(
    children: Element,
    #[props(default = "primary".to_string())] variant: String,
    #[props(default = "standard".to_string())] size: String,
    #[props(default = false)] disabled: bool,
    #[props(default = false)] loading: bool,
    #[props(default = false)] full_width: bool,
    onclick: Option<EventHandler<MouseEvent>>,
) -> Element {
    let variant_cls = match variant.as_str() {
        "secondary" => "secondary",
        "ghost" => "ghost",
        "danger" => "danger",
        _ => "primary",
    };

    let size_cls = match size.as_str() {
        "compact" => "compact",
        "icon" => "icon",
        _ => "standard",
    };

    let width_cls = if full_width { "full-width" } else { "" };

    rsx! {
        button {
            class: "kimi-btn {variant_cls} {size_cls} {width_cls}",
            disabled: disabled || loading,
            onclick: move |e| {
                if let Some(handler) = onclick {
                    if !disabled && !loading {
                        handler.call(e);
                    }
                }
            },
            if loading {
                svg {
                    class: "kimi-spinner",
                    width: "16",
                    height: "16",
                    xmlns: "http://www.w3.org/2000/svg",
                    fill: "none",
                    view_box: "0 0 24 24",
                    circle {
                        opacity: "0.25",
                        cx: "12",
                        cy: "12",
                        r: "10",
                        stroke: "currentColor",
                        "stroke-width": "4",
                    }
                    path {
                        opacity: "0.75",
                        fill: "currentColor",
                        d: "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z",
                    }
                }
            }
            {children}
        }
    }
}
