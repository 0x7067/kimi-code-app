//! F-010: embedded terminal panel.
//!
//! Opens a PTY-backed shell via the backend, streams output through
//! `term:output` events, and forwards user input via `term_write`.
//! A simple text renderer — no full xterm emulation — sufficient for
//! command execution and output review.

use crate::ipc::{invoke, listen_forever};
use crate::state::*;
use dioxus::prelude::*;
use serde_json::{json, Value};

#[component]
pub fn TerminalPane() -> Element {
    let mut output = use_signal(String::new);
    let mut term_id = use_signal(|| None::<u64>);
    let mut input = use_signal(String::new);

    // Open a terminal when the panel becomes visible and none is active.
    use_effect(move || {
        if term_id.read().is_some() {
            return;
        }
        let _cwd = PROJECT.read().clone();
        spawn(async move {
            match invoke("term_open", json!({"cwd": _cwd, "rows": 24, "cols": 120})).await {
                Ok(Value::Object(o)) => {
                    if let Some(id) = o.get("id").and_then(|v| v.as_u64()) {
                        term_id.set(Some(id));
                    }
                }
                Err(e) => {
                    let msg = err_msg(&e);
                    let _ = output.set(format!("\n[terminal error: {msg}]\n"));
                }
                _ => {}
            }
        });
    });

    // Subscribe to PTY output events.
    use_effect(move || {
        let mut output = output;
        listen_forever("term:output", move |payload| {
            if let Some(data) = payload.get("data").and_then(|v| v.as_str()) {
                let mut s = output.write();
                s.push_str(data);
                if s.len() > 256_000 {
                    let split = s.len() - 128_000;
                    *s = s.split_off(split);
                }
            }
        });
    });

    // Subscribe to PTY exit events.
    use_effect(move || {
        let mut output = output;
        let mut term_id = term_id;
        listen_forever("term:exit", move |payload| {
            let id = payload.get("id").and_then(|v| v.as_u64());
            if id == *term_id.read() {
                term_id.set(None);
                output.write().push_str("\n[process exited]\n");
            }
        });
    });

    // Auto-scroll to bottom whenever output changes.
    use_effect(move || {
        let _ = output.read().len();
        document::eval("const el = document.getElementById('terminal-body'); if (el) el.scrollTop = el.scrollHeight;");
    });

    let mut send_input = move || {
        let text = input.read().clone();
        if text.is_empty() {
            return;
        }
        if let Some(id) = *term_id.read() {
            let data = format!("{text}\n");
            input.set(String::new());
            spawn(async move {
                let _ = invoke("term_write", json!({"id": id, "data": data})).await;
            });
        }
    };

    let mut close_terminal = move || {
        if let Some(id) = *term_id.read() {
            spawn(async move {
                let _ = invoke("term_close", json!({"id": id})).await;
            });
        }
        let _ = term_id.set(None);
        let _ = output.set(String::new());
        *TERMINAL_OPEN.write() = false;
    };

    let id_opt = *term_id.read();

    rsx! {
        div { class: "terminal-pane",
            div { class: "terminal-head",
                span { class: "terminal-title", "Terminal" }
                if id_opt.is_some() {
                    span { class: "terminal-status", "●" }
                } else {
                    span { class: "terminal-status dead", "○" }
                }
                div { class: "terminal-actions",
                    button {
                        class: "ghost icon-btn",
                        title: "Clear output",
                        onclick: move |_| output.set(String::new()),
                        "Clear"
                    }
                    button {
                        class: "ghost icon-btn",
                        title: "Close terminal",
                        onclick: move |_| close_terminal(),
                        "Close"
                    }
                }
            }
            div {
                id: "terminal-body",
                class: "terminal-body",
                pre { "{output}" }
            }
            div { class: "terminal-input-line",
                span { class: "terminal-prompt", "$" }
                input {
                    class: "terminal-input",
                    r#type: "text",
                    placeholder: if id_opt.is_some() { "Type a command…" } else { "Terminal closed" },
                    disabled: id_opt.is_none(),
                    value: "{input}",
                    oninput: move |e| input.set(e.value()),
                    onkeydown: move |e| {
                        if e.key() == Key::Enter {
                            send_input();
                        }
                    },
                }
            }
        }
    }
}
