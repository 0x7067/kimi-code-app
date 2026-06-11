//! Reducer applying ACP `session/update` notifications to the global signals.

use super::model::*;
use super::signals::*;
use dioxus::prelude::ReadableExt;
use serde_json::Value;

fn s(v: &Value, key: &str) -> String {
    v.get(key).and_then(|x| x.as_str()).unwrap_or_default().to_string()
}

fn content_text(content: &Value) -> String {
    match content {
        Value::String(t) => t.clone(),
        Value::Array(blocks) => blocks.iter().map(content_text).collect::<Vec<_>>().join(""),
        Value::Object(_) => {
            if let Some(t) = content.get("text").and_then(|t| t.as_str()) {
                t.to_string()
            } else if let Some(inner) = content.get("content") {
                content_text(inner)
            } else {
                String::new()
            }
        }
        _ => String::new(),
    }
}

fn push_chunk(update: &Value, make: fn(String) -> Item, append: fn(&mut Item, &str) -> bool) {
    let text = content_text(update.get("content").unwrap_or(&Value::Null));
    let mut items = ITEMS.write();
    if let Some(last) = items.last_mut() {
        if append(last, &text) {
            return;
        }
    }
    items.push(make(text));
}

/// Apply one `session/update` notification to the thread state.
pub fn apply_update(params: &Value) {
    let sid = s(params, "sessionId");
    if let Some(current) = SESSION_ID.read().as_deref() {
        if !sid.is_empty() && sid != current {
            return; // update for another session
        }
    }
    let update = params.get("update").unwrap_or(params);
    let kind = s(update, "sessionUpdate");

    match kind.as_str() {
        "user_message_chunk" => push_chunk(update, Item::User, |item, text| {
            if let Item::User(last) = item {
                last.push_str(text);
                true
            } else {
                false
            }
        }),
        "agent_message_chunk" => push_chunk(update, Item::Agent, |item, text| {
            if let Item::Agent(last) = item {
                last.push_str(text);
                true
            } else {
                false
            }
        }),
        "agent_thought_chunk" => push_chunk(update, Item::Thought, |item, text| {
            if let Item::Thought(last) = item {
                last.push_str(text);
                true
            } else {
                false
            }
        }),
        "tool_call" => {
            let tc = ToolCall {
                id: s(update, "toolCallId"),
                title: s(update, "title"),
                kind: s(update, "kind"),
                status: {
                    let st = s(update, "status");
                    if st.is_empty() { "pending".into() } else { st }
                },
                output: content_text(update.get("content").unwrap_or(&Value::Null)),
            };
            ITEMS.write().push(Item::Tool(tc));
        }
        "tool_call_update" => {
            let id = s(update, "toolCallId");
            let mut items = ITEMS.write();
            for item in items.iter_mut().rev() {
                if let Item::Tool(tc) = item {
                    if tc.id == id {
                        let st = s(update, "status");
                        if !st.is_empty() {
                            tc.status = st;
                        }
                        let title = s(update, "title");
                        if !title.is_empty() {
                            tc.title = title;
                        }
                        let out = content_text(update.get("content").unwrap_or(&Value::Null));
                        if !out.is_empty() {
                            // Handle both full-replacement and incremental-chunk protocols
                            // so CLI output accumulates correctly for real-time rendering.
                            if tc.output.is_empty() {
                                tc.output = out;
                            } else if out.starts_with(&tc.output) {
                                // Full output that grew — take the longer version.
                                tc.output = out;
                            } else if tc.output.ends_with(&out)
                                && out.len() <= tc.output.len()
                            {
                                // Repeated suffix — ignore.
                            } else {
                                // Incremental chunk — append.
                                tc.output.push_str(&out);
                            }
                        }
                        break;
                    }
                }
            }
        }
        "plan" => {
            let entries = update
                .get("entries")
                .and_then(|e| e.as_array())
                .map(|arr| {
                    arr.iter()
                        .map(|e| PlanEntry {
                            content: s(e, "content"),
                            priority: s(e, "priority"),
                            status: s(e, "status"),
                        })
                        .collect()
                })
                .unwrap_or_default();
            *PLAN.write() = entries;
        }
        "available_commands_update" => {
            let cmds = update
                .get("availableCommands")
                .and_then(|c| c.as_array())
                .map(|arr| {
                    arr.iter()
                        .map(|c| SlashCommand {
                            name: s(c, "name"),
                            description: s(c, "description"),
                        })
                        .collect()
                })
                .unwrap_or_default();
            *COMMANDS.write() = cmds;
        }
        "current_mode_update" => {
            let mode = s(update, "currentModeId");
            for opt in CONFIG_OPTIONS.write().iter_mut() {
                if opt.id == "mode" {
                    opt.current = mode.clone();
                }
            }
        }
        "config_option_update" => {
            if let Some(opts) = update.get("configOptions") {
                set_config_options(opts);
            }
        }
        _ => {}
    }
}

pub fn set_config_options(v: &Value) {
    let parsed = v
        .as_array()
        .map(|arr| {
            arr.iter()
                .map(|o| ConfigOption {
                    id: s(o, "id"),
                    name: s(o, "name"),
                    current: o
                        .get("currentValue")
                        .map(|cv| match cv {
                            Value::String(x) => x.clone(),
                            other => other.to_string(),
                        })
                        .unwrap_or_default(),
                    options: o
                        .get("options")
                        .and_then(|x| x.as_array())
                        .map(|arr| {
                            arr.iter()
                                .map(|so| SelectOption { value: s(so, "value"), name: s(so, "name") })
                                .collect()
                        })
                        .unwrap_or_default(),
                })
                .collect()
        })
        .unwrap_or_default();
    *CONFIG_OPTIONS.write() = parsed;
}

pub fn err_msg(e: &Value) -> String {
    e.get("message")
        .and_then(|m| m.as_str())
        .map(|m| m.to_string())
        .unwrap_or_else(|| e.to_string())
}
