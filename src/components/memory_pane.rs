//! F-007.10: Memory panel showing project index and user preferences.

use crate::ipc::invoke;
use crate::state::*;
use dioxus::prelude::*;
use serde_json::Value;

#[component]
pub fn MemoryPane() -> Element {
    let mut index = use_signal(|| None::<Value>);
    let mut loading = use_signal(|| false);

    let mut refresh = move || {
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
        refresh();
    });

    let idx = index.read().clone();
    let settings = APP_SETTINGS.read().clone();

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
                        onclick: move |_| refresh(),
                        "Re-index"
                    }
                    button {
                        class: "ghost",
                        onclick: move |_| *SHOW_MEMORY.write() = false,
                        "Close"
                    }
                }
            }
            div { class: "memory-body",
                if let Some(idx) = idx {
                    ProjectIndexSection { index: idx.clone() }
                }
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
