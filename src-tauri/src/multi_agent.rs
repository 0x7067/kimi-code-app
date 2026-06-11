//! F-004: Multi-agent orchestration state tracker.
//!
//! Manages parallel ACP sessions in isolated git worktrees. Each agent is a
//! separate ACP session (same subprocess, different sessionId + cwd). Updates
//! for non-current sessions are routed here instead of the main UI thread.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AgentTask {
    pub id: String,
    pub name: String,
    pub prompt: String,
    pub worktree_path: String,
    pub session_id: Option<String>,
    pub status: String, // "pending" | "running" | "done" | "error"
    pub output: String,
    pub tool_calls: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MultiAgentRun {
    pub run_id: String,
    pub parent_cwd: String,
    pub tasks: Vec<AgentTask>,
    pub created_at: i64,
}

pub struct MultiAgentState {
    /// Active and recent multi-agent runs, keyed by run_id.
    pub runs: Mutex<HashMap<String, MultiAgentRun>>,
    /// Current run_id being displayed in the UI.
    pub current_run: Mutex<Option<String>>,
}

impl Default for MultiAgentState {
    fn default() -> Self {
        Self {
            runs: Mutex::new(HashMap::new()),
            current_run: Mutex::new(None),
        }
    }
}

impl MultiAgentState {
    pub fn start_run(&self, parent_cwd: String, tasks: Vec<AgentTask>) -> String {
        let run_id = format!(
            "ma-{}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            std::process::id()
        );
        let run = MultiAgentRun {
            run_id: run_id.clone(),
            parent_cwd,
            tasks,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        };
        self.runs.lock().unwrap().insert(run_id.clone(), run);
        *self.current_run.lock().unwrap() = Some(run_id.clone());
        run_id
    }

    pub fn get_run(&self, run_id: &str) -> Option<MultiAgentRun> {
        self.runs.lock().unwrap().get(run_id).cloned()
    }

    pub fn list_runs(&self, limit: usize) -> Vec<MultiAgentRun> {
        let runs = self.runs.lock().unwrap();
        let mut list: Vec<MultiAgentRun> = runs.values().cloned().collect();
        list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        list.truncate(limit);
        list
    }

    pub fn set_task_session(&self, run_id: &str, task_id: &str, session_id: String) {
        let mut runs = self.runs.lock().unwrap();
        if let Some(run) = runs.get_mut(run_id) {
            for t in &mut run.tasks {
                if t.id == task_id {
                    t.session_id = Some(session_id);
                    t.status = "running".into();
                    break;
                }
            }
        }
    }

    pub fn apply_update(&self, run_id: &str, session_id: &str, update: &Value) {
        let mut runs = self.runs.lock().unwrap();
        let Some(run) = runs.get_mut(run_id) else { return };
        let Some(task) = run.tasks.iter_mut().find(|t| t.session_id.as_deref() == Some(session_id)) else {
            return;
        };

        let kind = update
            .get("sessionUpdate")
            .and_then(|s| s.as_str())
            .unwrap_or("");
        match kind {
            "agent_message_chunk" => {
                if let Some(text) = content_text(update.get("content")) {
                    task.output.push_str(&text);
                }
            }
            "tool_call" => {
                if let Some(title) = update.get("title").and_then(|s| s.as_str()) {
                    task.tool_calls.push(title.to_string());
                }
            }
            _ => {}
        }
        if let Some(reason) = update.get("stopReason").and_then(|s| s.as_str()) {
            task.status = if reason == "error" {
                "error".into()
            } else {
                "done".into()
            };
        }
    }

    pub fn set_task_status(&self, run_id: &str, task_id: &str, status: &str) {
        let mut runs = self.runs.lock().unwrap();
        if let Some(run) = runs.get_mut(run_id) {
            for t in &mut run.tasks {
                if t.id == task_id {
                    t.status = status.into();
                    break;
                }
            }
        }
    }
}

fn content_text(v: Option<&Value>) -> Option<String> {
    let v = v?;
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Array(blocks) => Some(
            blocks
                .iter()
                .filter_map(|b| content_text(Some(b)))
                .collect::<Vec<_>>()
                .join(""),
        ),
        Value::Object(_) => v
            .get("text")
            .and_then(|t| t.as_str())
            .map(String::from)
            .or_else(|| content_text(v.get("content"))),
        _ => None,
    }
}
