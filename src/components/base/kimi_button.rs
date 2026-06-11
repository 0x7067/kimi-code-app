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
    let base = "inline-flex items-center justify-center font-medium rounded-lg transition-all duration-150 ease-out focus:outline-none focus:ring-2 focus:ring-[#1E90FF] focus:ring-offset-2 focus:ring-offset-[#141414]";

    let variant_cls = match variant.as_str() {
        "primary" => "bg-[#1E90FF] text-white hover:bg-[#4AA8FF] active:scale-[0.98]",
        "secondary" => "bg-[#262626] text-[#F5F5F5] hover:bg-[#333333] active:scale-[0.98]",
        "ghost" => "bg-transparent text-[#A3A3A3] hover:text-[#F5F5F5]",
        "danger" => "bg-[#EF4444] text-white hover:bg-[#F87171] active:scale-[0.98]",
        _ => "bg-[#1E90FF] text-white hover:bg-[#4AA8FF]",
    };

    let size_cls = match size.as_str() {
        "standard" => "h-8 px-3 text-sm",
        "compact" => "h-7 px-2 text-xs",
        "icon" => "h-8 w-8 p-0",
        _ => "h-8 px-3 text-sm",
    };

    let state_cls = if disabled || loading {
        "opacity-50 cursor-not-allowed"
    } else {
        "cursor-pointer"
    };

    let width_cls = if full_width { "w-full" } else { "" };

    rsx! {
        button {
            class: "{base} {variant_cls} {size_cls} {state_cls} {width_cls}",
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
                    class: "animate-spin -ml-1 mr-2 h-4 w-4 text-current",
                    xmlns: "http://www.w3.org/2000/svg",
                    fill: "none",
                    view_box: "0 0 24 24",
                    circle {
                        class: "opacity-25",
                        cx: "12",
                        cy: "12",
                        r: "10",
                        stroke: "currentColor",
                        "stroke-width": "4",
                    }
                    path {
                        class: "opacity-75",
                        fill: "currentColor",
                        d: "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z",
                    }
                }
            }
            {children}
        }
    }
}
