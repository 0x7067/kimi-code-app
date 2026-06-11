//! Global Dioxus signals holding all UI state.

use super::model::*;
use dioxus::prelude::*;

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

pub static ATTACHMENTS: GlobalSignal<Vec<Attachment>> = Signal::global(Vec::new);
pub static SESSION_SEARCH: GlobalSignal<String> = Signal::global(String::new);
pub static VIEW: GlobalSignal<View> = Signal::global(|| View::Chat);
pub static SHOW_DIFF: GlobalSignal<bool> = Signal::global(|| false);
pub static DIFF: GlobalSignal<String> = Signal::global(String::new);
pub static ERROR: GlobalSignal<Option<String>> = Signal::global(|| None);

pub fn reset_thread() {
    ITEMS.write().clear();
    PLAN.write().clear();
    COMMANDS.write().clear();
    CONFIG_OPTIONS.write().clear();
    *PERMISSION.write() = None;
    *RUNNING.write() = false;
}
