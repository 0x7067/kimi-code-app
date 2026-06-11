//! F-007: Memory panel showing project index, user preferences, and stored
//! memory snippets with search, pin, and delete.

use crate::actions::{refresh_memories, save_memory, delete_memory, pin_memory};
use crate::ipc::invoke;
use crate::state::*;
use dioxus::prelude::*;
use serde_json::Value;

#[component]
pub fn MemoryPane() -> Element {
    let mut index = use_signal(|| None::<Value>);
    let mut loading = use_signal(|| false);
    let mut search = use_signal(String::new);
    let mut new_content = use_signal(String::new);

    let mut refresh_index = move || {
        if let Some(cwd) = PROJECT.read().clone() {
            loading.set(true);
            spawn(async move {
                match invoke("index_project", serde_json::json!({"cwd": cwd})).await {
                    Ok(v) => index.set(Some(v)),
                    Err(e) => {
                        *ERROR.write() = Some(format!("Index failed: {e}"));
                    }
                }
                loading.set(false);
            });
        }
    };

    use_effect(move || {
        refresh_index();
        spawn(async move { refresh_memories().await; });
    });

    let idx = index.read().clone();
    let settings = APP_SETTINGS.read().clone();
    let snippets = MEMORY_SNIPPETS.read().clone();
    let query = search.read().clone().to_lowercase();

    let filtered: Vec<Value> = if query.is_empty() {
        snippets
    } else {
        snippets.into_iter().filter(|s| {
            s.get("content").and_then(|v| v.as_str())
                .map(|c| c.to_lowercase().contains(&query))
                .unwrap_or(false)
        }).collect()
    };

    rsx! {
        div { class: "memory-pane",
            div { class: "memory-head",
                span { "Memory" }
                div { class: "memory-actions",
                    if *loading.read() {
                        span { class: "memory-spinner", "Indexing…" }
                    }
                    button {
                        class: "ghost",
                        onclick: move |_| { refresh_index(); spawn(async move { refresh_memories().await; }); },
                        "Refresh"
                    }
                    button {
                        class: "ghost",
                        onclick: move |_| *SHOW_MEMORY.write() = false,
                        "Close"
                    }
                }
            }
            div { class: "memory-body",
                // -- Add memory --
                div { class: "memory-section",
                    h4 { "Add memory" }
                    div { class: "memory-row",
                        input {
                            class: "prefs-input",
                            placeholder: "Something to remember about this project…",
                            value: "{new_content}",
                            oninput: move |e| new_content.set(e.value()),
                            onkeydown: move |e| {
                                if e.key() == Key::Enter {
                                    let text = new_content.read().clone();
                                    if !text.trim().is_empty() {
                                        new_content.set(String::new());
                                        spawn(async move {
                                            save_memory(text, "user".into()).await;
                                        });
                                    }
                                }
                            },
                        }
                        button {
                            class: "primary",
                            onclick: move |_| {
                                let text = new_content.read().clone();
                                if !text.trim().is_empty() {
                                    new_content.set(String::new());
                                    spawn(async move {
                                        save_memory(text, "user".into()).await;
                                    });
                                }
                            },
                            "Save"
                        }
                    }
                }

                // -- Search memories --
                if !MEMORY_SNIPPETS.read().is_empty() {
                    div { class: "memory-section",
                        h4 { "Memories ({filtered.len()})" }
                        input {
                            class: "prefs-input",
                            placeholder: "Search memories…",
                            value: "{search}",
                            oninput: move |e| search.set(e.value()),
                        }
                        div { class: "memory-list",
                            for s in filtered.iter() {
                                MemorySnippetRow { snippet: s.clone() }
                            }
                        }
                    }
                }

                // -- Project index --
                if let Some(idx) = idx {
                    ProjectIndexSection { index: idx.clone() }
                }

                // -- User preferences --
                UserPrefsSection {
                    tech_stack: settings.tech_stack.clone(),
                    coding_style: settings.coding_style.clone(),
                    naming_conventions: settings.naming_conventions.clone(),
                }
            }
        }
    }
}

#[component]
fn MemorySnippetRow(snippet: Value) -> Element {
    let id = snippet.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let content = snippet.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let pinned = snippet.get("pinned").and_then(|v| v.as_bool()).unwrap_or(false);
    let source = snippet.get("source").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let created = snippet.get("createdAt").and_then(|v| v.as_i64()).unwrap_or(0);
    let rel = snippet.get("relevanceScore").and_then(|v| v.as_f64());

    let id_clone = id.clone();
    let id_pin = id.clone();

    rsx! {
        div {
            class: if pinned { "memory-snippet pinned" } else { "memory-snippet" },
            div { class: "memory-snippet-content", "{content}" }
            div { class: "memory-snippet-meta",
                span { class: "memory-snippet-source", "{source}" }
                if let Some(r) = rel {
                    span { class: "memory-snippet-score", "score: {r:.2}" }
                }
                span { class: "memory-snippet-date", "{created}" }
                button {
                    class: "ghost",
                    onclick: move |_| {
                        let id = id_pin.clone();
                        spawn(async move { pin_memory(id, !pinned).await; });
                    },
                    if pinned { "Unpin" } else { "Pin" }
                }
                button {
                    class: "ghost danger",
                    onclick: move |_| {
                        let id = id_clone.clone();
                        request_confirm(
                            "Delete memory snippet?",
                            "This snippet will be removed permanently. This cannot be undone.",
                            "Delete",
                            true,
                            move || {
                                let id = id.clone();
                                spawn(async move { delete_memory(id).await; });
                            },
                        );
                    },
                    "Delete"
                }
            }
        }
    }
}

#[component]
fn ProjectIndexSection(index: Value) -> Element {
    let key_files = index.get("keyFiles").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    let deps = index.get("dependencies").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    let langs = index.get("languages").and_then(|v| v.as_object()).cloned().unwrap_or_default();
    let total_files = index.get("totalFiles").and_then(|v| v.as_u64()).unwrap_or(0);
    let total_dirs = index.get("totalDirs").and_then(|v| v.as_u64()).unwrap_or(0);

    rsx! {
        div { class: "memory-section",
            h4 { "Project index" }
            div { class: "memory-stats",
                span { "{total_files} files" }
                span { "{total_dirs} dirs" }
            }
            if !key_files.is_empty() {
                div { class: "memory-subsection",
                    h5 { "Key files" }
                    ul { class: "memory-list",
                        for f in key_files.iter() {
                            if let Some(s) = f.as_str() {
                                li { key: "{s}", "{s}" }
                            }
                        }
                    }
                }
            }
            if !deps.is_empty() {
                div { class: "memory-subsection",
                    h5 { "Dependencies ({deps.len()})" }
                    div { class: "memory-tags",
                        for d in deps.iter() {
                            if let Some(s) = d.as_str() {
                                span { key: "{s}", class: "memory-tag", "{s}" }
                            }
                        }
                    }
                }
            }
            if !langs.is_empty() {
                div { class: "memory-subsection",
                    h5 { "Languages" }
                    div { class: "memory-tags",
                        for (lang, count) in langs.iter() {
                            span { key: "{lang}", class: "memory-tag", "{lang} ({count})" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn UserPrefsSection(tech_stack: String, coding_style: String, naming_conventions: String) -> Element {
    rsx! {
        div { class: "memory-section",
            h4 { "User preferences" }
            if !tech_stack.is_empty() {
                div { class: "memory-pref-row",
                    span { class: "memory-pref-label", "Tech stack" }
                    span { class: "memory-pref-value", "{tech_stack}" }
                }
            }
            if !coding_style.is_empty() {
                div { class: "memory-pref-row",
                    span { class: "memory-pref-label", "Coding style" }
                    span { class: "memory-pref-value", "{coding_style}" }
                }
            }
            if !naming_conventions.is_empty() {
                div { class: "memory-pref-row",
                    span { class: "memory-pref-label", "Naming conventions" }
                    span { class: "memory-pref-value", "{naming_conventions}" }
                }
            }
            if tech_stack.is_empty() && coding_style.is_empty() && naming_conventions.is_empty() {
                p { class: "memory-hint",
                    "No preferences set. Open Settings → Memory preferences to add them."
                }
            }
        }
    }
}
