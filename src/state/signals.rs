//! Global Dioxus signals holding all UI state.

use super::model::*;
use dioxus::prelude::*;
use std::collections::HashMap;

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
/// Monotonic turn counter (F-013/F-015): each prompt/steer claims an epoch so
/// a superseded turn's completion does not clobber the newer turn's state.
pub static TURN_EPOCH: GlobalSignal<u64> = Signal::global(|| 0);
/// Messages queued while a turn is running (F-014), dispatched FIFO on turn end.
pub static PENDING_QUEUE: GlobalSignal<Vec<String>> = Signal::global(Vec::new);
/// One-shot text handed to the composer (a queued chip clicked for editing).
pub static COMPOSER_PREFILL: GlobalSignal<Option<String>> = Signal::global(|| None);
pub static PERMISSION: GlobalSignal<Option<PermissionRequest>> = Signal::global(|| None);

/// F-003.11: whether the session-creation dialog is open.
pub static SHOW_NEW_SESSION: GlobalSignal<bool> = Signal::global(|| false);
/// F-003.13: whether the manual-compact confirmation dialog is open.
pub static SHOW_COMPACT_CONFIRM: GlobalSignal<bool> = Signal::global(|| false);
/// F-003.10: project roots collapsed in the sidebar tree.
pub static COLLAPSED_PROJECTS: GlobalSignal<std::collections::HashSet<String>> =
    Signal::global(Default::default);
/// F-003.14: sessions with an in-flight turn — sessionId → (turn epoch,
/// last-activity Unix seconds). Maintained by turn start/end in actions.rs.
pub static RUNNING_SESSIONS: GlobalSignal<HashMap<String, (u64, i64)>> = Signal::global(HashMap::new);
/// Soft cross-process conflict guard: a session that looked active in another
/// process when the user clicked it, awaiting "Resume anyway?" confirmation.
pub static RESUME_CONFLICT: GlobalSignal<Option<SessionMeta>> = Signal::global(|| None);

pub static ATTACHMENTS: GlobalSignal<Vec<Attachment>> = Signal::global(Vec::new);
pub static SESSION_SEARCH: GlobalSignal<String> = Signal::global(String::new);
/// In-conversation search (F-002.9): whether the search bar is open and its query.
pub static SEARCH_OPEN: GlobalSignal<bool> = Signal::global(|| false);
pub static CONVO_SEARCH: GlobalSignal<String> = Signal::global(String::new);
/// Context-window usage fraction 0.0–1.0 (F-002.14 / F-003.12), populated from
/// ACP status/usage payloads when the agent reports them.
pub static CONTEXT_USAGE: GlobalSignal<f64> = Signal::global(|| 0.0);
/// F-011: GUI app settings, loaded from the backend store on startup and
/// re-saved on every change (live-apply, F-011.13).
pub static APP_SETTINGS: GlobalSignal<AppSettings> = Signal::global(AppSettings::default);

pub static VIEW: GlobalSignal<View> = Signal::global(|| View::Chat);
pub static SHOW_DIFF: GlobalSignal<bool> = Signal::global(|| false);
pub static DIFF: GlobalSignal<String> = Signal::global(String::new);
pub static ERROR: GlobalSignal<Option<String>> = Signal::global(|| None);

/// Cached thread state per session so switching sessions does not lose scrollback.
pub static SCROLLBACK_CACHE: GlobalSignal<HashMap<String, (Vec<Item>, Vec<PlanEntry>)>> = Signal::global(HashMap::new);
/// Locally-overridden semantic titles for sessions (key = sessionId).
pub static SESSION_TITLES: GlobalSignal<HashMap<String, String>> = Signal::global(HashMap::new);

pub fn reset_thread() {
    ITEMS.write().clear();
    PLAN.write().clear();
    COMMANDS.write().clear();
    CONFIG_OPTIONS.write().clear();
    *PERMISSION.write() = None;
    *RUNNING.write() = false;
    *CONTEXT_USAGE.write() = 0.0;
}

/// Save current session thread state into the scrollback cache.
pub fn cache_current_scrollback() {
    if let Some(sid) = SESSION_ID.read().clone() {
        let items = ITEMS.read().clone();
        let plan = PLAN.read().clone();
        SCROLLBACK_CACHE.write().insert(sid, (items, plan));
    }
}
