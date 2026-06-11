use crate::actions::{cancel_turn, send_prompt, set_config};
use crate::conversation::{filter_mentions, mention_candidates_from_diff, mention_token};
use crate::ipc::invoke;
use crate::state::*;
use dioxus::prelude::*;
use serde_json::{json, Value};

/// Candidate paths for @mentions (F-002.12 scaffold).
///
/// TODO(F-002.12): no backend file-listing command exists yet in
/// `src-tauri/src/commands/`; until one does, fall back to the changed files
/// already known from the diff state.
fn mention_candidates() -> Vec<String> {
    mention_candidates_from_diff(&DIFF.read())
}

#[component]
pub fn Composer() -> Element {
    let mut draft = use_signal(String::new);
    let mut slash_selected = use_signal(|| 0usize);
    let mut mention_selected = use_signal(|| 0usize);
    let running = *RUNNING.read();
    let has_session = SESSION_ID.read().is_some();
    let show_slash = draft.read().starts_with('/') && !draft.read().contains(' ');
    let filter = draft.read().trim_start_matches('/').to_string();

    let filtered: Vec<SlashCommand> = COMMANDS
        .read()
        .iter()
        .filter(|c| c.name.starts_with(&filter))
        .cloned()
        .collect();

    // F-002.12: @mention dropdown over known project file paths.
    let mention = if show_slash { None } else { mention_token(&draft.read()) };
    let mentions: Vec<String> = mention
        .as_ref()
        .map(|(_, q)| filter_mentions(&mention_candidates(), q))
        .unwrap_or_default();
    let show_mentions = mention.is_some() && !mentions.is_empty();

    let mut submit = move |thinking: bool| {
        let text = draft.read().trim().to_string();
        if text.is_empty() || *RUNNING.read() || SESSION_ID.read().is_none() {
            return;
        }
        draft.set(String::new());
        spawn(send_prompt(text, thinking));
    };

    let mut insert_mention = move |path: &str| {
        let current = draft.read().clone();
        if let Some((at, _)) = mention_token(&current) {
            draft.set(format!("{}@{path} ", &current[..at]));
        }
    };

    rsx! {
        div { class: "composer",
            if show_slash && !filtered.is_empty() {
                div { class: "slash-menu",
                    for (i, cmd) in filtered.iter().enumerate() {
                        {
                            let name = cmd.name.clone();
                            let selected = i == *slash_selected.read();
                            rsx! {
                                div {
                                    key: "{cmd.name}",
                                    class: if selected { "slash-item selected" } else { "slash-item" },
                                    onclick: move |_| draft.set(format!("/{name} ")),
                                    span { class: "slash-name", "/{cmd.name}" }
                                    span { class: "slash-desc", "{cmd.description}" }
                                }
                            }
                        }
                    }
                }
            }
            if show_mentions {
                div { class: "slash-menu mention-menu",
                    for (i, path) in mentions.iter().enumerate() {
                        {
                            let path = path.clone();
                            let selected = i == *mention_selected.read();
                            rsx! {
                                div {
                                    key: "{path}",
                                    class: if selected { "slash-item selected" } else { "slash-item" },
                                    onclick: move |_| insert_mention(&path),
                                    span { class: "slash-name", "@{path}" }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "composer-box",
                if !ATTACHMENTS.read().is_empty() {
                    div { class: "attachments",
                        for (i, a) in ATTACHMENTS.read().iter().enumerate() {
                            span { key: "{i}", class: "attachment-chip",
                                "{a.name}"
                                button {
                                    class: "chip-x",
                                    onclick: move |_| { ATTACHMENTS.write().remove(i); },
                                    "Remove"
                                }
                            }
                        }
                    }
                }
                textarea {
                    placeholder: if has_session { "Message Kimi…  ( / for commands, @ for files, ⌘⏎ to send)" } else { "Start a session first" },
                    value: "{draft}",
                    disabled: !has_session,
                    oninput: move |e| {
                        draft.set(e.value());
                        slash_selected.set(0);
                        mention_selected.set(0);
                    },
                    onkeydown: move |e| {
                        if show_slash {
                            match e.key() {
                                Key::ArrowUp => {
                                    e.prevent_default();
                                    if !filtered.is_empty() {
                                        let new_sel = if *slash_selected.read() == 0 {
                                            filtered.len() - 1
                                        } else {
                                            *slash_selected.read() - 1
                                        };
                                        slash_selected.set(new_sel);
                                    }
                                }
                                Key::ArrowDown => {
                                    e.prevent_default();
                                    if !filtered.is_empty() {
                                        let new_sel = (*slash_selected.read() + 1) % filtered.len();
                                        slash_selected.set(new_sel);
                                    }
                                }
                                Key::Enter => {
                                    if !e.modifiers().shift() {
                                        e.prevent_default();
                                        if let Some(cmd) = filtered.get(*slash_selected.read()) {
                                            draft.set(format!("/{cmd} ", cmd = cmd.name));
                                        }
                                    }
                                }
                                Key::Escape => {
                                    // just let the menu close naturally by clearing draft prefix
                                    // or do nothing; user can type space/backspace
                                }
                                _ => {}
                            }
                        } else if show_mentions {
                            match e.key() {
                                Key::ArrowUp => {
                                    e.prevent_default();
                                    let sel = *mention_selected.read();
                                    mention_selected.set(if sel == 0 { mentions.len() - 1 } else { sel - 1 });
                                }
                                Key::ArrowDown => {
                                    e.prevent_default();
                                    let sel = *mention_selected.read();
                                    mention_selected.set((sel + 1) % mentions.len());
                                }
                                Key::Enter | Key::Tab => {
                                    e.prevent_default();
                                    if let Some(path) = mentions.get(*mention_selected.read()).cloned() {
                                        insert_mention(&path);
                                    }
                                }
                                _ => {}
                            }
                        } else {
                            // F-002.13 keyboard shortcuts.
                            match e.key() {
                                Key::Enter => {
                                    let m = e.modifiers();
                                    if m.meta() && m.shift() {
                                        e.prevent_default();
                                        submit(true); // send with thinking
                                    } else if m.meta() || !m.shift() {
                                        e.prevent_default();
                                        submit(false);
                                    }
                                }
                                Key::Escape => {
                                    e.prevent_default();
                                    if *RUNNING.read() {
                                        spawn(cancel_turn());
                                    } else if *SEARCH_OPEN.read() {
                                        *SEARCH_OPEN.write() = false;
                                        CONVO_SEARCH.write().clear();
                                    } else {
                                        draft.set(String::new());
                                    }
                                }
                                Key::Character(c) if c == "f" && e.modifiers().meta() => {
                                    // F-002.9: Cmd+F toggles in-conversation search.
                                    e.prevent_default();
                                    let open = !*SEARCH_OPEN.read();
                                    *SEARCH_OPEN.write() = open;
                                    if !open {
                                        CONVO_SEARCH.write().clear();
                                    }
                                }
                                _ => {}
                            }
                        }
                    },
                }
                div { class: "composer-controls",
                    button {
                        class: "ghost",
                        title: "Attach image",
                        disabled: !has_session,
                        onclick: move |_| {
                            spawn(async {
                                if let Ok(Value::Object(img)) = invoke("pick_image", json!({})).await {
                                    ATTACHMENTS.write().push(Attachment {
                                        name: img.get("name").and_then(|v| v.as_str()).unwrap_or("image").into(),
                                        mime: img.get("mimeType").and_then(|v| v.as_str()).unwrap_or("image/png").into(),
                                        data: img.get("data").and_then(|v| v.as_str()).unwrap_or("").into(),
                                    });
                                }
                            });
                        },
                        "Attach"
                    }
                    for opt in CONFIG_OPTIONS.read().iter() {
                        {
                            let id = opt.id.clone();
                            rsx! {
                                select {
                                    key: "{opt.id}",
                                    class: "cfg-select",
                                    title: "{opt.name}",
                                    value: "{opt.current}",
                                    onchange: move |e| { spawn(set_config(id.clone(), e.value())); },
                                    for so in opt.options.iter() {
                                        option { value: "{so.value}", selected: so.value == opt.current, "{so.name}" }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "spacer" }
                    if running {
                        button { class: "danger", onclick: move |_| { spawn(cancel_turn()); }, "Stop" }
                    } else {
                        button {
                            class: "primary",
                            title: "Send (⌘⏎) · Send with thinking (⌘⇧⏎)",
                            disabled: !has_session,
                            onclick: move |_| submit(false),
                            "Send"
                        }
                    }
                }
            }
        }
    }
}
