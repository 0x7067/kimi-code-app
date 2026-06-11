use crate::actions::{connect, refresh_diff};
use crate::state::*;
use dioxus::prelude::*;

#[component]
pub fn Topbar() -> Element {
    let connected = *CONNECTED.read();
    rsx! {
        header { class: "topbar",
            div { class: "topbar-left",
                span { class: if connected { "dot ok" } else { "dot bad" } }
                span { class: "agent-name", "{AGENT_INFO}" }
                if !connected {
                    button { class: "ghost", onclick: |_| { spawn(connect()); }, "Reconnect" }
                }
            }
            div { class: "topbar-right",
                button {
                    class: if *SHOW_DIFF.read() { "ghost active" } else { "ghost" },
                    onclick: |_| {
                        let now = !*SHOW_DIFF.read();
                        *SHOW_DIFF.write() = now;
                        if now { spawn(refresh_diff()); }
                    },
                    "Diff"
                }
                button {
                    class: if *VIEW.read() == View::Settings { "ghost active" } else { "ghost" },
                    onclick: |_| {
                        let v = *VIEW.read();
                        *VIEW.write() = if v == View::Settings { View::Chat } else { View::Settings };
                    },
                    "Settings"
                }
            }
        }
    }
}
