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

/// Per-tool-type default approval action (F-011.5): "ask" or "auto".
#[derive(Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ApprovalPrefs {
    pub shell: String,
    pub file_edit: String,
    pub mcp: String,
    pub git: String,
}

impl Default for ApprovalPrefs {
    fn default() -> Self {
        let ask = || "ask".to_string();
        Self { shell: ask(), file_edit: ask(), mcp: ask(), git: ask() }
    }
}

/// GUI-side app settings (F-011), persisted via the backend app-settings
/// store (`app_settings.json`) so changes apply without restart (F-011.13).
#[derive(Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AppSettings {
    /// F-011.1: manual kimi binary override (None = auto-detect).
    pub kimi_bin_override: Option<String>,
    /// F-011.4: "always" | "never" | "ask" (ask = per-send shortcut decides).
    pub thinking_default: String,
    /// F-011.5: per-tool-type approval defaults.
    pub approvals: ApprovalPrefs,
    /// F-011.6: YOLO — auto-approve everything.
    pub yolo: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            kimi_bin_override: None,
            thinking_default: "ask".into(),
            approvals: ApprovalPrefs::default(),
            yolo: false,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Attachment {
    pub name: String,
    pub mime: String,
    pub data: String,
}
