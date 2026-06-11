//! Domain types shared across the UI.

#[derive(Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub title: String,
    pub kind: String,
    pub status: String,
    pub output: String,
}

#[derive(Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub enum Item {
    User(String),
    Agent(String),
    Thought(String),
    Tool(ToolCall),
    /// Subtle marker showing where a turn was cancelled (F-013).
    Cancelled,
}

#[derive(Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct PlanEntry {
    pub content: String,
    pub priority: String,
    pub status: String,
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

/// F-009.2: a reusable task template with a pre-defined prompt.
#[derive(Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct TaskTemplate {
    pub name: String,
    pub description: String,
    pub prompt: String,
}

/// F-009: a scheduled automation.
#[derive(Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct Automation {
    pub id: String,
    pub name: String,
    pub cron: String,
    pub prompt: String,
    pub cwd: String,
    pub enabled: bool,
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
    /// F-007.1: preferred tech stack (comma-separated list).
    pub tech_stack: String,
    /// F-007.1: coding style notes (freeform text).
    pub coding_style: String,
    /// F-007.1: naming conventions (freeform text).
    pub naming_conventions: String,
    /// F-009.2: reusable task templates.
    pub task_templates: Vec<TaskTemplate>,
    /// F-003.4 / F-011.7: automatically trigger /compact when context usage
    /// exceeds the threshold.
    pub auto_compact: bool,
    /// F-003.4 / F-011.7: context-usage fraction (0.0–1.0) at which auto-compact
    /// fires. Default 0.8 (80%).
    pub auto_compact_threshold: f64,
    /// F-009: scheduled automations.
    pub automations: Vec<Automation>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            kimi_bin_override: None,
            thinking_default: "ask".into(),
            approvals: ApprovalPrefs::default(),
            yolo: false,
            tech_stack: String::new(),
            coding_style: String::new(),
            naming_conventions: String::new(),
            task_templates: default_task_templates(),
            auto_compact: true,
            auto_compact_threshold: 0.8,
            automations: Vec::new(),
        }
    }
}

fn default_task_templates() -> Vec<TaskTemplate> {
    vec![
        TaskTemplate {
            name: "Explain".into(),
            description: "Explain how this code works".into(),
            prompt: "Explain how this code works step by step, including any important design decisions or trade-offs.".into(),
        },
        TaskTemplate {
            name: "Tests".into(),
            description: "Add unit tests".into(),
            prompt: "Write comprehensive unit tests for the current module. Cover edge cases, error paths, and typical usage.".into(),
        },
        TaskTemplate {
            name: "Refactor".into(),
            description: "Improve readability".into(),
            prompt: "Refactor this code to improve readability and maintainability without changing external behavior.".into(),
        },
        TaskTemplate {
            name: "Debug".into(),
            description: "Help debug an issue".into(),
            prompt: "Help me debug this issue. Describe the symptoms, any error messages, and what I've already tried.".into(),
        },
        TaskTemplate {
            name: "Document".into(),
            description: "Add documentation".into(),
            prompt: "Add clear documentation comments to this code. Explain the purpose of each public API, parameters, and return values.".into(),
        },
    ]
}

#[derive(Clone, PartialEq, Debug)]
pub struct Attachment {
    pub name: String,
    pub mime: String,
    pub data: String,
}
