//! Pure parsing of kimi's on-disk session store (F-012). The CLI and this app
//! share `~/.kimi-code/`: `session_index.jsonl` maps sessions to work dirs and
//! each session dir carries a `state.json` with timestamps and a title. No
//! Tauri or process I/O here so everything is unit-testable; the only file
//! system entry point is [`list_sessions`], exercised with tempdir fixtures.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;

/// One line of `session_index.jsonl`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct IndexEntry {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    #[serde(rename = "sessionDir")]
    pub session_dir: String,
    #[serde(rename = "workDir")]
    pub work_dir: String,
}

/// Parse a single index line; `None` for malformed or incomplete lines.
pub fn parse_index_line(line: &str) -> Option<IndexEntry> {
    serde_json::from_str(line.trim()).ok()
}

/// Parse the whole index, skipping malformed lines. Later lines win on
/// duplicate session ids (the CLI appends; the newest record is current).
pub fn parse_index(content: &str) -> Vec<IndexEntry> {
    let mut entries: Vec<IndexEntry> = Vec::new();
    for entry in content.lines().filter_map(parse_index_line) {
        if let Some(slot) = entries.iter_mut().find(|e| e.session_id == entry.session_id) {
            *slot = entry;
        } else {
            entries.push(entry);
        }
    }
    entries
}

/// Keep only entries whose workDir matches `work_dir` (trailing-slash
/// insensitive).
pub fn filter_by_work_dir(entries: Vec<IndexEntry>, work_dir: &str) -> Vec<IndexEntry> {
    let want = work_dir.trim_end_matches('/');
    entries
        .into_iter()
        .filter(|e| e.work_dir.trim_end_matches('/') == want)
        .collect()
}

/// Metadata extracted from a session dir's `state.json`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
pub struct StateMeta {
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,
    pub title: Option<String>,
    #[serde(rename = "lastPrompt")]
    pub last_prompt: Option<String>,
}

/// Parse a `state.json` body; defaults (all `None`) for malformed content.
pub fn parse_state(content: &str) -> StateMeta {
    serde_json::from_str(content).unwrap_or_default()
}

/// A session summary handed to the frontend. `source` (app vs cli) is not
/// recorded in kimi's store, so it is deliberately absent rather than faked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SessionSummary {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    #[serde(rename = "workDir")]
    pub work_dir: String,
    pub title: String,
    #[serde(rename = "createdAt", skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(rename = "updatedAt", skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

/// Title fallback chain: explicit title → first line of the last prompt →
/// trailing segment of the session id.
pub fn derive_title(state: &StateMeta, session_id: &str) -> String {
    if let Some(t) = state.title.as_deref().map(str::trim).filter(|t| !t.is_empty()) {
        return t.to_string();
    }
    if let Some(p) = state
        .last_prompt
        .as_deref()
        .and_then(|p| p.lines().next())
        .map(str::trim)
        .filter(|p| !p.is_empty())
    {
        return p.to_string();
    }
    session_id
        .rsplit(['_', '-'])
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or(session_id)
        .to_string()
}

/// Combine an index entry with its (possibly missing) state metadata.
pub fn summarize(entry: &IndexEntry, state: &StateMeta) -> SessionSummary {
    SessionSummary {
        session_id: entry.session_id.clone(),
        work_dir: entry.work_dir.clone(),
        title: derive_title(state, &entry.session_id),
        created_at: state.created_at.clone(),
        updated_at: state.updated_at.clone(),
    }
}

/// Sort newest-first by `updatedAt` (RFC 3339 strings sort lexicographically);
/// sessions without a timestamp sink to the end.
pub fn sort_by_updated_desc(sessions: &mut [SessionSummary]) {
    sessions.sort_by(|a, b| match (&b.updated_at, &a.updated_at) {
        (Some(b), Some(a)) => b.cmp(a),
        (Some(_), None) => std::cmp::Ordering::Greater,
        (None, Some(_)) => std::cmp::Ordering::Less,
        (None, None) => std::cmp::Ordering::Equal,
    });
}

/// Normalize a `session/list` ACP result into a session array. Accepts either
/// a bare array or an object wrapping it under `sessions`; `None` when the
/// result carries no recognizable session list (caller falls back to the
/// on-disk store).
pub fn sessions_from_list_result(result: &Value) -> Option<Vec<Value>> {
    match result {
        Value::Array(items) => Some(items.clone()),
        Value::Object(obj) => obj.get("sessions")?.as_array().cloned(),
        _ => None,
    }
}

/// Find the on-disk session dir for `session_id` in index content (latest
/// index record wins, matching `parse_index` dedup semantics).
pub fn session_dir_for(content: &str, session_id: &str) -> Option<String> {
    parse_index(content)
        .into_iter()
        .find(|e| e.session_id == session_id)
        .map(|e| e.session_dir)
}

/// Read the store under `home` and list sessions for `work_dir`, enriched
/// from each session dir's `state.json`, sorted by `updatedAt` desc.
pub fn list_sessions(home: &Path, work_dir: &str) -> Vec<SessionSummary> {
    let content = std::fs::read_to_string(home.join("session_index.jsonl")).unwrap_or_default();
    let mut out: Vec<SessionSummary> = filter_by_work_dir(parse_index(&content), work_dir)
        .iter()
        .map(|entry| {
            let state = std::fs::read_to_string(Path::new(&entry.session_dir).join("state.json"))
                .map(|s| parse_state(&s))
                .unwrap_or_default();
            summarize(entry, &state)
        })
        .collect();
    sort_by_updated_desc(&mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str, dir: &str, wd: &str) -> IndexEntry {
        IndexEntry {
            session_id: id.into(),
            session_dir: dir.into(),
            work_dir: wd.into(),
        }
    }

    #[test]
    fn parses_valid_index_line() {
        let line = r#"{"sessionId":"session_abc","sessionDir":"/h/.kimi-code/sessions/wd_x/session_abc","workDir":"/Users/pedro/proj"}"#;
        assert_eq!(
            parse_index_line(line),
            Some(entry(
                "session_abc",
                "/h/.kimi-code/sessions/wd_x/session_abc",
                "/Users/pedro/proj"
            ))
        );
    }

    #[test]
    fn rejects_malformed_index_lines() {
        assert_eq!(parse_index_line(""), None);
        assert_eq!(parse_index_line("not json"), None);
        assert_eq!(parse_index_line(r#"{"sessionId":"x"}"#), None); // missing fields
    }

    #[test]
    fn parse_index_skips_bad_lines_and_dedupes_keeping_last() {
        let content = "\
{\"sessionId\":\"a\",\"sessionDir\":\"/d/a\",\"workDir\":\"/w1\"}\n\
garbage\n\
{\"sessionId\":\"b\",\"sessionDir\":\"/d/b\",\"workDir\":\"/w2\"}\n\
{\"sessionId\":\"a\",\"sessionDir\":\"/d/a2\",\"workDir\":\"/w1\"}\n";
        let entries = parse_index(content);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0], entry("a", "/d/a2", "/w1"));
        assert_eq!(entries[1], entry("b", "/d/b", "/w2"));
    }

    #[test]
    fn session_dir_for_finds_latest_record_or_none() {
        let content = "\
{\"sessionId\":\"a\",\"sessionDir\":\"/d/a\",\"workDir\":\"/w1\"}\n\
{\"sessionId\":\"a\",\"sessionDir\":\"/d/a2\",\"workDir\":\"/w1\"}\n";
        assert_eq!(session_dir_for(content, "a"), Some("/d/a2".to_string()));
        assert_eq!(session_dir_for(content, "missing"), None);
    }

    #[test]
    fn filters_by_work_dir_ignoring_trailing_slash() {
        let entries = vec![
            entry("a", "/d/a", "/w/proj"),
            entry("b", "/d/b", "/w/proj/"),
            entry("c", "/d/c", "/w/other"),
        ];
        let kept = filter_by_work_dir(entries, "/w/proj/");
        assert_eq!(kept.iter().map(|e| e.session_id.as_str()).collect::<Vec<_>>(), ["a", "b"]);
    }

    #[test]
    fn parses_state_json_fields() {
        let state = parse_state(
            r#"{"createdAt":"2026-06-10T15:31:12.647Z","updatedAt":"2026-06-10T15:31:12.743Z","title":"Hello","isCustomTitle":false,"lastPrompt":"Hello\nWorld"}"#,
        );
        assert_eq!(state.created_at.as_deref(), Some("2026-06-10T15:31:12.647Z"));
        assert_eq!(state.updated_at.as_deref(), Some("2026-06-10T15:31:12.743Z"));
        assert_eq!(state.title.as_deref(), Some("Hello"));
        assert_eq!(state.last_prompt.as_deref(), Some("Hello\nWorld"));
    }

    #[test]
    fn malformed_state_json_yields_defaults() {
        assert_eq!(parse_state("nope"), StateMeta::default());
        assert_eq!(parse_state(""), StateMeta::default());
    }

    #[test]
    fn title_falls_back_to_prompt_first_line_then_id_suffix() {
        let id = "session_688d3a4a-0edb-43fd-81c1-85f551f47024";
        let full = StateMeta {
            title: Some("My title".into()),
            last_prompt: Some("prompt line".into()),
            ..Default::default()
        };
        assert_eq!(derive_title(&full, id), "My title");

        let prompt_only = StateMeta {
            title: Some("   ".into()),
            last_prompt: Some("first line\nsecond line".into()),
            ..Default::default()
        };
        assert_eq!(derive_title(&prompt_only, id), "first line");

        assert_eq!(derive_title(&StateMeta::default(), id), "85f551f47024");
    }

    #[test]
    fn sorts_by_updated_at_desc_with_missing_last() {
        fn s(id: &str, updated: Option<&str>) -> SessionSummary {
            SessionSummary {
                session_id: id.into(),
                work_dir: "/w".into(),
                title: id.into(),
                created_at: None,
                updated_at: updated.map(String::from),
            }
        }
        let mut v = vec![
            s("old", Some("2026-06-01T00:00:00.000Z")),
            s("none", None),
            s("new", Some("2026-06-11T00:00:00.000Z")),
        ];
        sort_by_updated_desc(&mut v);
        let ids: Vec<&str> = v.iter().map(|s| s.session_id.as_str()).collect();
        assert_eq!(ids, ["new", "old", "none"]);
    }

    #[test]
    fn lists_sessions_from_a_store_fixture() {
        let home = tempfile::tempdir().unwrap();
        let mk = |id: &str, state: Option<&str>| -> String {
            let dir = home.path().join("sessions").join(id);
            std::fs::create_dir_all(&dir).unwrap();
            if let Some(s) = state {
                std::fs::write(dir.join("state.json"), s).unwrap();
            }
            dir.to_string_lossy().into_owned()
        };
        let d1 = mk(
            "session_one",
            Some(r#"{"createdAt":"2026-06-01T00:00:00Z","updatedAt":"2026-06-01T00:00:00Z","title":"First"}"#),
        );
        let d2 = mk(
            "session_two",
            Some(r#"{"createdAt":"2026-06-09T00:00:00Z","updatedAt":"2026-06-10T00:00:00Z","title":"","lastPrompt":"do the thing\nplease"}"#),
        );
        let d3 = mk("session_other", None); // different workDir, must be filtered
        let index = format!(
            "{}\n{}\n{}\n",
            serde_json::json!({"sessionId": "session_one", "sessionDir": d1, "workDir": "/w/proj"}),
            serde_json::json!({"sessionId": "session_two", "sessionDir": d2, "workDir": "/w/proj"}),
            serde_json::json!({"sessionId": "session_other", "sessionDir": d3, "workDir": "/w/other"}),
        );
        std::fs::write(home.path().join("session_index.jsonl"), index).unwrap();

        let sessions = list_sessions(home.path(), "/w/proj");
        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0].session_id, "session_two"); // newest first
        assert_eq!(sessions[0].title, "do the thing");
        assert_eq!(sessions[1].session_id, "session_one");
        assert_eq!(sessions[1].title, "First");
        assert_eq!(sessions[1].updated_at.as_deref(), Some("2026-06-01T00:00:00Z"));
    }

    #[test]
    fn normalizes_session_list_results() {
        use serde_json::json;
        let s = json!({"sessionId": "a"});
        assert_eq!(sessions_from_list_result(&json!([s])), Some(vec![s.clone()]));
        assert_eq!(
            sessions_from_list_result(&json!({"sessions": [s]})),
            Some(vec![s])
        );
        assert_eq!(sessions_from_list_result(&json!({"unexpected": true})), None);
        assert_eq!(sessions_from_list_result(&json!(null)), None);
    }

    #[test]
    fn empty_store_lists_nothing() {
        let home = tempfile::tempdir().unwrap();
        assert!(list_sessions(home.path(), "/w/proj").is_empty());
    }
}
