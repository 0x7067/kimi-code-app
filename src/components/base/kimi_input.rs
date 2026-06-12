//! KimiInput — Text input with Kimi styling, focus ring, and optional icons.

use dioxus::prelude::*;

#[component]
pub fn KimiInput(
    #[props(default)] value: String,
    #[props(default)] placeholder: String,
    #[props(default = false)] disabled: bool,
    #[props(default = false)] error: bool,
    #[props(default = false)] multiline: bool,
    onchange: Option<EventHandler<String>>,
    onsubmit: Option<EventHandler<()>>,
    leading_icon: Option<Element>,
    trailing_icon: Option<Element>,
) -> Element {
    let mut local_value = use_signal(|| value.clone());

    // Keep the local draft in sync when the parent updates `value` (e.g. a
    // programmatic clear or reset). Without this, the signal captures the
    // initial prop once and ignores later parent changes.
    use_effect(use_reactive((&value,), move |(value,)| {
        if local_value.peek().as_str() != value.as_str() {
            local_value.set(value);
        }
    }));

    let error_cls = if error { "error" } else { "" };
    let leading_cls = if leading_icon.is_some() && !multiline { "has-leading" } else { "" };
    let trailing_cls = if trailing_icon.is_some() && !multiline { "has-trailing" } else { "" };

    rsx! {
        div {
            class: "kimi-input-wrap",
            if let Some(icon) = leading_icon {
                div {
                    class: "kimi-input-icon leading",
                    {icon}
                }
            }
            if multiline {
                textarea {
                    class: "kimi-input multi {error_cls}",
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
                    class: "kimi-input single {error_cls} {leading_cls} {trailing_cls}",
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
                div {
                    class: "kimi-input-icon trailing",
                    {icon}
                }
            }
        }
    }
}
