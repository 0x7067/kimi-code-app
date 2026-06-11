//! F-004: Multi-agent orchestration commands.

use serde_json::{json, Value};
use tauri::{AppHandle, Manager};

#[tauri::command]
pub async fn create_multi_agent_run(
    app: AppHandle,
    parent_cwd: String,
    task_names: Vec<String>,
) -> Result<Value, String> {
    let state = app.state::<crate::commands::AppState>();
    let tasks: Vec<crate::multi_agent::AgentTask> = task_names
        .into_iter()
        .enumerate()
        .map(|(i, name)| crate::multi_agent::AgentTask {
            id: format!("task-{}", i),
            name,
            prompt: String::new(),
            worktree_path: String::new(),
            session_id: None,
            status: "pending".into(),
            output: String::new(),
            tool_calls: Vec::new(),
        })
        .collect();
    let run_id = state.multi_agent.start_run(parent_cwd, tasks);
    Ok(json!({"runId": run_id}))
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
