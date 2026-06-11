use crate::ipc::invoke;
use crate::state::*;
use dioxus::prelude::*;
use serde_json::json;

#[component]
pub fn LoginModal() -> Element {
    let lines = LOGIN_LINES.read().clone();
    let running = *LOGIN_RUNNING.read();
    rsx! {
        div { class: "overlay",
            div { class: "modal",
                h3 { "Sign in to Kimi" }
                p { "Authenticate with your Kimi account using the device-code flow." }
                if !lines.is_empty() {
                    pre { class: "login-output", {lines.join("\n")} }
                }
                div { class: "modal-actions",
                    button {
                        class: "primary",
                        disabled: running,
                        onclick: |_| {
                            LOGIN_LINES.write().clear();
                            *LOGIN_RUNNING.write() = true;
                            spawn(async {
                                let _ = invoke("kimi_login", json!({})).await;
                            });
                        },
                        if running { "Waiting for login…" } else { "Login with Kimi" }
                    }
                    button { class: "ghost", onclick: |_| *NEEDS_LOGIN.write() = false, "Close" }
                }
            }
        }
    }
}
