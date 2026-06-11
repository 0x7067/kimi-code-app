//! Git integration for the diff review pane and multi-agent worktrees.

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

/// List git worktrees for the project (F-004.2 / F-004.11).
#[tauri::command]
pub async fn list_worktrees(cwd: String) -> Result<Vec<Value>, String> {
    let out = tokio::process::Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(&cwd)
        .output()
        .await
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Ok(Vec::new());
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let mut worktrees = Vec::new();
    let mut current = std::collections::HashMap::new();
    for line in text.lines() {
        if line.is_empty() {
            if let Some(path) = current.get("worktree") {
                let is_main = current.get("HEAD").is_some() && current.get("branch").is_none();
                worktrees.push(json!({
                    "path": path,
                    "head": current.get("HEAD").unwrap_or(&"".to_string()),
                    "branch": current.get("branch").unwrap_or(&"".to_string()),
                    "main": is_main,
                }));
            }
            current.clear();
        } else if let Some((key, val)) = line.split_once(' ') {
            current.insert(key.to_string(), val.to_string());
        } else {
            current.insert(line.to_string(), String::new());
        }
    }
    if let Some(path) = current.get("worktree") {
        let is_main = current.get("HEAD").is_some() && current.get("branch").is_none();
        worktrees.push(json!({
            "path": path,
            "head": current.get("HEAD").unwrap_or(&"".to_string()),
            "branch": current.get("branch").unwrap_or(&"".to_string()),
            "main": is_main,
        }));
    }
    Ok(worktrees)
}

/// Create a new git worktree for isolated agent execution (F-004.2).
#[tauri::command]
pub async fn create_worktree(cwd: String, name: String) -> Result<Value, String> {
    let sanitized = name
        .replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "-")
        .trim_matches('-')
        .to_string();
    if sanitized.is_empty() {
        return Err("Worktree name cannot be empty".into());
    }
    let path = std::path::Path::new(&cwd).join(".worktrees").join(&sanitized);
    let out = tokio::process::Command::new("git")
        .args(["worktree", "add", "-b", &sanitized, path.to_str().unwrap_or(".")])
        .current_dir(&cwd)
        .output()
        .await
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(format!("git worktree add failed: {stderr}"));
    }
    Ok(json!({ "path": path.to_string_lossy(), "name": sanitized }))
}

/// Remove a git worktree (F-004.2 cleanup).
#[tauri::command]
pub async fn remove_worktree(cwd: String, path: String) -> Result<(), String> {
    let out = tokio::process::Command::new("git")
        .args(["worktree", "remove", "-f", &path])
        .current_dir(&cwd)
        .output()
        .await
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(format!("git worktree remove failed: {stderr}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_worktree_sanitizes_name() {
        let sanitized = "hello/world test!"
            .replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "-");
        assert_eq!(sanitized, "hello-world-test-");
    }

    #[test]
    fn create_worktree_rejects_empty_name() {
        let sanitized = "!!!"
            .replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "-")
            .trim_matches('-')
            .to_string();
        assert!(sanitized.is_empty());
    }
}
