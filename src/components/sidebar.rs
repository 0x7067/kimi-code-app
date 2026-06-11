use crate::actions::{load_session, new_session, refresh_sessions};
use crate::components::base::KimiIcon;
use crate::components::icons::IconSearch;
use crate::conversation::format_updated_at;
use crate::ipc::invoke;
use crate::state::*;
use dioxus::prelude::*;
use serde_json::{json, Value};

/// Current Unix time in seconds (webview clock in wasm; system clock in tests).
fn now_epoch() -> i64 {
    #[cfg(target_arch = "wasm32")]
    {
        (js_sys::Date::now() / 1000.0) as i64
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0)
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
                disabled: PROJECT.read().is_none() || !*CONNECTED.read(),
                onclick: move |_| { spawn(new_session()); },
                "+ New session"
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
                for sess in filtered {
                    {
                        let active = SESSION_ID.read().as_deref() == Some(sess.id.as_str());
                        let meta = sess.clone();
                        let title = SESSION_TITLES.read().get(&sess.id).cloned().unwrap_or(sess.title.clone());
                        rsx! {
                            div {
                                key: "{sess.id}",
                                class: if active { "session-item active" } else { "session-item" },
                                onclick: move |_| { spawn(load_session(meta.clone())); },
                                div { class: "session-title", "{title}" }
                                div { class: "session-meta",
                                    {sess.cwd.rsplit('/').next().unwrap_or("").to_string()}
                                    " - "
                                    {format_updated_at(&sess.updated_at, now_epoch())}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
