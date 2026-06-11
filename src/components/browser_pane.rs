//! F-006: Browser preview pane with device toggles, live reload, and URL sharing.

use crate::ipc::invoke;
use crate::state::*;
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
enum Device {
    Mobile,
    Tablet,
    Desktop,
}

impl Device {
    fn width(&self) -> &str {
        match self {
            Device::Mobile => "375px",
            Device::Tablet => "768px",
            Device::Desktop => "100%",
        }
    }
    fn label(&self) -> &str {
        match self {
            Device::Mobile => "Mobile",
            Device::Tablet => "Tablet",
            Device::Desktop => "Desktop",
        }
    }
}

fn cache_busted_url(url: &str, reload_count: usize) -> String {
    let separator = if url.contains('?') { '&' } else { '?' };
    format!("{url}{separator}__kimi_reload={reload_count}")
}

#[component]
pub fn BrowserPane() -> Element {
    let mut url = use_signal(|| "http://localhost:3000".to_string());
    let mut active = use_signal(|| "http://localhost:3000".to_string());
    let mut reload_count = use_signal(|| 0_usize);
    let mut device = use_signal(|| Device::Desktop);

    use_effect(move || {
        // F-006.5: start live-reload watcher when pane opens.
        if let Some(cwd) = PROJECT.read().clone() {
            spawn(async move {
                let _ = invoke("start_browser_watcher", serde_json::json!({"cwd": cwd})).await;
            });
        }
        // Listen for reload events from the backend.
        crate::ipc::listen_forever("browser:reload", move |_| {
            let next = *reload_count.read() + 1;
            reload_count.set(next);
        });
    });

    let send_url_to_chat = move || {
        let u = active.read().clone();
        if !u.is_empty() {
            *COMPOSER_PREFILL.write() = Some(format!("Check out this page: {u}"));
        }
    };

    let dev = *device.read();

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
                        let next = *reload_count.read() + 1;
                        reload_count.set(next);
                    },
                    "Refresh"
                }
                div { class: "browser-devices",
                    for d in [Device::Mobile, Device::Tablet, Device::Desktop] {
                        button {
                            key: "{d.label()}",
                            class: if dev == d { "ghost active" } else { "ghost" },
                            onclick: move |_| device.set(d),
                            "{d.label()}"
                        }
                    }
                }
                button {
                    class: "ghost",
                    onclick: move |_| send_url_to_chat(),
                    "Share"
                }
                button {
                    class: "ghost",
                    onclick: move |_| {
                        spawn(async {
                            let _ = invoke("stop_browser_watcher", serde_json::json!({})).await;
                        });
                        *SHOW_BROWSER.write() = false;
                    },
                    "Close"
                }
            }
            div { class: "browser-frame-wrap",
                iframe {
                    class: "browser-frame",
                    style: "width: {dev.width()};",
                    src: "{cache_busted_url(&active.read(), *reload_count.read())}",
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_busted_url_adds_reload_parameter() {
        assert_eq!(
            cache_busted_url("http://localhost:3000", 7),
            "http://localhost:3000?__kimi_reload=7"
        );
    }

    #[test]
    fn cache_busted_url_preserves_existing_query() {
        assert_eq!(
            cache_busted_url("http://localhost:3000/?a=1", 8),
            "http://localhost:3000/?a=1&__kimi_reload=8"
        );
    }
}
