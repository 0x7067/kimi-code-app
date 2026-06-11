//! Sidebar (F-003): project picker, project tree with nested sessions
//! (F-003.10), background running-sessions panel (F-003.14), session search,
//! and the new-session dialog trigger (F-003.11).

use crate::actions::{refresh_sessions, request_load_session};
use crate::components::base::KimiIcon;
use crate::components::icons::IconSearch;
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
            class: if active { "session-item active" } else { "session-item" },
            onclick: move |_| { spawn(request_load_session(meta.clone())); },
            div { class: "session-title",
                if running {
                    span { class: "session-running-dot", title: "Turn in progress" }
                }
                "{title}"
            }
            div { class: "session-meta",
                {sess.cwd.rsplit('/').next().unwrap_or("").to_string()}
                " - "
                {format_updated_at(&sess.updated_at, now_epoch())}
            }
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

    rsx! {
        aside { class: "sidebar",
            div { class: "sidebar-head",
                span { class: "brand",
                    KimiIcon { size: 22, variant: "rounded", animate_dot: true }
                    span { class: "brand-text", "Kimi Code" }
                }
            }
            div { class: "project-picker",
                select {
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
            button {
                class: "primary new-session",
                disabled: !*CONNECTED.read(),
                // F-003.11: opens the session-creation dialog instead of
                // creating a bare session immediately.
                onclick: move |_| { *SHOW_NEW_SESSION.write() = true; },
                "+ New session"
            }
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
                                    div { class: "session-title",
                                        span { class: "session-running-dot", title: "Turn in progress" }
                                        "{title}"
                                    }
                                    div { class: "session-meta", {relative_label(started, now)} }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "session-search-wrap",
                IconSearch { size: 14 }
                input {
                    class: "session-search",
                    r#type: "search",
                    placeholder: "Search sessions…",
                    value: "{SESSION_SEARCH}",
                    oninput: move |e| *SESSION_SEARCH.write() = e.value(),
                }
            }
            div { class: "session-list",
                if let Some((groups, rest)) = tree {
                    // F-003.10 project tree: every known project, collapsible,
                    // with its sessions nested underneath.
                    for (root, owned) in groups {
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
                    for sess in rest {
                        SessionRow { key: "{sess.id}", sess }
                    }
                } else {
                    // No project selected: flat recent-session list.
                    for sess in filtered {
                        SessionRow { key: "{sess.id}", sess }
                    }
                }
            }
        }
    }
}
