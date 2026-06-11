//! F-009: Automation scheduler and execution history.
//!
//! Automations are stored in the app-settings JSON (front-end owns the schema).
//! The backend scheduler wakes every 60s, checks cron expressions, and runs
//! eligible automations headlessly via `headless::run_prompt`.

use chrono::{TimeZone, Utc};
use cron::Schedule;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;
use tauri::Manager;
use tokio::time::interval;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AutomationDef {
    pub id: String,
    pub name: String,
    pub cron: String,
    pub prompt: String,
    pub cwd: String,
    pub enabled: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ExecutionRecord {
    pub runs: Vec<ExecutionRun>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionRun {
    pub automation_id: String,
    pub started_at: i64,
    pub finished_at: i64,
    pub status: String, // "success" | "error" | "cancelled"
    pub output: String,
    pub tool_calls: Vec<String>,
}

fn history_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| e.to_string())?
        .join("automations");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("history.json"))
}

fn load_history(app: &tauri::AppHandle) -> Result<ExecutionRecord, String> {
    let path = history_path(app)?;
    match std::fs::read_to_string(&path) {
        Ok(s) => serde_json::from_str(&s).map_err(|e| e.to_string()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(ExecutionRecord::default()),
        Err(e) => Err(e.to_string()),
    }
}

fn save_history(app: &tauri::AppHandle, rec: &ExecutionRecord) -> Result<(), String> {
    let path = history_path(app)?;
    let body = serde_json::to_string_pretty(rec).map_err(|e| e.to_string())?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, body).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, path).map_err(|e| e.to_string())?;
    Ok(())
}

/// Append a run to the history log, pruning old entries after 100.
pub fn log_run(app: &tauri::AppHandle, run: ExecutionRun) -> Result<(), String> {
    let mut rec = load_history(app)?;
    rec.runs.push(run);
    if rec.runs.len() > 100 {
        rec.runs.drain(0..rec.runs.len() - 100);
    }
    save_history(app, &rec)
}

/// List recent runs, newest first.
pub fn list_runs(app: &tauri::AppHandle, limit: usize) -> Result<Vec<ExecutionRun>, String> {
    let mut rec = load_history(app)?;
    rec.runs.reverse();
    rec.runs.truncate(limit);
    Ok(rec.runs)
}

/// Return true when `cron_expr` had at least one scheduled occurrence in the
/// half-open tick window `(last_tick_ts, now_ts]`.
pub fn cron_due_between(cron_expr: &str, last_tick_ts: i64, now_ts: i64) -> bool {
    if now_ts <= last_tick_ts {
        return false;
    }
    let Ok(schedule) = Schedule::from_str(cron_expr) else {
        return false;
    };
    let Some(last_tick) = Utc.timestamp_opt(last_tick_ts, 0).single() else {
        return false;
    };
    schedule
        .after(&last_tick)
        .take_while(|dt| dt.timestamp() <= now_ts)
        .any(|dt| dt.timestamp() > last_tick_ts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn due_between_fires_for_occurrence_inside_window() {
        let before = Utc.with_ymd_and_hms(2026, 6, 11, 8, 59, 30).unwrap().timestamp();
        let after = Utc.with_ymd_and_hms(2026, 6, 11, 9, 0, 15).unwrap().timestamp();

        assert!(cron_due_between("0 0 9 * * *", before, after));
    }

    #[test]
    fn due_between_does_not_fire_for_future_only_schedule() {
        let before = Utc.with_ymd_and_hms(2026, 6, 11, 8, 58, 0).unwrap().timestamp();
        let after = Utc.with_ymd_and_hms(2026, 6, 11, 8, 59, 0).unwrap().timestamp();

        assert!(!cron_due_between("0 0 9 * * *", before, after));
    }

    #[test]
    fn due_between_rejects_invalid_cron() {
        assert!(!cron_due_between("not cron", 100, 200));
    }
}

fn read_automations_from_disk(app: &tauri::AppHandle) -> Vec<AutomationDef> {
    let path = match app.path().app_config_dir() {
        Ok(d) => d.join("app_settings.json"),
        Err(_) => return Vec::new(),
    };
    let raw = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let val: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    val.get("automations")
        .and_then(|a| a.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| serde_json::from_value::<AutomationDef>(v.clone()).ok())
                .collect()
        })
        .unwrap_or_default()
}

/// Start the background scheduler tick. This should be called once at app startup.
pub fn start_scheduler(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut tick = interval(Duration::from_secs(60));
        let mut last_tick = Utc::now().timestamp();
        loop {
            tick.tick().await;
            let list = read_automations_from_disk(&app);
            let now = Utc::now().timestamp();
            for auto in list {
                if !auto.enabled {
                    continue;
                }
                if cron_due_between(&auto.cron, last_tick, now) {
                    let app = app.clone();
                    let auto_id = auto.id.clone();
                    let prompt = auto.prompt.clone();
                    let cwd = auto.cwd.clone();
                    tauri::async_runtime::spawn(async move {
                        let started = Utc::now().timestamp();
                        let result = crate::headless::run_prompt(&cwd, &prompt).await;
                        let finished = Utc::now().timestamp();
                        let run = match result {
                            Ok(res) => ExecutionRun {
                                automation_id: auto_id,
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
                            Err(e) => ExecutionRun {
                                automation_id: auto_id,
                                started_at: started,
                                finished_at: finished,
                                status: "error".into(),
                                output: e,
                                tool_calls: Vec::new(),
                            },
                        };
                        let _ = log_run(&app, run);
                    });
                }
            }
            last_tick = now;
        }
    });
}
