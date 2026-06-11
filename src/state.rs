//! App-wide state and the ACP event reducer.

use dioxus::prelude::*;
use serde_json::Value;

#[derive(Clone, PartialEq, Debug)]
pub struct ToolCall {
    pub id: String,
    pub title: String,
    pub kind: String,
    pub status: String,
    pub output: String,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Item {
    User(String),
    Agent(String),
    Thought(String),
    Tool(ToolCall),
}

#[derive(Clone, PartialEq, Debug)]
pub struct PlanEntry {
    pub content: String,
    pub priority: String,
    pub status: String,
}

#[derive(Clone, PartialEq, Debug)]
pub struct SelectOption {
    pub value: String,
    pub name: String,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ConfigOption {
    pub id: String,
    pub name: String,
    pub current: String,
    pub options: Vec<SelectOption>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct SessionMeta {
    pub id: String,
    pub cwd: String,
    pub title: String,
    pub updated_at: String,
}

#[derive(Clone, PartialEq, Debug)]
pub struct PermissionRequest {
    pub request_id: u64,
    pub title: String,
    pub detail: String,
    pub options: Vec<(String, String, String)>, // (optionId, name, kind)
}

#[derive(Clone, PartialEq, Debug)]
pub struct SlashCommand {
    pub name: String,
    pub description: String,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum View {
    Chat,
    Settings,
}

pub static CONNECTED: GlobalSignal<bool> = Signal::global(|| false);
pub static AGENT_INFO: GlobalSignal<String> = Signal::global(String::new);
pub static NEEDS_LOGIN: GlobalSignal<bool> = Signal::global(|| false);
pub static LOGIN_LINES: GlobalSignal<Vec<String>> = Signal::global(Vec::new);
pub static LOGIN_RUNNING: GlobalSignal<bool> = Signal::global(|| false);

pub static PROJECT: GlobalSignal<Option<String>> = Signal::global(|| None);
pub static RECENT_PROJECTS: GlobalSignal<Vec<String>> = Signal::global(Vec::new);
pub static SESSIONS: GlobalSignal<Vec<SessionMeta>> = Signal::global(Vec::new);

pub static SESSION_ID: GlobalSignal<Option<String>> = Signal::global(|| None);
pub static ITEMS: GlobalSignal<Vec<Item>> = Signal::global(Vec::new);
pub static PLAN: GlobalSignal<Vec<PlanEntry>> = Signal::global(Vec::new);
pub static CONFIG_OPTIONS: GlobalSignal<Vec<ConfigOption>> = Signal::global(Vec::new);
pub static COMMANDS: GlobalSignal<Vec<SlashCommand>> = Signal::global(Vec::new);
pub static RUNNING: GlobalSignal<bool> = Signal::global(|| false);
pub static PERMISSION: GlobalSignal<Option<PermissionRequest>> = Signal::global(|| None);

#[derive(Clone, PartialEq, Debug)]
pub struct Attachment {
    pub name: String,
    pub mime: String,
    pub data: String,
}

pub static ATTACHMENTS: GlobalSignal<Vec<Attachment>> = Signal::global(Vec::new);
pub static SESSION_SEARCH: GlobalSignal<String> = Signal::global(String::new);
pub static VIEW: GlobalSignal<View> = Signal::global(|| View::Chat);
pub static SHOW_DIFF: GlobalSignal<bool> = Signal::global(|| false);
pub static DIFF: GlobalSignal<String> = Signal::global(String::new);
pub static ERROR: GlobalSignal<Option<String>> = Signal::global(|| None);

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
        "user_message_chunk" => {
            let text = content_text(update.get("content").unwrap_or(&Value::Null));
            let mut items = ITEMS.write();
            if let Some(Item::User(last)) = items.last_mut() {
                last.push_str(&text);
            } else {
                items.push(Item::User(text));
            }
        }
        "agent_message_chunk" => {
            let text = content_text(update.get("content").unwrap_or(&Value::Null));
            let mut items = ITEMS.write();
            if let Some(Item::Agent(last)) = items.last_mut() {
                last.push_str(&text);
            } else {
                items.push(Item::Agent(text));
            }
        }
        "agent_thought_chunk" => {
            let text = content_text(update.get("content").unwrap_or(&Value::Null));
            let mut items = ITEMS.write();
            if let Some(Item::Thought(last)) = items.last_mut() {
                last.push_str(&text);
            } else {
                items.push(Item::Thought(text));
            }
        }
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
                            tc.output = out;
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

pub fn reset_thread() {
    ITEMS.write().clear();
    PLAN.write().clear();
    COMMANDS.write().clear();
    CONFIG_OPTIONS.write().clear();
    *PERMISSION.write() = None;
    *RUNNING.write() = false;
}

pub fn err_msg(e: &Value) -> String {
    e.get("message")
        .and_then(|m| m.as_str())
        .map(|m| m.to_string())
        .unwrap_or_else(|| e.to_string())
}
