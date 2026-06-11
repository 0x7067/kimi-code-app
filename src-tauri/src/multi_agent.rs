//! F-004: Multi-agent orchestration state tracker.
//!
//! Manages parallel ACP sessions in isolated git worktrees. Each agent is a
//! separate ACP session (same subprocess, different sessionId + cwd). Updates
//! for non-current sessions are routed here instead of the main UI thread.

use serde::{Deserialize, Serialize};
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
