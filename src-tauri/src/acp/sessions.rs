//! Concurrent session registry (F-001.9). Tracks per-session update history
//! so multiple independent sessions never cross-contaminate.

use serde_json::Value;
use std::collections::HashMap;

#[derive(Default)]
pub struct SessionRegistry {
    sessions: HashMap<String, Vec<Value>>,
    order: Vec<String>,
}

impl SessionRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Ensure a session exists (e.g. after `session/new`).
    pub fn ensure(&mut self, id: &str) {
        if !self.sessions.contains_key(id) {
            self.sessions.insert(id.to_string(), Vec::new());
            self.order.push(id.to_string());
        }
    }

    /// Route a `session/update` params object into its session's history.
    /// Returns the session id, or `None` if the params carry no `sessionId`.
    pub fn record_update(&mut self, params: &Value) -> Option<String> {
        let id = params.get("sessionId")?.as_str()?.to_string();
        self.ensure(&id);
        self.sessions.get_mut(&id).expect("ensured").push(params.clone());
        Some(id)
    }

    pub fn messages(&self, id: &str) -> &[Value] {
        self.sessions.get(id).map(Vec::as_slice).unwrap_or(&[])
    }

    /// Session ids in creation order (used for context replay).
    pub fn session_ids(&self) -> Vec<String> {
        self.order.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn update(session: &str, text: &str) -> Value {
        json!({ "sessionId": session, "update": { "sessionUpdate": "agent_message_chunk", "content": { "text": text } } })
    }

    #[test]
    fn routes_two_sessions_without_cross_contamination() {
        let mut reg = SessionRegistry::new();
        assert_eq!(reg.record_update(&update("a", "for-a-1")), Some("a".into()));
        assert_eq!(reg.record_update(&update("b", "for-b-1")), Some("b".into()));
        assert_eq!(reg.record_update(&update("a", "for-a-2")), Some("a".into()));

        assert_eq!(reg.messages("a").len(), 2);
        assert_eq!(reg.messages("b").len(), 1);
        assert_eq!(reg.messages("a")[1]["update"]["content"]["text"], "for-a-2");
        assert_eq!(reg.messages("b")[0]["update"]["content"]["text"], "for-b-1");
        assert_eq!(reg.session_ids(), vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn update_without_session_id_is_not_recorded() {
        let mut reg = SessionRegistry::new();
        assert_eq!(reg.record_update(&json!({"update": {}})), None);
        assert!(reg.session_ids().is_empty());
    }

    #[test]
    fn ensure_registers_empty_session_once() {
        let mut reg = SessionRegistry::new();
        reg.ensure("s");
        reg.ensure("s");
        assert_eq!(reg.session_ids(), vec!["s".to_string()]);
        assert!(reg.messages("s").is_empty());
        assert!(reg.messages("missing").is_empty());
    }
}
