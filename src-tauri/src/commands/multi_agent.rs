//! F-004: Multi-agent orchestration commands.

use serde_json::Value;
use tauri::{AppHandle, Manager};

fn build_agent_tasks(
    parent_cwd: &str,
    run_id: &str,
    task_names: Vec<String>,
) -> Vec<crate::multi_agent::AgentTask> {
    task_names
        .into_iter()
        .filter_map(|name| {
            let name = name.trim().to_string();
            if name.is_empty() {
                return None;
            }
            Some(name)
        })
        .enumerate()
        .map(|(i, name)| {
            let id = format!("task-{}", i);
            let worktree_path = std::path::Path::new(parent_cwd)
                .join(".worktrees")
                .join(format!("{run_id}-{id}"))
                .to_string_lossy()
                .to_string();
            crate::multi_agent::AgentTask {
                id,
                prompt: name.clone(),
                name,
                worktree_path,
                session_id: None,
                status: "pending".into(),
                output: String::new(),
                tool_calls: Vec::new(),
            }
        })
        .collect()
}

fn runnable_task_ids(run: &crate::multi_agent::MultiAgentRun) -> Vec<String> {
    run.tasks
        .iter()
        .filter(|task| task.status != "running")
        .map(|task| task.id.clone())
        .collect()
}

#[tauri::command]
pub async fn create_multi_agent_run(
    app: AppHandle,
    parent_cwd: String,
    task_names: Vec<String>,
) -> Result<Value, String> {
    let state = app.state::<crate::commands::AppState>();
    let run_id = format!(
        "ma-{}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_millis(),
        std::process::id()
    );
    let tasks = build_agent_tasks(&parent_cwd, &run_id, task_names);
    if tasks.is_empty() {
        return Err("No subtasks to run".into());
    }
    let run_id = state.multi_agent.start_run_with_id(run_id, parent_cwd, tasks);
    let run = state
        .multi_agent
        .get_run(&run_id)
        .ok_or_else(|| "run not found after creation".to_string())?;
    Ok(serde_json::to_value(&run).map_err(|e| e.to_string())?)
}

#[tauri::command]
pub async fn list_multi_agent_runs(app: AppHandle, limit: usize) -> Result<Value, String> {
    let state = app.state::<crate::commands::AppState>();
    let runs = state.multi_agent.list_runs(limit);
    Ok(serde_json::to_value(&runs).map_err(|e| e.to_string())?)
}

#[tauri::command]
pub async fn get_multi_agent_run(app: AppHandle, run_id: String) -> Result<Value, String> {
    let state = app.state::<crate::commands::AppState>();
    let run = state
        .multi_agent
        .get_run(&run_id)
        .ok_or_else(|| "run not found".to_string())?;
    Ok(serde_json::to_value(&run).map_err(|e| e.to_string())?)
}

async fn ensure_task_worktree(parent_cwd: &str, task: &crate::multi_agent::AgentTask) -> Result<(), String> {
    let path = std::path::Path::new(&task.worktree_path);
    if path.exists() {
        return Ok(());
    }
    let branch = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(&task.id)
        .replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "-");
    let out = tokio::process::Command::new("git")
        .args(["worktree", "add", "-b", &branch, &task.worktree_path])
        .current_dir(parent_cwd)
        .output()
        .await
        .map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(())
    } else {
        Err(format!(
            "git worktree add failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ))
    }
}

#[tauri::command]
pub async fn run_multi_agent_task(app: AppHandle, run_id: String, task_id: String) -> Result<Value, String> {
    let state = app.state::<crate::commands::AppState>();
    let run = state
        .multi_agent
        .get_run(&run_id)
        .ok_or_else(|| "run not found".to_string())?;
    let task = run
        .tasks
        .iter()
        .find(|task| task.id == task_id)
        .cloned()
        .ok_or_else(|| "task not found".to_string())?;

    state.multi_agent.set_task_status(&run_id, &task_id, "running");
    if let Err(err) = ensure_task_worktree(&run.parent_cwd, &task).await {
        state
            .multi_agent
            .set_task_result(&run_id, &task_id, "error", err, Vec::new());
        let run = state
            .multi_agent
            .get_run(&run_id)
            .ok_or_else(|| "run not found after worktree error".to_string())?;
        return Ok(serde_json::to_value(&run).map_err(|e| e.to_string())?);
    }
    let result = crate::headless::run_prompt(&task.worktree_path, &task.prompt).await;
    match result {
        Ok(res) => {
            let status = if res.stop_reason == "cancelled" {
                "cancelled"
            } else {
                "done"
            };
            state
                .multi_agent
                .set_task_result(&run_id, &task_id, status, res.text, res.tool_calls);
        }
        Err(err) => {
            state
                .multi_agent
                .set_task_result(&run_id, &task_id, "error", err, Vec::new());
        }
    }
    let run = state
        .multi_agent
        .get_run(&run_id)
        .ok_or_else(|| "run not found after task execution".to_string())?;
    Ok(serde_json::to_value(&run).map_err(|e| e.to_string())?)
}

#[tauri::command]
pub async fn run_multi_agent_tasks(app: AppHandle, run_id: String) -> Result<Value, String> {
    let state = app.state::<crate::commands::AppState>();
    let run = state
        .multi_agent
        .get_run(&run_id)
        .ok_or_else(|| "run not found".to_string())?;
    let task_ids = runnable_task_ids(&run);
    for task_id in task_ids {
        let _ = run_multi_agent_task(app.clone(), run_id.clone(), task_id).await?;
    }
    let run = state
        .multi_agent
        .get_run(&run_id)
        .ok_or_else(|| "run not found after task execution".to_string())?;
    Ok(serde_json::to_value(&run).map_err(|e| e.to_string())?)
}

#[tauri::command]
pub async fn set_task_session(
    app: AppHandle,
    run_id: String,
    task_id: String,
    session_id: String,
) -> Result<(), String> {
    let state = app.state::<crate::commands::AppState>();
    state.multi_agent.set_task_session(&run_id, &task_id, session_id);
    Ok(())
}

#[tauri::command]
pub async fn set_task_status(
    app: AppHandle,
    run_id: String,
    task_id: String,
    status: String,
) -> Result<(), String> {
    let state = app.state::<crate::commands::AppState>();
    state.multi_agent.set_task_status(&run_id, &task_id, &status);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_tasks_with_prompts_and_worktree_paths() {
        let tasks = build_agent_tasks(
            "/repo",
            "ma-123",
            vec!["Review auth".to_string(), "Add tests".to_string()],
        );

        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].id, "task-0");
        assert_eq!(tasks[0].prompt, "Review auth");
        assert!(tasks[0].worktree_path.ends_with("/repo/.worktrees/ma-123-task-0"));
        assert_eq!(tasks[0].status, "pending");
        assert_eq!(tasks[1].id, "task-1");
    }

    #[test]
    fn blank_task_names_are_dropped() {
        let tasks = build_agent_tasks("/repo", "ma-123", vec![" ".to_string(), "Ship".to_string()]);

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name, "Ship");
    }

    #[test]
    fn runnable_task_ids_skip_running_tasks() {
        let run = crate::multi_agent::MultiAgentRun {
            run_id: "ma-test".into(),
            parent_cwd: "/repo".into(),
            created_at: 0,
            tasks: vec![
                crate::multi_agent::AgentTask {
                    id: "task-0".into(),
                    name: "Done before".into(),
                    prompt: "Done before".into(),
                    worktree_path: "/repo/.worktrees/done".into(),
                    session_id: None,
                    status: "done".into(),
                    output: String::new(),
                    tool_calls: Vec::new(),
                },
                crate::multi_agent::AgentTask {
                    id: "task-1".into(),
                    name: "Running now".into(),
                    prompt: "Running now".into(),
                    worktree_path: "/repo/.worktrees/running".into(),
                    session_id: None,
                    status: "running".into(),
                    output: String::new(),
                    tool_calls: Vec::new(),
                },
                crate::multi_agent::AgentTask {
                    id: "task-2".into(),
                    name: "Pending".into(),
                    prompt: "Pending".into(),
                    worktree_path: "/repo/.worktrees/pending".into(),
                    session_id: None,
                    status: "pending".into(),
                    output: String::new(),
                    tool_calls: Vec::new(),
                },
            ],
        };

        assert_eq!(runnable_task_ids(&run), vec!["task-0", "task-2"]);
    }
}
