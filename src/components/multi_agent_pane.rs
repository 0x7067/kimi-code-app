//! F-004: Multi-agent orchestration — task decomposition, parallel execution,
//! and progress dashboard.

use crate::ipc::invoke;
use crate::state::*;
use dioxus::prelude::*;
use serde_json::Value;

#[component]
pub fn MultiAgentPane() -> Element {
    let mut worktrees = use_signal(Vec::<Value>::new);
    let mut loading = use_signal(|| false);
    let mut new_name = use_signal(String::new);
    let mut task_input = use_signal(String::new);
    let mut decomposing = use_signal(|| false);
    let mut run = use_signal(|| None::<Value>);

    let mut refresh_wt = move || {
        if let Some(cwd) = PROJECT.read().clone() {
            loading.set(true);
            spawn(async move {
                match invoke("list_worktrees", serde_json::json!({"cwd": cwd})).await {
                    Ok(Value::Array(list)) => worktrees.set(list),
                    _ => {}
                }
                loading.set(false);
            });
        }
    };

    use_effect(move || {
        refresh_wt();
    });

    let create_wt = move || {
        let name = new_name.read().trim().to_string();
        if name.is_empty() {
            return;
        }
        if let Some(cwd) = PROJECT.read().clone() {
            spawn(async move {
                match invoke("create_worktree", serde_json::json!({"cwd": cwd, "name": name})).await {
                    Ok(_) => {
                        new_name.set(String::new());
                        refresh_wt();
                    }
                    Err(e) => *ERROR.write() = Some(format!("Create worktree failed: {e}")),
                }
            });
        }
    };

    let mut decompose = move || {
        let text = task_input.read().trim().to_string();
        if text.is_empty() {
            return;
        }
        decomposing.set(true);
        spawn(async move {
            // Ask the current agent to decompose the task.
            let prompt = format!(
                "Break the following complex task into 2-5 independent subtasks that can be worked on in parallel. \
                Return ONLY a JSON array of strings, each string being a concise subtask description. \
                Do not include any other text. Task: {}",
                text
            );
            // We'll use the headless runner for decomposition to avoid cluttering the chat.
            let cwd = PROJECT.read().clone().unwrap_or_default();
            match invoke(
                "run_automation_now",
                serde_json::json!({"automationId": "decompose", "prompt": prompt, "cwd": cwd}),
            )
            .await
            {
                Ok(Value::Object(mut res)) => {
                    let output = res
                        .remove("output")
                        .and_then(|v| v.as_str().map(String::from))
                        .unwrap_or_default();
                    // Try to parse JSON array from the output.
                    let names: Vec<String> = serde_json::from_str(&output).unwrap_or_else(|_| {
                        // Fallback: split by newlines and filter empty.
                        output
                            .lines()
                            .map(|l| l.trim().trim_start_matches("- ").to_string())
                            .filter(|l| !l.is_empty())
                            .collect()
                    });
                    if let Some(cwd) = PROJECT.read().clone() {
                        match invoke(
                            "create_multi_agent_run",
                            serde_json::json!({"parentCwd": cwd, "taskNames": names}),
                        )
                        .await
                        {
                            Ok(v) => run.set(Some(v)),
                            Err(e) => *ERROR.write() = Some(format!("Create run failed: {e}")),
                        }
                    }
                }
                Ok(_) => *ERROR.write() = Some("Decomposition returned unexpected format".into()),
                Err(e) => *ERROR.write() = Some(format!("Decomposition failed: {e}")),
            }
            decomposing.set(false);
        });
    };

    let list = worktrees.read().clone();
    let current_run = run.read().clone();

    rsx! {
        div { class: "multi-agent-pane",
            div { class: "multi-agent-head",
                span { "Multi-agent" }
                button {
                    class: "ghost",
                    onclick: move |_| *SHOW_MULTI_AGENT.write() = false,
                    "Close"
                }
            }
            div { class: "multi-agent-body",
                // -- Task decomposition --
                div { class: "memory-section",
                    h4 { "Decompose task" }
                    div { class: "memory-row",
                        input {
                            class: "prefs-input",
                            placeholder: "Describe a complex task to break into parallel subtasks…",
                            value: "{task_input}",
                            oninput: move |e| task_input.set(e.value()),
                            onkeydown: move |e| {
                                if e.key() == Key::Enter { decompose(); }
                            },
                        }
                        button {
                            class: "primary",
                            disabled: *decomposing.read(),
                            onclick: move |_| decompose(),
                            if *decomposing.read() { "Decomposing…" } else { "Decompose" }
                        }
                    }
                }

                // -- Active run --
                if let Some(run_val) = current_run {
                    RunDashboard { run: run_val }
                }

                // -- Worktrees --
                div { class: "memory-section",
                    h4 { "Worktrees" }
                    div { class: "multi-agent-create",
                        input {
                            class: "prefs-input",
                            placeholder: "worktree name…",
                            value: "{new_name}",
                            oninput: move |e| new_name.set(e.value()),
                            onkeydown: move |e| {
                                if e.key() == Key::Enter { create_wt(); }
                            },
                        }
                        button {
                            class: "primary",
                            onclick: move |_| create_wt(),
                            "Create worktree"
                        }
                    }
                    if *loading.read() {
                        p { class: "memory-hint", "Loading worktrees…" }
                    } else if list.is_empty() {
                        p { class: "memory-hint", "No worktrees found." }
                    } else {
                        div { class: "multi-agent-list",
                            for wt in list.iter() {
                                {
                                    let path = wt.get("path").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                    let branch = wt.get("branch").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                    let is_main = wt.get("main").and_then(|v| v.as_bool()).unwrap_or(false);
                                    rsx! {
                                        div {
                                            key: "{path}",
                                            class: "multi-agent-row",
                                            div { class: "multi-agent-info",
                                                span { class: "multi-agent-branch",
                                                    if is_main { "main" } else { "{branch}" }
                                                }
                                                span { class: "multi-agent-path", "{path}" }
                                            }
                                            if !is_main {
                                                button {
                                                    class: "ghost danger",
                                                    onclick: move |_| {
                                                        let path = path.clone();
                                                        let branch = branch.clone();
                                                        let refresh_wt = refresh_wt.clone();
                                                        request_confirm(
                                                            "Remove worktree?",
                                                            format!("The worktree \"{branch}\" and its directory will be deleted. Uncommitted changes will be lost."),
                                                            "Remove",
                                                            true,
                                                            move || {
                                                                let path = path.clone();
                                                                let mut refresh_wt = refresh_wt.clone();
                                                                if let Some(cwd) = PROJECT.read().clone() {
                                                                    spawn(async move {
                                                                        match invoke("remove_worktree", serde_json::json!({"cwd": cwd, "path": path})).await {
                                                                            Ok(_) => refresh_wt(),
                                                                            Err(e) => *ERROR.write() = Some(format!("Remove worktree failed: {e}")),
                                                                        }
                                                                    });
                                                                }
                                                            },
                                                        );
                                                    },
                                                    "Remove"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn RunDashboard(run: Value) -> Element {
    let mut current_run = use_signal(|| run.clone());
    let visible_run = current_run.read().clone();
    let run_id = visible_run
        .get("runId")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let tasks = visible_run
        .get("tasks")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    rsx! {
        div { class: "memory-section",
            h4 { "Active run: {run_id}" }
            if !tasks.is_empty() {
                {
                    let ids = tasks
                        .iter()
                        .filter_map(|task| task.get("id").and_then(|v| v.as_str()).map(String::from))
                        .collect::<Vec<_>>();
                    let run_id_all = run_id.clone();
                    rsx! {
                button {
                    class: "ghost",
                    onclick: move |_| {
                        for task_id in ids.clone() {
                            let run_id = run_id_all.clone();
                            spawn(async move {
                                match invoke("run_multi_agent_task", serde_json::json!({"runId": run_id, "taskId": task_id})).await {
                                    Ok(v) => current_run.set(v),
                                    Err(e) => *ERROR.write() = Some(format!("Run task failed: {e}")),
                                }
                            });
                            }
                    },
                    "Run all"
                }
                    }
                }
            }
            div { class: "automation-list",
                for task in tasks.iter() {
                    {
                        let task_id = task.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let name = task.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let status = task.get("status").and_then(|v| v.as_str()).unwrap_or("pending").to_string();
                        let output = task.get("output").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let worktree = task.get("worktreePath").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let status_class = match status.as_str() {
                            "done" => "run-status success",
                            "error" => "run-status error",
                            "running" => "run-status",
                            _ => "run-status disabled",
                        };
                        rsx! {
                            div { class: "automation-card",
                                div { class: "automation-info",
                                    span { class: "automation-name", "{name}" }
                                    span { class: "{status_class}", "{status}" }
                                    if !worktree.is_empty() {
                                        span { class: "automation-cron", "{worktree}" }
                                    }
                                }
                                if status != "running" {
                                    div { class: "automation-actions",
                                        {
                                            let run_id = run_id.clone();
                                            let task_id = task_id.clone();
                                            rsx! {
                                        button {
                                            class: "ghost",
                                            onclick: move |_| {
                                                let run_id = run_id.clone();
                                                let task_id = task_id.clone();
                                                spawn(async move {
                                                    match invoke("run_multi_agent_task", serde_json::json!({"runId": run_id, "taskId": task_id})).await {
                                                        Ok(v) => current_run.set(v),
                                                        Err(e) => *ERROR.write() = Some(format!("Run task failed: {e}")),
                                                    }
                                                });
                                            },
                                            "Run"
                                        }
                                            }
                                        }
                                    }
                                }
                                if !output.is_empty() {
                                    pre { class: "run-output", "{output}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
