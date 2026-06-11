//! Domain types shared across the UI.

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
    /// Subtle marker showing where a turn was cancelled (F-013).
    Cancelled,
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

#[derive(Clone, PartialEq, Debug)]
pub struct Attachment {
    pub name: String,
    pub mime: String,
    pub data: String,
}
