//! Sidebar (F-003): project picker, project tree with nested sessions
//! (F-003.10), background running-sessions panel (F-003.14), session search,
//! and the new-session dialog trigger (F-003.11).
//!
//! SS-01: Codex-style sidebar layout with Kimi branding.

use crate::actions::{refresh_sessions, request_load_session};
use crate::components::base::KimiIcon;
use crate::components::icons::{IconEdit, IconProps, IconSearch, IconSettings, IconUser};
use crate::conversation::{
    background_sessions, format_updated_at, group_sessions_by_project, now_epoch, relative_label,
};
use crate::ipc::invoke;
use crate::state::*;
use dioxus::prelude::*;
use serde_json::{json, Value};

/// One clickable session row (used by both the flat list and the tree).
#[component]
fn SessionRow(sess: SessionMeta) -> Element {
    let active = SESSION_ID.read().as_deref() == Some(sess.id.as_str());
    let running = RUNNING_SESSIONS.read().contains_key(&sess.id);
    let meta = sess.clone();
    let title = SESSION_TITLES.read().get(&sess.id).cloned().unwrap_or(sess.title.clone());
    rsx! {
        div {
            "data-testid": "session-row",
            "data-session-id": "{sess.id}",
            class: if active { "session-item active" } else { "session-item" },
            onclick: move |_| { spawn(request_load_session(meta.clone())); },
            div { class: "session-item-row",
                div { class: "session-title",
                    if running {
                        span { class: "session-running-dot", title: "Turn in progress" }
                    }
                    "{title}"
                }
                span { class: "session-age",
                    {format_updated_at(&sess.updated_at, now_epoch())}
                }
            }
            div { class: "session-meta",
                {sess.cwd.rsplit('/').next().unwrap_or("").to_string()}
            }
        }
    }
}

// Inline nav icons not yet in the design system.
#[component]
fn IconPlug(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            "stroke-width": "{props.stroke_width}",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            path { d: "M12 22v-5" }
            path { d: "M9 8V2" }
            path { d: "M15 8V2" }
            path { d: "M18 8v5a4 4 0 0 1-4 4h-4a4 4 0 0 1-4-4V8Z" }
        }
    }
}

#[component]
fn IconClock(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            "stroke-width": "{props.stroke_width}",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            circle { cx: "12", cy: "12", r: "10" }
            polyline { points: "12 6 12 12 16 14" }
        }
    }
}

#[component]
pub fn Sidebar() -> Element {
    // F-012: re-list kimi sessions whenever the selected project changes.
    use_effect(move || {
        let _ = PROJECT.read().clone(); // subscribe
        spawn(refresh_sessions());
    });
    let project = PROJECT.read().clone();
    let sessions = SESSIONS.read().clone();
    let query = SESSION_SEARCH.read().to_lowercase();
    let filtered: Vec<SessionMeta> = sessions
        .iter()
        .filter(|sess| {
            query.is_empty()
                || sess.title.to_lowercase().contains(&query)
                || sess.cwd.to_lowercase().contains(&query)
        })
        .cloned()
        .collect();

    // F-003.14: sessions with an in-flight turn other than the current one.
    let now = now_epoch();
    let background: Vec<(String, i64)> = {
        let current = SESSION_ID.read().clone();
        background_sessions(current.as_deref(), &RUNNING_SESSIONS.read())
    };

    // F-003.10: with a project selected, group sessions under project roots.
    let tree = project
        .as_ref()
        .map(|_| group_sessions_by_project(&RECENT_PROJECTS.read(), &filtered));

    // SS-01: collapse long project lists.
    let mut show_all_projects = use_signal(|| false);

    rsx! {
        aside { class: "sidebar", "data-testid": "sidebar",
            // ---------- Brand ----------
            div { class: "sidebar-head",
                span { class: "brand",
                    KimiIcon { size: 22, variant: "rounded", animate_dot: true }
                    span { class: "brand-text", "Kimi Code" }
                }
            }

            // ---------- Project picker ----------
            div { class: "project-picker",
                select {
                    "data-testid": "project-select",
                    value: project.clone().unwrap_or_default(),
                    onchange: move |e| {
                        let v = e.value();
                        *PROJECT.write() = if v.is_empty() { None } else { Some(v) };
                    },
                    option { value: "", "All projects" }
                    for p in RECENT_PROJECTS.read().iter() {
                        option { value: "{p}", selected: Some(p) == project.as_ref(),
                            {p.rsplit('/').next().unwrap_or(p).to_string()}
                        }
                    }
                }
                button {
                    "data-testid": "open-folder-button",
                    class: "ghost icon-btn",
                    title: "Open folder…",
                    onclick: move |_| {
                        spawn(async {
                            if let Ok(Value::String(path)) = invoke("pick_folder", json!({})).await {
                                if !RECENT_PROJECTS.read().contains(&path) {
                                    RECENT_PROJECTS.write().insert(0, path.clone());
                                }
                                *PROJECT.write() = Some(path);
                            }
                        });
                    },
                    "Open…"
                }
            }

            // ---------- Navigation (SS-01) ----------
            nav { class: "sidebar-nav",
                div {
                    "data-testid": "nav-new-chat",
                    class: "sidebar-nav-item",
                    onclick: move |_| { *SHOW_NEW_SESSION.write() = true; },
                    IconEdit { size: 16 }
                    "New chat"
                }
                div {
                    "data-testid": "nav-search",
                    class: "sidebar-nav-item",
                    onclick: move |_| {
                        document::eval("const el = document.getElementById('sidebar-search-input'); if (el) el.focus();");
                    },
                    IconSearch { size: 16 }
                    "Search"
                }
                div {
                    "data-testid": "nav-plugins",
                    class: "sidebar-nav-item",
                    onclick: move |_| { *VIEW.write() = View::Settings; },
                    IconPlug { size: 16 }
                    "Plugins"
                }
                div {
                    "data-testid": "nav-automations",
                    class: "sidebar-nav-item",
                    onclick: move |_| {
                        let current = *SHOW_AUTOMATIONS.read();
                        *SHOW_AUTOMATIONS.write() = !current;
                    },
                    IconClock { size: 16 }
                    "Automations"
                }
            }

            // ---------- Background sessions ----------
            if !background.is_empty() {
                div { class: "bg-sessions",
                    div { class: "sidebar-section-title", "Background sessions" }
                    for (sid, started) in background {
                        {
                            let meta = SESSIONS.read().iter().find(|s| s.id == sid).cloned();
                            let title = SESSION_TITLES
                                .read()
                                .get(&sid)
                                .cloned()
                                .or_else(|| meta.as_ref().map(|m| m.title.clone()))
                                .unwrap_or_else(|| sid.clone());
                            rsx! {
                                div {
                                    key: "{sid}",
                                    class: "session-item bg-session",
                                    onclick: move |_| {
                                        if let Some(m) = meta.clone() {
                                            spawn(request_load_session(m));
                                        }
                                    },
                                    div { class: "session-item-row",
                                        div { class: "session-title",
                                            span { class: "session-running-dot", title: "Turn in progress" }
                                            "{title}"
                                        }
                                        span { class: "session-age", {relative_label(started, now)} }
                                    }
                                    div { class: "session-meta",
                                        {meta.as_ref().map(|m| m.cwd.rsplit('/').next().unwrap_or("").to_string()).unwrap_or_default()}
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // ---------- Search ----------
            div { class: "session-search-wrap",
                IconSearch { size: 14 }
                input {
                    "data-testid": "sidebar-search-input",
                    id: "sidebar-search-input",
                    class: "session-search",
                    r#type: "search",
                    placeholder: "Search sessions…",
                    value: "{SESSION_SEARCH}",
                    oninput: move |e| *SESSION_SEARCH.write() = e.value(),
                }
            }

            // ---------- Projects & Chats ----------
            if let Some((groups, rest)) = tree {
                // F-003.10 project tree: every known project, collapsible,
                // with its sessions nested underneath.
                div { class: "sidebar-section-title", "Projects" }
                {
                    let limit = if *show_all_projects.read() { groups.len() } else { groups.len().min(5) };
                    let truncated = groups.len() > 5;
                    rsx! {
                        for (root, owned) in groups.into_iter().take(limit) {
                            {
                                let collapsed = COLLAPSED_PROJECTS.read().contains(&root);
                                let label = root.rsplit('/').next().unwrap_or(&root).to_string();
                                let selected = project.as_deref() == Some(root.as_str());
                                let root_key = root.clone();
                                let root_sel = root.clone();
                                rsx! {
                                    div { key: "{root}", class: "project-group",
                                        div {
                                            class: if selected { "project-group-head selected" } else { "project-group-head" },
                                            button {
                                                class: "ghost chevron-btn",
                                                onclick: move |_| {
                                                    let mut set = COLLAPSED_PROJECTS.write();
                                                    if !set.remove(&root_key) {
                                                        set.insert(root_key.clone());
                                                    }
                                                },
                                                span { class: if collapsed { "chevron" } else { "chevron open" }, "›" }
                                            }
                                            span {
                                                class: "project-group-name",
                                                onclick: move |_| { *PROJECT.write() = Some(root_sel.clone()); },
                                                "{label}"
                                            }
                                            if !owned.is_empty() {
                                                span { class: "project-group-count", "{owned.len()}" }
                                            }
                                        }
                                        if !collapsed {
                                            div { class: "project-group-sessions",
                                                for sess in owned {
                                                    SessionRow { key: "{sess.id}", sess }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        if truncated && !*show_all_projects.read() {
                            button {
                                class: "ghost",
                                onclick: move |_| { show_all_projects.set(true); },
                                "Show more"
                            }
                        }
                    }
                }
                if !rest.is_empty() {
                    div { class: "sidebar-section-title", "Chats" }
                    div { class: "session-list",
                        for sess in rest {
                            SessionRow { key: "{sess.id}", sess }
                        }
                    }
                }
            } else {
                // No project selected: flat recent-session list under Chats header.
                div { class: "sidebar-section-title", "Chats" }
                div { class: "session-list",
                    for sess in filtered {
                        SessionRow { key: "{sess.id}", sess }
                    }
                }
            }

            // ---------- Footer ----------
            div { class: "sidebar-footer",
                div {
                    "data-testid": "sidebar-settings",
                    class: "sidebar-footer-item",
                    onclick: move |_| {
                        let next = if *VIEW.read() == View::Settings { View::Chat } else { View::Settings };
                        *VIEW.write() = next;
                    },
                    IconSettings { size: 16 }
                    "Settings"
                }
                div { class: "sidebar-footer-item",
                    IconUser { size: 16 }
                    "Guest"
                }
            }
        }
    }
}
