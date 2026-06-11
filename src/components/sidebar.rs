use crate::actions::{load_session, new_session};
use crate::ipc::invoke;
use crate::state::*;
use dioxus::prelude::*;
use serde_json::{json, Value};

#[component]
pub fn Sidebar() -> Element {
    let project = PROJECT.read().clone();
    let sessions = SESSIONS.read().clone();
    let query = SESSION_SEARCH.read().to_lowercase();
    let filtered: Vec<SessionMeta> = sessions
        .iter()
        .filter(|sess| project.as_ref().map_or(true, |p| &sess.cwd == p))
        .filter(|sess| {
            query.is_empty()
                || sess.title.to_lowercase().contains(&query)
                || sess.cwd.to_lowercase().contains(&query)
        })
        .cloned()
        .collect();

    rsx! {
        aside { class: "sidebar",
            div { class: "sidebar-head",
                span { class: "brand",
                    span { class: "brand-mark", "K" }
                    "Kimi Code"
                }
            }
            div { class: "project-picker",
                select {
                    value: project.clone().unwrap_or_default(),
                    onchange: |e| {
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
                    onclick: |_| {
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
                disabled: PROJECT.read().is_none() || !*CONNECTED.read(),
                onclick: |_| { spawn(new_session()); },
                "+ New session"
            }
            div { class: "session-search-wrap",
                input {
                    class: "session-search",
                    r#type: "search",
                    placeholder: "Search sessions…",
                    value: "{SESSION_SEARCH}",
                    oninput: |e| *SESSION_SEARCH.write() = e.value(),
                }
            }
            div { class: "session-list",
                for sess in filtered {
                    {
                        let active = SESSION_ID.read().as_deref() == Some(sess.id.as_str());
                        let meta = sess.clone();
                        rsx! {
                            div {
                                key: "{sess.id}",
                                class: if active { "session-item active" } else { "session-item" },
                                onclick: |_| { spawn(load_session(meta.clone())); },
                                div { class: "session-title", "{sess.title}" }
                                div { class: "session-meta",
                                    {sess.cwd.rsplit('/').next().unwrap_or("").to_string()}
                                    " - "
                                    {sess.updated_at.get(..10).unwrap_or("").to_string()}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
