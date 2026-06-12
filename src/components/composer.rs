use crate::actions::{cancel_turn, enqueue_prompt, save_app_settings, send_prompt, steer_prompt};
use crate::components::base::{KimiDropdown, KimiDropdownItem};
use crate::components::icons::{IconFolder, IconListPlus, IconPlus, IconSquare};
use crate::conversation::{filter_mentions, mention_candidates_from_diff, mention_token};
use crate::ipc::invoke;
use crate::state::*;
use dioxus::prelude::*;
use gloo_timers::callback::Timeout;
use serde_json::{json, Value};

/// Candidate paths for @mentions (F-002.12).
/// Prefers the cached backend file list; falls back to diff-changed files
/// when the backend listing is empty.
fn mention_candidates() -> Vec<String> {
    let backend = PROJECT_FILES.read().clone();
    if !backend.is_empty() {
        backend
    } else {
        mention_candidates_from_diff(&DIFF.read())
    }
}

fn approval_menu_label(yolo: bool) -> &'static str {
    if yolo {
        "Approvals: auto"
    } else {
        "Approvals: ask"
    }
}

#[component]
pub fn Composer() -> Element {
    let mut draft = use_signal(String::new);
    let mut slash_selected = use_signal(|| 0usize);
    let mut mention_selected = use_signal(|| 0usize);
    let mut send_feedback = use_signal(|| false);
    let running = *RUNNING.read();
    let has_session = SESSION_ID.read().is_some();
    let editing = COMPOSER_EDIT_INDEX.read().is_some();
    let observing = *OBSERVE_MODE.read();
    let yolo = APP_SETTINGS.read().yolo;
    let show_slash = draft.read().starts_with('/') && !draft.read().contains(' ');
    let filter = draft.read().trim_start_matches('/').to_string();

    let filtered: Vec<SlashCommand> = COMMANDS
        .read()
        .iter()
        .filter(|c| c.name.starts_with(&filter))
        .cloned()
        .collect();

    // F-002.12: @mention dropdown over known project file paths.
    let mention = if show_slash {
        None
    } else {
        mention_token(&draft.read())
    };
    let mentions: Vec<String> = mention
        .as_ref()
        .map(|(_, q)| filter_mentions(&mention_candidates(), q))
        .unwrap_or_default();
    let show_mentions = mention.is_some() && !mentions.is_empty();

    // F-014: a queued chip clicked for editing lands here.
    use_effect(move || {
        let pending = COMPOSER_PREFILL.read().clone(); // subscribe
        if let Some(text) = pending {
            *COMPOSER_PREFILL.write() = None;
            draft.set(text);
        }
    });

    // Lightweight presence signal for the chat header. Keep it derived from the
    // draft instead of making prompt submission depend on global state.
    use_effect(move || {
        *COMPOSER_HAS_DRAFT.write() = has_session && !observing && !draft.read().trim().is_empty();
    });

    // F-002.12: warm the project file cache for @mentions.
    use_effect(move || {
        let _ = PROJECT.read().clone(); // re-run when project changes
        spawn(async {
            crate::actions::refresh_project_files().await;
        });
    });

    // Send (idle) or steer (running, F-015): steering cancels the active
    // turn and immediately sends the new message in its place.
    let mut submit = move |explicit_thinking: bool| {
        let text = draft.read().trim().to_string();
        if text.is_empty() || SESSION_ID.read().is_none() {
            return;
        }
        // F-011.4: the thinking-mode default makes plain ⏎ send with the
        // thinking flag when set to "always"; ⌘⇧⏎ stays an explicit override.
        let thinking =
            crate::conversation::effective_thinking(&APP_SETTINGS.read().thinking_default, explicit_thinking);
        draft.set(String::new());
        send_feedback.set(true);
        let handle = Timeout::new(220, move || {
            send_feedback.set(false);
        });
        handle.forget();
        // F-002.7: if editing a previous message, truncate the thread and
        // replace it before sending.
        if let Some(idx) = *COMPOSER_EDIT_INDEX.read() {
            *COMPOSER_EDIT_INDEX.write() = None;
            let items = ITEMS.read().clone();
            let truncated = crate::conversation::edit_and_resend(&items, idx, &text);
            *ITEMS.write() = truncated;
        }
        if *RUNNING.read() {
            spawn(steer_prompt(text));
        } else {
            spawn(send_prompt(text, thinking));
        }
    };

    // F-014: park the draft in the pending queue (sends after this turn).
    let mut enqueue = move || {
        let text = draft.read().trim().to_string();
        if text.is_empty() || SESSION_ID.read().is_none() {
            return;
        }
        draft.set(String::new());
        if *RUNNING.read() {
            enqueue_prompt(&text);
        } else {
            spawn(send_prompt(text, false));
        }
    };

    let mut insert_mention = move |path: &str| {
        let current = draft.read().clone();
        if let Some((at, _)) = mention_token(&current) {
            draft.set(format!("{}@{path} ", &current[..at]));
        }
    };

    let project_label = PROJECT.read().clone().unwrap_or_else(|| "No project".to_string());

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
                    placeholder: if !has_session {
                        "Start a session first"
                    } else if observing {
                        "Observing — this session is active in another process"
                    } else if editing {
                        "Editing message… (⌘⏎ to resend, Esc to cancel)"
                    } else if running {
                        "Send to steer…  (⏎ interrupts and sends, ⌥⏎ queues)"
                    } else {
                        "Message Kimi…  ( / for commands, @ for files, ⌘⏎ to send)"
                    },
                    value: "{draft}",
                    disabled: !has_session || observing,
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
                                    if m.alt() {
                                        // F-014: ⌥⏎ queues instead of steering/sending.
                                        e.prevent_default();
                                        enqueue();
                                    } else if m.meta() && m.shift() {
                                        e.prevent_default();
                                        submit(true); // send with thinking
                                    } else if m.meta() || !m.shift() {
                                        e.prevent_default();
                                        submit(false); // send, or steer while running
                                    }
                                }
                                Key::Escape => {
                                    e.prevent_default();
                                    if *RUNNING.read() {
                                        spawn(cancel_turn());
                                    } else if *SEARCH_OPEN.read() {
                                        *SEARCH_OPEN.write() = false;
                                        CONVO_SEARCH.write().clear();
                                    } else if COMPOSER_EDIT_INDEX.read().is_some() {
                                        *COMPOSER_EDIT_INDEX.write() = None;
                                        draft.set(String::new());
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

                // SS-03: Codex-style toolbar (inside composer-box, below textarea).
                div { class: "composer-toolbar",
                    div { class: "composer-toolbar-left",
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
                            IconPlus { size: 16 }
                        }
                        if has_session {
                            KimiDropdown {
                                trigger: rsx! {
                                    button { class: "ghost", "Templates" }
                                },
                                for tmpl in APP_SETTINGS.read().task_templates.iter() {
                                    {
                                        let prompt = tmpl.prompt.clone();
                                        rsx! {
                                            KimiDropdownItem {
                                                onclick: move |_| draft.set(prompt.clone()),
                                                "{tmpl.name} — {tmpl.description}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        KimiDropdown {
                            trigger: rsx! {
                                button { class: "ghost",
                                    svg {
                                        width: "14",
                                        height: "14",
                                        view_box: "0 0 24 24",
                                        fill: "none",
                                        stroke: "currentColor",
                                        "stroke-width": "2",
                                        "stroke-linecap": "round",
                                        "stroke-linejoin": "round",
                                        path { d: "M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" }
                                    }
                                    {approval_menu_label(yolo)}
                                }
                            },
                            KimiDropdownItem {
                                onclick: move |_| {
                                    APP_SETTINGS.write().yolo = true;
                                    spawn(async { save_app_settings().await });
                                },
                                "Always approve"
                            }
                            KimiDropdownItem {
                                onclick: move |_| {
                                    APP_SETTINGS.write().yolo = false;
                                    spawn(async { save_app_settings().await });
                                },
                                "Ask every time"
                            }
                        }
                        if !project_label.is_empty() && project_label != "No project" {
                            span {
                                class: "composer-context-item",
                                style: "cursor: default;",
                                IconFolder { size: 13, color: "currentColor" }
                                "{project_label}"
                            }
                        }
                    }
                    div { class: "composer-toolbar-right",
                        if observing {
                            button {
                                class: "primary",
                                title: "Take control of this session",
                                onclick: move |_| {
                                    if let Some(id) = SESSION_ID.read().clone() {
                                        if let Some(meta) = SESSIONS.read().iter().find(|s| s.id == id).cloned() {
                                            *RESUME_CONFLICT.write() = Some(meta);
                                        }
                                    }
                                },
                                "Take Control"
                            }
                        } else if running {
                            button {
                                class: "ghost queue-btn",
                                title: "Queue message — sends after the current turn (⌥⏎)",
                                onclick: move |_| enqueue(),
                                IconListPlus { size: 14 }
                                "Queue"
                            }
                            button {
                                class: "danger stop-btn",
                                title: "Stop the current turn (Esc) — typing ⏎ steers instead",
                                onclick: move |_| { spawn(cancel_turn()); },
                                IconSquare { size: 12, color: "currentColor" }
                                "Stop"
                            }
                        } else {
                            {
                                let sent = *send_feedback.read();
                                rsx! {
                                    button {
                                        class: if sent { "composer-send sent" } else { "composer-send" },
                                        title: "Send (⌘⏎) · Send with thinking (⌘⇧⏎)",
                                        disabled: !has_session,
                                        onclick: move |_| submit(false),
                                        if sent {
                                            svg {
                                                width: "16",
                                                height: "16",
                                                view_box: "0 0 24 24",
                                                fill: "none",
                                                stroke: "currentColor",
                                                "stroke-width": "2.5",
                                                "stroke-linecap": "round",
                                                "stroke-linejoin": "round",
                                                path { d: "M20 6 9 17l-5-5" }
                                            }
                                        } else {
                                            svg {
                                                width: "16",
                                                height: "16",
                                                view_box: "0 0 24 24",
                                                fill: "none",
                                                stroke: "currentColor",
                                                "stroke-width": "2.5",
                                                "stroke-linecap": "round",
                                                "stroke-linejoin": "round",
                                                path { d: "M12 19V5" }
                                                path { d: "M5 12l7-7 7 7" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// F-014: messages queued while a turn runs, shown between thread and
/// composer. Click a chip to edit it back into the composer; ✕ removes it.
#[component]
pub fn PendingQueue() -> Element {
    let queue = PENDING_QUEUE.read().clone();
    if queue.is_empty() {
        return rsx! {};
    }
    rsx! {
        div { class: "pending-queue",
            span { class: "pending-label", "Queued ({queue.len()})" }
            for (i, text) in queue.iter().enumerate() {
                {
                    let text = text.clone();
                    let label = text.clone();
                    rsx! {
                        div {
                            key: "{i}-{text}",
                            class: "pending-chip",
                            title: "Click to edit in the composer",
                            onclick: move |_| {
                                if crate::conversation::queue_remove(&mut PENDING_QUEUE.write(), i).is_some() {
                                    *COMPOSER_PREFILL.write() = Some(text.clone());
                                }
                            },
                            span { class: "pending-chip-text", "{label}" }
                            button {
                                class: "chip-x",
                                title: "Remove from queue",
                                onclick: move |e| {
                                    e.stop_propagation();
                                    crate::conversation::queue_remove(&mut PENDING_QUEUE.write(), i);
                                },
                                "✕"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approval_menu_label_reflects_yolo_state() {
        assert_eq!(approval_menu_label(true), "Approvals: auto");
        assert_eq!(approval_menu_label(false), "Approvals: ask");
    }
}
