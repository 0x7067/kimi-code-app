//! F-009: Automations panel — create, edit, delete, and run automations.

use crate::actions::save_app_settings;
use crate::ipc::invoke;
use crate::state::*;
use dioxus::prelude::*;
use serde_json::Value;

fn update_settings(f: impl FnOnce(&mut AppSettings)) {
    f(&mut APP_SETTINGS.write());
    spawn(async { save_app_settings().await });
}

#[component]
pub fn AutomationPane() -> Element {
    let mut runs = use_signal(Vec::<Value>::new);
    let mut loading_runs = use_signal(|| false);
    let mut editing = use_signal(|| None::<usize>);

    let mut refresh_runs = move || {
        loading_runs.set(true);
        spawn(async move {
            match invoke("list_automation_runs", serde_json::json!({"limit": 20})).await {
                Ok(Value::Array(arr)) => runs.set(arr),
                _ => {}
            }
            loading_runs.set(false);
        });
    };

    use_effect(move || {
        refresh_runs();
    });

    let automations = APP_SETTINGS.read().automations.clone();
    let is_editing = editing.read().clone();

    rsx! {
        div { class: "automation-pane",
            div { class: "automation-head",
                span { "Automations" }
                div { class: "automation-actions",
                    button {
                        class: "ghost",
                        onclick: move |_| refresh_runs(),
                        "Refresh history"
                    }
                    button {
                        class: "primary",
                        onclick: move |_| {
                            editing.set(Some(automations.len()));
                        },
                        "+ New automation"
                    }
                    button {
                        class: "ghost",
                        onclick: move |_| *SHOW_AUTOMATIONS.write() = false,
                        "Close"
                    }
                }
            }
            div { class: "automation-body",
                // -- Automation list --
                div { class: "automation-section",
                    h4 { "Active automations" }
                    if automations.is_empty() {
                        p { class: "memory-hint", "No automations configured yet." }
                    } else {
                        div { class: "automation-list",
                            for (idx, auto) in automations.iter().enumerate() {
                                AutomationCard {
                                    idx,
                                    auto: auto.clone(),
                                    editing: is_editing == Some(idx),
                                    on_edit: move |_| editing.set(Some(idx)),
                                    on_done: move |_| editing.set(None),
                                }
                            }
                        }
                    }
                }

                // -- New automation form --
                if is_editing == Some(automations.len()) {
                    AutomationEditor {
                        idx: automations.len(),
                        auto: crate::state::Automation {
                            id: format!("auto-{}", js_sys::Date::now() as u64),
                            name: String::new(),
                            cron: "0 9 * * 1".into(), // Mondays at 9am
                            prompt: String::new(),
                            cwd: PROJECT.read().clone().unwrap_or_default(),
                            enabled: true,
                        },
                        on_done: move |_| editing.set(None),
                    }
                }

                // -- Execution history --
                div { class: "automation-section",
                    h4 { "Execution history" }
                    if *loading_runs.read() {
                        p { class: "memory-hint", "Loading…" }
                    } else if runs.read().is_empty() {
                        p { class: "memory-hint", "No runs yet." }
                    } else {
                        div { class: "automation-runs",
                            for run in runs.read().iter().cloned() {
                                RunRow { run }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn AutomationCard(
    idx: usize,
    auto: crate::state::Automation,
    editing: bool,
    on_edit: EventHandler<()>,
    on_done: EventHandler<()>,
) -> Element {
    let id = auto.id.clone();
    let id_run = auto.id.clone();
    let id_del = auto.id.clone();

    if editing {
        rsx! {
            AutomationEditor {
                idx,
                auto,
                on_done: move |_| on_done.call(()),
            }
        }
    } else {
        rsx! {
            div { class: "automation-card",
                div { class: "automation-info",
                    span { class: "automation-name", "{auto.name}" }
                    span { class: "automation-cron", "{auto.cron}" }
                    span { class: if auto.enabled { "automation-status enabled" } else { "automation-status disabled" },
                        if auto.enabled { "Enabled" } else { "Disabled" }
                    }
                }
                div { class: "automation-actions",
                    button {
                        class: "ghost",
                        onclick: move |_| on_edit.call(()),
                        "Edit"
                    }
                    button {
                        class: "ghost",
                        onclick: move |_| {
                            let id = id_run.clone();
                            let prompt = auto.prompt.clone();
                            let cwd = auto.cwd.clone();
                            spawn(async move {
                                match invoke("run_automation_now", serde_json::json!({"automationId": id, "prompt": prompt, "cwd": cwd})).await {
                                    Ok(_) => {}
                                    Err(e) => *ERROR.write() = Some(format!("Run failed: {e}")),
                                }
                            });
                        },
                        "Run now"
                    }
                    button {
                        class: "ghost danger",
                        onclick: move |_| {
                            let id = id_del.clone();
                            update_settings(|s| {
                                s.automations.retain(|a| a.id != id);
                            });
                        },
                        "Delete"
                    }
                }
            }
        }
    }
}

#[component]
fn AutomationEditor(
    idx: usize,
    auto: crate::state::Automation,
    on_done: EventHandler<()>,
) -> Element {
    let mut name = use_signal(|| auto.name.clone());
    let mut cron = use_signal(|| auto.cron.clone());
    let mut prompt = use_signal(|| auto.prompt.clone());
    let mut cwd = use_signal(|| auto.cwd.clone());
    let mut enabled = use_signal(|| auto.enabled);
    let is_new = idx >= APP_SETTINGS.read().automations.len();

    rsx! {
        div { class: "automation-editor",
            div { class: "prefs-row",
                label { class: "prefs-label", "Name" }
                input {
                    class: "prefs-input",
                    value: "{name}",
                    oninput: move |e| name.set(e.value()),
                }
            }
            div { class: "prefs-row",
                label { class: "prefs-label", "Schedule (cron)" }
                input {
                    class: "prefs-input",
                    value: "{cron}",
                    oninput: move |e| cron.set(e.value()),
                }
            }
            div { class: "prefs-row",
                label { class: "prefs-label", "Working directory" }
                input {
                    class: "prefs-input",
                    value: "{cwd}",
                    oninput: move |e| cwd.set(e.value()),
                }
            }
            div { class: "prefs-row",
                label { class: "prefs-label", "Prompt" }
                textarea {
                    class: "settings-editor",
                    value: "{prompt}",
                    oninput: move |e| prompt.set(e.value()),
                    rows: 4,
                }
            }
            div { class: "prefs-row prefs-approval",
                label { class: "prefs-label",
                    input {
                        r#type: "checkbox",
                        checked: *enabled.read(),
                        onchange: move |e| enabled.set(e.checked()),
                    }
                    "Enabled"
                }
            }
            div { class: "automation-actions",
                button {
                    class: "primary",
                    onclick: move |_| {
                        let updated = crate::state::Automation {
                            id: auto.id.clone(),
                            name: name.read().clone(),
                            cron: cron.read().clone(),
                            prompt: prompt.read().clone(),
                            cwd: cwd.read().clone(),
                            enabled: *enabled.read(),
                        };
                        update_settings(|s| {
                            if is_new {
                                s.automations.push(updated);
                            } else if let Some(a) = s.automations.get_mut(idx) {
                                *a = updated;
                            }
                        });
                        on_done.call(());
                    },
                    "Save"
                }
                button {
                    class: "ghost",
                    onclick: move |_| on_done.call(()),
                    "Cancel"
                }
            }
        }
    }
}

#[component]
fn RunRow(run: Value) -> Element {
    let status = run.get("status").and_then(|v| v.as_str()).unwrap_or("");
    let output = run.get("output").and_then(|v| v.as_str()).unwrap_or("");
    let started = run.get("startedAt").and_then(|v| v.as_i64()).unwrap_or(0);
    let status_class = match status {
        "success" => "run-status success",
        "error" => "run-status error",
        _ => "run-status",
    };

    rsx! {
        div { class: "automation-run",
            div { class: "run-header",
                span { class: "{status_class}", "{status}" }
                span { class: "run-time", "{started}" }
            }
            if !output.is_empty() {
                pre { class: "run-output", "{output}" }
            }
        }
    }
}
