//! F-006: Minimal embedded browser preview via iframe.

use crate::state::*;
use dioxus::prelude::*;

#[component]
pub fn BrowserPane() -> Element {
    let mut url = use_signal(|| "http://localhost:3000".to_string());
    let mut active = use_signal(|| "http://localhost:3000".to_string());

    rsx! {
        div { class: "browser-pane",
            div { class: "browser-head",
                input {
                    class: "browser-url",
                    value: "{url}",
                    oninput: move |e| url.set(e.value()),
                    onkeydown: move |e| {
                        if e.key() == Key::Enter {
                            active.set(url.read().clone());
                        }
                    },
                }
                button {
                    class: "ghost",
                    onclick: move |_| active.set(url.read().clone()),
                    "Load"
                }
                button {
                    class: "ghost",
                    onclick: move |_| {
                        let current = active.read().clone();
                        active.set(current);
                    },
                    "Refresh"
                }
                button {
                    class: "ghost",
                    onclick: move |_| *SHOW_BROWSER.write() = false,
                    "Close"
                }
            }
            iframe {
                class: "browser-frame",
                src: "{active}",
            }
        }
    }
}
