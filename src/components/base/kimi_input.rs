//! KimiInput — Text input with Kimi styling, focus ring, and optional icons.

use crate::design_tokens::Colors;
use dioxus::prelude::*;

#[component]
pub fn KimiInput(
    #[props(default = "")] value: String,
    #[props(default = "")] placeholder: String,
    #[props(default = false)] disabled: bool,
    #[props(default = false)] error: bool,
    #[props(default = false)] multiline: bool,
    onchange: Option<EventHandler<String>>,
    onsubmit: Option<EventHandler<()>>,
    leading_icon: Option<Element>,
    trailing_icon: Option<Element>,
) -> Element {
    let mut local_value = use_signal(|| value.clone());

    let base = "w-full bg-[#1E1E1E] border rounded-xl text-[#F5F5F5] placeholder-[#737373] transition-all duration-150 ease-out focus:outline-none";

    let state_cls = if error {
        "border-[#EF4444] focus:border-[#EF4444] focus:shadow-[0_0_0_2px_rgba(239,68,68,0.3)]"
    } else if disabled {
        "border-[#2E2E2E] opacity-50 cursor-not-allowed"
    } else {
        "border-[#2E2E2E] focus:border-[#1E90FF] focus:shadow-[0_0_0_2px_rgba(30,144,255,0.3)]"
    };

    let size_cls = if multiline {
        "py-3 px-3 min-h-[80px] resize-y"
    } else {
        "h-11 py-2 px-3"
    };

    let pad_left = if leading_icon.is_some() && !multiline { " pl-10" } else { "" };
    let pad_right = if trailing_icon.is_some() && !multiline { " pr-10" } else { "" };

    rsx! {
        div { class: "relative flex items-center",
            if let Some(icon) = leading_icon {
                div { class: "absolute left-3 text-[#737373] pointer-events-none",
                    {icon}
                }
            }
            if multiline {
                textarea {
                    class: "{base} {state_cls} {size_cls}",
                    value: "{local_value()}",
                    placeholder: "{placeholder}",
                    disabled: disabled,
                    oninput: move |e| {
                        let new = e.value();
                        local_value.set(new.clone());
                        if let Some(h) = onchange { h.call(new); }
                    },
                }
            } else {
                input {
                    class: "{base} {state_cls} {size_cls}{pad_left}{pad_right}",
                    r#type: "text",
                    value: "{local_value()}",
                    placeholder: "{placeholder}",
                    disabled: disabled,
                    oninput: move |e| {
                        let new = e.value();
                        local_value.set(new.clone());
                        if let Some(h) = onchange { h.call(new); }
                    },
                    onkeydown: move |e| {
                        if e.key() == Key::Enter {
                            if let Some(h) = onsubmit { h.call(()); }
                        }
                    },
                }
            }
            if let Some(icon) = trailing_icon {
                div { class: "absolute right-3 text-[#737373] pointer-events-none",
                    {icon}
                }
            }
        }
    }
}
