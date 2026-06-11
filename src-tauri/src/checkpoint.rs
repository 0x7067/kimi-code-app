//! Session checkpoint save/restore (F-002.6).
//!
//! Checkpoints are JSON snapshots of a conversation's `items` and `plan`,
//! stored under `<sessionDir>/checkpoints/<name>.json`.

use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// Directory name for checkpoints inside a session dir.
const CHECKPOINT_DIR: &str = "checkpoints";

/// Current UTC time as RFC 3339 (e.g. `2026-06-11T10:55:00Z`).
fn rfc3339_now() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let days_since_1970 = now / 86_400;
    let secs_of_day = (now % 86_400) as u32;
    let (y, m, d) = civil_from_days(days_since_1970);
    let hour = secs_of_day / 3_600;
    let min = (secs_of_day % 3_600) / 60;
    let sec = secs_of_day % 60;
    format!("{y:04}-{m:02}-{d:02}T{hour:02}:{min:02}:{sec:02}Z")
}

/// Reverse of Howard Hinnant's days-from-civil algorithm.
fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = (yoe as i64 + era * 400) as i32;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m as u32, d as u32)
}

/// Build the checkpoint directory path for a session.
fn checkpoint_dir(session_dir: &str) -> PathBuf {
    Path::new(session_dir).join(CHECKPOINT_DIR)
}

/// Build the file path for a named checkpoint.
fn checkpoint_path(session_dir: &str, name: &str) -> PathBuf {
    checkpoint_dir(session_dir).join(format!("{name}.json"))
}

/// Sanitize a checkpoint name for use as a filename.
fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

/// Save a checkpoint. `items` and `plan` are the raw JSON arrays from the
/// frontend. Returns the sanitized name used for the file.
pub fn save_checkpoint(
    session_dir: &str,
    name: &str,
    items: Value,
    plan: Value,
) -> Result<String, String> {
    let safe = sanitize_name(name);
    if safe.is_empty() {
        return Err("checkpoint name cannot be empty".into());
    }
    let dir = checkpoint_dir(session_dir);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let payload = json!({
        "name": &safe,
        "savedAt": rfc3339_now(),
        "items": items,
        "plan": plan,
    });
    let path = checkpoint_path(session_dir, &safe);
    std::fs::write(&path, serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    Ok(safe)
}

/// List all checkpoints for a session, newest first.
pub fn list_checkpoints(session_dir: &str) -> Result<Vec<Value>, String> {
    let dir = checkpoint_dir(session_dir);
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut entries: Vec<(std::time::SystemTime, Value)> = std::fs::read_dir(&dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "json")
                .unwrap_or(false)
        })
        .filter_map(|e| {
            let modified = e.metadata().ok()?.modified().ok()?;
            let content = std::fs::read_to_string(e.path()).ok()?;
            let v: Value = serde_json::from_str(&content).ok()?;
            Some((modified, v))
        })
        .collect();
    entries.sort_by(|a, b| b.0.cmp(&a.0));
    Ok(entries.into_iter().map(|(_, v)| v).collect())
}

/// Load a checkpoint by name. Returns `(items, plan)` as JSON values.
pub fn load_checkpoint(session_dir: &str, name: &str) -> Result<(Value, Value), String> {
    let path = checkpoint_path(session_dir, name);
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let v: Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    let items = v.get("items").cloned().unwrap_or_else(|| json!([]));
    let plan = v.get("plan").cloned().unwrap_or_else(|| json!([]));
    Ok((items, plan))
}

/// Delete a checkpoint by name.
pub fn delete_checkpoint(session_dir: &str, name: &str) -> Result<(), String> {
    let path = checkpoint_path(session_dir, name);
    std::fs::remove_file(&path).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_session_dir() -> (tempfile::TempDir, String) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        (dir, path)
    }

    #[test]
    fn save_creates_checkpoint_file() {
        let (_tmp, session_dir) = temp_session_dir();
        let items = json!([{"role": "user", "content": "hello"}]);
        let plan = json!([{"content": "step 1", "status": "done"}]);
        let name = save_checkpoint(&session_dir, "test-check", items.clone(), plan.clone()).unwrap();
        assert_eq!(name, "test-check");
        let path = checkpoint_path(&session_dir, "test-check");
        assert!(path.exists());
    }

    #[test]
    fn save_sanitizes_name() {
        let (_tmp, session_dir) = temp_session_dir();
        let name = save_checkpoint(&session_dir, "hello world!!!", json!([]), json!([])).unwrap();
        assert_eq!(name, "hello-world");
    }

    #[test]
    fn save_rejects_empty_name() {
        let (_tmp, session_dir) = temp_session_dir();
        let result = save_checkpoint(&session_dir, "!!!", json!([]), json!([]));
        assert!(result.is_err());
    }

    #[test]
    fn list_returns_newest_first() {
        let (_tmp, session_dir) = temp_session_dir();
        save_checkpoint(&session_dir, "old", json!([]), json!([])).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(50));
        save_checkpoint(&session_dir, "new", json!([]), json!([])).unwrap();
        let list = list_checkpoints(&session_dir).unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0]["name"], "new");
        assert_eq!(list[1]["name"], "old");
    }

    #[test]
    fn load_roundtrips_items_and_plan() {
        let (_tmp, session_dir) = temp_session_dir();
        let items = json!([{"role": "user", "content": "hi"}]);
        let plan = json!([{"content": "do thing", "status": "pending"}]);
        save_checkpoint(&session_dir, "roundtrip", items.clone(), plan.clone()).unwrap();
        let (loaded_items, loaded_plan) = load_checkpoint(&session_dir, "roundtrip").unwrap();
        assert_eq!(loaded_items, items);
        assert_eq!(loaded_plan, plan);
    }

    #[test]
    fn delete_removes_checkpoint() {
        let (_tmp, session_dir) = temp_session_dir();
        save_checkpoint(&session_dir, "gone", json!([]), json!([])).unwrap();
        assert!(checkpoint_path(&session_dir, "gone").exists());
        delete_checkpoint(&session_dir, "gone").unwrap();
        assert!(!checkpoint_path(&session_dir, "gone").exists());
    }

    #[test]
    fn list_empty_dir_returns_empty() {
        let (_tmp, session_dir) = temp_session_dir();
        let list = list_checkpoints(&session_dir).unwrap();
        assert!(list.is_empty());
    }
}