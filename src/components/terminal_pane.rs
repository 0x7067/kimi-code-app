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

fn terminal_output_prompt(output: &str) -> Option<String> {
    let trimmed = output.trim();
    if trimmed.is_empty() {
        return None;
    }
    let char_count = trimmed.chars().count();
    let tail: String = trimmed
        .chars()
        .skip(char_count.saturating_sub(12_000))
        .collect();
    Some(format!(
        "Analyze this terminal output:\n\n```text\n{}\n```",
        tail
    ))
}

fn record_command(history: &mut Vec<String>, command: &str) {
    let command = command.trim();
    if command.is_empty() {
        return;
    }
    if history.last().map(|last| last.as_str()) != Some(command) {
        history.push(command.to_string());
    }
    if history.len() > 100 {
        let drop_count = history.len() - 100;
        history.drain(0..drop_count);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_output_prompt_ignores_empty_output() {
        assert_eq!(terminal_output_prompt(" \n\t"), None);
    }

    #[test]
    fn terminal_output_prompt_wraps_output_for_chat() {
        assert_eq!(
            terminal_output_prompt("cargo test failed").unwrap(),
            "Analyze this terminal output:\n\n```text\ncargo test failed\n```"
        );
    }

    #[test]
    fn terminal_output_prompt_truncates_on_char_boundary() {
        let output = format!("{}Ω", "a".repeat(12_500));
        let prompt = terminal_output_prompt(&output).unwrap();
        assert!(prompt.contains('Ω'));
        assert!(prompt.len() < output.len() + 200);
    }

    #[test]
    fn record_command_skips_empty_and_adjacent_duplicates() {
        let mut history = Vec::new();
        record_command(&mut history, "  ");
        record_command(&mut history, "cargo test");
        record_command(&mut history, "cargo test");
        record_command(&mut history, "cargo check");
        assert_eq!(history, vec!["cargo test", "cargo check"]);
    }

    #[test]
    fn filter_history_returns_recent_matching_commands() {
        let history = vec![
            "cargo test".to_string(),
            "pnpm test".to_string(),
            "cargo check".to_string(),
        ];
        assert_eq!(
            filter_history(&history, "cargo"),
            vec!["cargo check".to_string(), "cargo test".to_string()]
        );
    }
}

fn filter_history(history: &[String], query: &str) -> Vec<String> {
    let query = query.trim().to_lowercase();
    history
        .iter()
        .rev()
        .filter(|cmd| query.is_empty() || cmd.to_lowercase().contains(&query))
        .take(12)
        .cloned()
        .collect()
}

#[component]
pub fn TerminalPane() -> Element {
    let mut output = use_signal(String::new);
    let mut term_id = use_signal(|| None::<u64>);
    let mut input = use_signal(String::new);
    let mut history = use_signal(Vec::<String>::new);
    let mut history_query = use_signal(String::new);

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
            record_command(&mut history.write(), &text);
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
    let history_results = filter_history(&history.read(), &history_query.read());
    let can_send_output = terminal_output_prompt(&output.read()).is_some();

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
                        title: "Send terminal output to chat",
                        disabled: !can_send_output,
                        onclick: move |_| {
                            if let Some(prompt) = terminal_output_prompt(&output.read()) {
                                *COMPOSER_PREFILL.write() = Some(prompt);
                            }
                        },
                        "Send to chat"
                    }
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
            if !history.read().is_empty() {
                div { class: "terminal-history",
                    input {
                        class: "terminal-history-search",
                        r#type: "search",
                        placeholder: "Search command history…",
                        value: "{history_query}",
                        oninput: move |e| history_query.set(e.value()),
                    }
                    div { class: "terminal-history-list",
                        for command in history_results.iter() {
                            {
                                let command = command.clone();
                                rsx! {
                                    button {
                                        key: "{command}",
                                        class: "terminal-history-item",
                                        title: "Use command",
                                        onclick: move |_| input.set(command.clone()),
                                        "{command}"
                                    }
                                }
                            }
                        }
                    }
                }
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
