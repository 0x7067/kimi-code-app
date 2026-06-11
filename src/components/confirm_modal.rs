//! ConfirmModal — generic confirmation dialog for destructive actions.
//!
//! Driven by the global `CONFIRM` signal. Use `request_confirm(...)` to open it.
//! Supports Escape to cancel and backdrop click to dismiss.

use crate::state::*;
use dioxus::prelude::*;

#[component]
pub fn ConfirmModal() -> Element {
    let Some(req) = CONFIRM.read().clone() else { return rsx! {} };

    let confirm_class = if req.danger { "danger" } else { "primary" };

    let on_confirm = req.on_confirm.clone();
    let accept = move |_| {
        (on_confirm)();
        *CONFIRM.write() = None;
    };

    rsx! {
        div {
            class: "overlay",
            // Backdrop click cancels.
            onclick: move |_| *CONFIRM.write() = None,
            div {
                class: "modal confirm-modal",
                // Clicks inside the dialog must not bubble to the backdrop.
                onclick: move |e| e.stop_propagation(),
                // Escape cancels; Enter confirms.
                onkeydown: move |e| {
                    match e.key() {
                        Key::Escape => *CONFIRM.write() = None,
                        _ => {}
                    }
                },
                h3 { "{req.title}" }
                if !req.body.is_empty() {
                    p { class: "confirm-body", "{req.body}" }
                }
                div { class: "modal-actions",
                    button {
                        class: "ghost",
                        onclick: move |_| *CONFIRM.write() = None,
                        "Cancel"
                    }
                    button {
                        class: "{confirm_class}",
                        autofocus: true,
                        onclick: accept,
                        "{req.confirm_label}"
                    }
                }
            }
        }
    }
}
