//! F-009: Automation execution and history commands.

use serde_json::Value;
use tauri::AppHandle;

#[tauri::command]
pub async fn list_automation_runs(app: AppHandle, limit: usize) -> Result<Value, String> {
    let runs = crate::automation::list_runs(&app, limit)?;
    Ok(serde_json::to_value(&runs).map_err(|e| e.to_string())?)
}

#[tauri::command]
pub async fn run_automation_now(
    app: AppHandle,
    cwd: String,
    prompt: String,
    automation_id: String,
) -> Result<Value, String> {
    let started = chrono::Utc::now().timestamp();
    let result = crate::headless::run_prompt(&cwd, &prompt).await;
    let finished = chrono::Utc::now().timestamp();
    let run = match result {
        Ok(res) => crate::automation::ExecutionRun {
            automation_id,
            started_at: started,
            finished_at: finished,
            status: if res.stop_reason == "cancelled" {
                "cancelled".into()
            } else {
                "success".into()
            },
            output: res.text,
            tool_calls: res.tool_calls,
        },
        Err(e) => crate::automation::ExecutionRun {
            automation_id,
            started_at: started,
            finished_at: finished,
            status: "error".into(),
            output: e,
            tool_calls: Vec::new(),
        },
    };
    crate::automation::log_run(&app, run.clone())?;
    Ok(serde_json::to_value(&run).map_err(|e| e.to_string())?)
}
