//! Git integration for the diff review pane.

use serde_json::{json, Value};

/// Git diff of working tree for the review pane.
#[tauri::command]
pub async fn git_diff(cwd: String) -> Result<Value, String> {
    let status = tokio::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&cwd)
        .output()
        .await
        .map_err(|e| e.to_string())?;
    if !status.status.success() {
        return Ok(json!({ "isRepo": false, "files": [], "diff": "" }));
    }
    let diff = tokio::process::Command::new("git")
        .args(["diff", "HEAD"])
        .current_dir(&cwd)
        .output()
        .await
        .map_err(|e| e.to_string())?;
    Ok(json!({
        "isRepo": true,
        "files": String::from_utf8_lossy(&status.stdout).lines().map(|l| l.to_string()).collect::<Vec<_>>(),
        "diff": String::from_utf8_lossy(&diff.stdout),
    }))
}
