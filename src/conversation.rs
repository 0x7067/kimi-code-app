//! Pure conversation logic: export formatting, in-conversation search,
//! @mention candidates, and context-usage parsing/thresholds.
//!
//! Everything here is side-effect free so it can be unit-tested natively.

use crate::design_tokens::Colors;
use crate::state::{Item, SessionMeta};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Current Unix time in seconds (webview clock in wasm; system clock natively).
pub fn now_epoch() -> i64 {
    #[cfg(target_arch = "wasm32")]
    {
        (js_sys::Date::now() / 1000.0) as i64
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0)
    }
}

// ---------- Message text / roles ----------

/// Plain text of an item, used for per-message copy and search.
pub fn item_plain_text(item: &Item) -> String {
    match item {
        Item::User(t) | Item::Agent(t) | Item::Thought(t) => t.clone(),
        Item::Tool(tc) => format!("{} {} {}\n{}", tc.kind, tc.title, tc.status, tc.output),
        Item::Cancelled => "cancelled".to_string(),
    }
}

// ---------- Export (F-002.10) ----------

/// Render the whole thread as a Markdown document.
pub fn export_markdown(title: &str, items: &[Item]) -> String {
    let mut out = format!("# {title}\n");
    for item in items {
        match item {
            Item::User(t) => out.push_str(&format!("\n## User\n\n{t}\n")),
            Item::Agent(t) => out.push_str(&format!("\n## Agent\n\n{t}\n")),
            Item::Thought(t) => {
                let quoted = t.lines().collect::<Vec<_>>().join("\n> ");
                out.push_str(&format!("\n## Thinking\n\n> {quoted}\n"));
            }
            Item::Tool(tc) => {
                out.push_str(&format!(
                    "\n## Tool: {} ({}, {})\n",
                    tc.title, tc.kind, tc.status
                ));
                if !tc.output.is_empty() {
                    out.push_str(&format!("\n```\n{}\n```\n", tc.output));
                }
            }
            Item::Cancelled => out.push_str("\n*— turn cancelled —*\n"),
        }
    }
    out
}

/// Render the whole thread as pretty-printed JSON.
pub fn export_json(session_id: &str, items: &[Item]) -> String {
    let msgs: Vec<Value> = items
        .iter()
        .map(|item| match item {
            Item::User(t) => json!({"role": "user", "content": t}),
            Item::Agent(t) => json!({"role": "agent", "content": t}),
            Item::Thought(t) => json!({"role": "thought", "content": t}),
            Item::Tool(tc) => json!({
                "role": "tool",
                "id": tc.id,
                "title": tc.title,
                "kind": tc.kind,
                "status": tc.status,
                "output": tc.output,
            }),
            Item::Cancelled => json!({"role": "marker", "content": "cancelled"}),
        })
        .collect();
    serde_json::to_string_pretty(&json!({"sessionId": session_id, "items": msgs}))
        .unwrap_or_default()
}

/// Build `{session}-{date}.{ext}`, sanitizing the session id for filesystems.
pub fn export_filename(session: &str, date: &str, ext: &str) -> String {
    let safe: String = session
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .collect();
    let safe = safe.trim_matches('-');
    let stem = if safe.is_empty() { "session" } else { safe };
    format!("{stem}-{date}.{ext}")
}

// ---------- Search (F-002.9) ----------

/// Case-insensitive match of a search query against an item's text.
pub fn item_matches(item: &Item, query: &str) -> bool {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return false;
    }
    item_plain_text(item).to_lowercase().contains(&q)
}

// ---------- @mentions (F-002.12 scaffold) ----------

/// If the draft ends in an `@token`, return (byte offset of `@`, query after it).
/// The `@` must start the draft or follow whitespace.
pub fn mention_token(draft: &str) -> Option<(usize, String)> {
    let idx = draft.rfind('@')?;
    let after = &draft[idx + 1..];
    if after.contains(char::is_whitespace) {
        return None;
    }
    if idx > 0 {
        let before = draft[..idx].chars().next_back()?;
        if !before.is_whitespace() {
            return None;
        }
    }
    Some((idx, after.to_string()))
}

/// Filter mention candidates by a case-insensitive substring, capped at 8.
pub fn filter_mentions(candidates: &[String], query: &str) -> Vec<String> {
    let q = query.to_lowercase();
    candidates
        .iter()
        .filter(|c| c.to_lowercase().contains(&q))
        .take(8)
        .cloned()
        .collect()
}

/// Extract candidate file paths from the diff pane text (`files\n\ndiff`).
///
/// TODO(F-002.12): replace with a dedicated backend file-listing command once
/// one exists in `src-tauri/src/commands/` — none does today, so changed files
/// from `git_diff` are the best available data source.
pub fn mention_candidates_from_diff(diff: &str) -> Vec<String> {
    diff.lines()
        .take_while(|l| !l.trim().is_empty())
        .map(str::trim)
        .filter(|l| !l.starts_with("No uncommitted") && !l.contains(' '))
        .map(String::from)
        .collect()
}

// ---------- Message editing (F-002.7) ----------

/// Truncate `items` to `index` (inclusive), replacing the user message at that
/// position with `new_text`. Returns the modified vector.
///
/// Panics if `index` is not a `Item::User` — only user messages are editable.
pub fn edit_and_resend(items: &[Item], index: usize, new_text: &str) -> Vec<Item> {
    let mut out = items[..=index].to_vec();
    out[index] = Item::User(new_text.trim().to_string());
    out
}

// ---------- Pending message queue (F-014) ----------

/// Append a message to the pending queue. Empty/whitespace text is rejected.
/// Returns whether the message was queued.
pub fn queue_push(queue: &mut Vec<String>, text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return false;
    }
    queue.push(trimmed.to_string());
    true
}

/// Remove and return the message at `index` (used both for the chip's ✕ and
/// for click-to-edit, which moves the text into the composer). `None` when
/// the index is out of bounds.
pub fn queue_remove(queue: &mut Vec<String>, index: usize) -> Option<String> {
    if index < queue.len() {
        Some(queue.remove(index))
    } else {
        None
    }
}

/// Pop the oldest queued message (FIFO dispatch on turn end).
pub fn queue_pop_front(queue: &mut Vec<String>) -> Option<String> {
    if queue.is_empty() {
        None
    } else {
        Some(queue.remove(0))
    }
}

// ---------- Relative time (F-012) ----------

/// Days since 1970-01-01 for a civil date (Howard Hinnant's algorithm).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// Parse an RFC 3339 / ISO 8601 timestamp (or a bare epoch number in seconds
/// or milliseconds) into epoch seconds. `None` for unrecognized input.
pub fn parse_timestamp(raw: &str) -> Option<i64> {
    let raw = raw.trim();
    if !raw.is_empty() && raw.chars().all(|c| c.is_ascii_digit()) {
        let n: i64 = raw.parse().ok()?;
        // Heuristic: epoch milliseconds are 13+ digits this century.
        return Some(if n >= 1_000_000_000_000 { n / 1000 } else { n });
    }
    if raw.len() < 19 {
        return None;
    }
    let (date, rest) = raw.split_at(10);
    let mut parts = date.split('-');
    let y: i64 = parts.next()?.parse().ok()?;
    let m: i64 = parts.next()?.parse().ok()?;
    let d: i64 = parts.next()?.parse().ok()?;
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    let time = &rest[1..]; // skip 'T' or ' '
    let h: i64 = time.get(0..2)?.parse().ok()?;
    let min: i64 = time.get(3..5)?.parse().ok()?;
    let s: i64 = time.get(6..8)?.parse().ok()?;
    let mut epoch = days_from_civil(y, m, d) * 86_400 + h * 3600 + min * 60 + s;
    // Apply a trailing numeric offset (fractional seconds are skipped first).
    let tail = &time[8..];
    let tail = tail.strip_prefix('.').map_or(tail, |frac| frac.trim_start_matches(|c: char| c.is_ascii_digit()));
    if let Some(sign) = tail.chars().next() {
        if sign == '+' || sign == '-' {
            let oh: i64 = tail.get(1..3)?.parse().ok()?;
            let om: i64 = tail.get(4..6).and_then(|s| s.parse().ok()).unwrap_or(0);
            let off = oh * 3600 + om * 60;
            epoch -= if sign == '+' { off } else { -off };
        }
    }
    Some(epoch)
}

/// Human relative label for a past timestamp: "just now", "5m ago", "3h ago",
/// "2d ago", "4w ago".
pub fn relative_label(then_epoch: i64, now_epoch: i64) -> String {
    let diff = (now_epoch - then_epoch).max(0);
    match diff {
        0..=59 => "just now".to_string(),
        60..=3_599 => format!("{}m ago", diff / 60),
        3_600..=86_399 => format!("{}h ago", diff / 3600),
        86_400..=604_799 => format!("{}d ago", diff / 86_400),
        _ => format!("{}w ago", diff / 604_800),
    }
}

/// Format a session's `updatedAt` for the sidebar: relative when parseable,
/// otherwise the raw date prefix (or empty).
pub fn format_updated_at(raw: &str, now_epoch: i64) -> String {
    match parse_timestamp(raw) {
        Some(then) => relative_label(then, now_epoch),
        None => raw.get(..10).unwrap_or("").to_string(),
    }
}

// ---------- Context usage (F-002.14 / F-003.12) ----------

/// Color for a context-usage fraction: green < 50%, yellow 50–80%, red > 80%.
pub fn usage_color(frac: f64) -> &'static str {
    if frac < 0.5 {
        Colors::SUCCESS
    } else if frac <= 0.8 {
        Colors::WARNING
    } else {
        Colors::ERROR
    }
}

fn normalize_fraction(n: f64) -> f64 {
    let f = if n > 1.0 { n / 100.0 } else { n };
    f.clamp(0.0, 1.0)
}

fn usage_object_fraction(u: &Value) -> Option<f64> {
    if let Some(n) = u.as_f64() {
        return Some(normalize_fraction(n));
    }
    let used = u
        .get("usedTokens")
        .or_else(|| u.get("used"))
        .or_else(|| u.get("inputTokens"))
        .and_then(Value::as_f64)?;
    let total = u
        .get("totalTokens")
        .or_else(|| u.get("total"))
        .or_else(|| u.get("contextWindow"))
        .and_then(Value::as_f64)?;
    if total <= 0.0 {
        return None;
    }
    Some(normalize_fraction(used / total))
}

/// Pull a context-usage fraction (0.0–1.0) out of an ACP `session/update`
/// payload if present, in any of the shapes agents are known to emit.
pub fn parse_context_usage(params: &Value) -> Option<f64> {
    let update = params.get("update").unwrap_or(params);
    for key in ["contextUsage", "usage", "tokenUsage"] {
        if let Some(u) = update.get(key).or_else(|| params.get(key)) {
            if let Some(frac) = usage_object_fraction(u) {
                return Some(frac);
            }
        }
    }
    None
}

// ---------- Project tree grouping (F-003.10) ----------

/// Group sessions under their project root for the sidebar tree.
///
/// Returns one `(project, sessions)` entry per known project, in project
/// order, plus the sessions whose `cwd` matches no known project (these stay
/// in the flat "recent" list).
pub fn group_sessions_by_project(
    projects: &[String],
    sessions: &[SessionMeta],
) -> (Vec<(String, Vec<SessionMeta>)>, Vec<SessionMeta>) {
    let groups: Vec<(String, Vec<SessionMeta>)> = projects
        .iter()
        .map(|p| {
            let owned: Vec<SessionMeta> =
                sessions.iter().filter(|s| &s.cwd == p).cloned().collect();
            (p.clone(), owned)
        })
        .collect();
    let ungrouped = sessions
        .iter()
        .filter(|s| !projects.contains(&s.cwd))
        .cloned()
        .collect();
    (groups, ungrouped)
}

// ---------- Manual compact gating (F-003.13) ----------

/// Whether the manual "/compact" trigger is currently allowed: requires a
/// connected agent, an active session, and no in-flight turn (compaction is a
/// prompt itself, so it must not race a running turn).
pub fn can_compact(connected: bool, has_session: bool, running: bool) -> bool {
    connected && has_session && !running
}

// ---------- Background running sessions (F-003.14) ----------

/// Record that `session_id` started a turn at `epoch` (global turn counter)
/// and time `now`.
pub fn turn_started(map: &mut HashMap<String, (u64, i64)>, session_id: &str, epoch: u64, now: i64) {
    map.insert(session_id.to_string(), (epoch, now));
}

/// Record that the turn claimed at `epoch` finished. Removal is skipped when
/// the session has since started a newer turn (e.g. a steer superseded it).
pub fn turn_finished(map: &mut HashMap<String, (u64, i64)>, session_id: &str, epoch: u64) {
    if map.get(session_id).is_some_and(|(e, _)| *e == epoch) {
        map.remove(session_id);
    }
}

/// Sessions with a running turn other than `current`, most recent activity
/// first. Each entry is `(session_id, last_activity_epoch_secs)`.
pub fn background_sessions(
    current: Option<&str>,
    map: &HashMap<String, (u64, i64)>,
) -> Vec<(String, i64)> {
    let mut out: Vec<(String, i64)> = map
        .iter()
        .filter(|(sid, _)| Some(sid.as_str()) != current)
        .map(|(sid, (_, at))| (sid.clone(), *at))
        .collect();
    out.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    out
}

// ---------- Cross-process resume conflict guard (F-003) ----------

/// Seconds of wire-log quiet under which a session counts as "active in
/// another process".
pub const RESUME_CONFLICT_THRESHOLD_SECS: u64 = 30;

/// Whether resuming a session should first warn about a likely conflict:
/// its wire log was written within the threshold AND it is not the session
/// this app instance is already driving (our own writes are not a conflict).
pub fn should_warn_resume(age_secs: Option<u64>, is_own_session: bool) -> bool {
    !is_own_session && age_secs.is_some_and(|a| a < RESUME_CONFLICT_THRESHOLD_SECS)
}

// ---------- Settings logic (F-011) ----------

/// F-011.4: whether a send carries the thinking flag. `explicit` is the
/// per-send shortcut (⌘⇧⏎); the default mode governs plain sends:
/// "always" → thinking on, "never"/"ask" → off unless explicit.
pub fn effective_thinking(default_mode: &str, explicit: bool) -> bool {
    explicit || default_mode == "always"
}

/// F-011.5: bucket an ACP tool call into an approval category.
/// `kind` is the ACP ToolKind; `title` disambiguates git commands.
pub fn approval_category(kind: &str, title: &str) -> &'static str {
    let t = title.trim().trim_start_matches('`');
    if t.starts_with("git ") || t == "git" {
        return "git";
    }
    match kind {
        "execute" => "shell",
        "edit" | "delete" | "move" | "read" | "search" => "file-edit",
        _ => "mcp",
    }
}

/// F-011.5/F-011.6: if the request should be auto-approved (YOLO on, or the
/// tool's category preference is "auto"), return the optionId to select —
/// preferring `allow_once` — otherwise None (ask via the modal). YOLO takes
/// precedence over per-category "ask".
pub fn auto_approve_option(
    yolo: bool,
    prefs: &crate::state::ApprovalPrefs,
    kind: &str,
    title: &str,
    options: &[(String, String, String)],
) -> Option<String> {
    let pref = match approval_category(kind, title) {
        "shell" => &prefs.shell,
        "file-edit" => &prefs.file_edit,
        "git" => &prefs.git,
        _ => &prefs.mcp,
    };
    if !yolo && pref != "auto" {
        return None;
    }
    options
        .iter()
        .find(|(_, _, k)| k == "allow_once")
        .or_else(|| options.iter().find(|(_, _, k)| k.starts_with("allow")))
        .map(|(id, _, _)| id.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{ApprovalPrefs, ToolCall};

    // ---------- F-011.4 thinking default ----------

    #[test]
    fn thinking_always_makes_plain_send_thinking() {
        assert!(effective_thinking("always", false));
        assert!(effective_thinking("always", true));
    }

    #[test]
    fn thinking_never_and_ask_follow_explicit_flag() {
        assert!(!effective_thinking("never", false));
        assert!(effective_thinking("never", true)); // explicit shortcut wins
        assert!(!effective_thinking("ask", false));
        assert!(effective_thinking("ask", true));
    }

    // ---------- F-011.5/6 approval preferences ----------

    fn opts() -> Vec<(String, String, String)> {
        vec![
            ("rej".into(), "Reject".into(), "reject_once".into()),
            ("alw".into(), "Always".into(), "allow_always".into()),
            ("once".into(), "Allow".into(), "allow_once".into()),
        ]
    }

    #[test]
    fn approval_categories_map_kind_and_git_title() {
        assert_eq!(approval_category("execute", "ls -la"), "shell");
        assert_eq!(approval_category("execute", "git push"), "git");
        assert_eq!(approval_category("edit", "Edit main.rs"), "file-edit");
        assert_eq!(approval_category("delete", "rm file"), "file-edit");
        assert_eq!(approval_category("fetch", "Fetch docs"), "mcp");
        assert_eq!(approval_category("other", "Some MCP tool"), "mcp");
    }

    #[test]
    fn ask_prefs_show_modal() {
        let prefs = ApprovalPrefs::default();
        assert_eq!(auto_approve_option(false, &prefs, "execute", "ls", &opts()), None);
    }

    #[test]
    fn auto_pref_short_circuits_with_allow_once() {
        let prefs = ApprovalPrefs { shell: "auto".into(), ..Default::default() };
        assert_eq!(
            auto_approve_option(false, &prefs, "execute", "ls", &opts()),
            Some("once".into())
        );
        // other categories still ask
        assert_eq!(auto_approve_option(false, &prefs, "edit", "Edit f", &opts()), None);
    }

    #[test]
    fn yolo_overrides_all_ask_prefs() {
        let prefs = ApprovalPrefs::default();
        for (kind, title) in [("execute", "ls"), ("edit", "e"), ("fetch", "f"), ("execute", "git push")] {
            assert_eq!(
                auto_approve_option(true, &prefs, kind, title, &opts()),
                Some("once".into()),
                "kind={kind}"
            );
        }
    }

    #[test]
    fn auto_falls_back_to_any_allow_option_and_none_without_allow() {
        let only_always = vec![("a".into(), "A".into(), "allow_always".into())];
        let prefs = ApprovalPrefs { git: "auto".into(), ..Default::default() };
        assert_eq!(
            auto_approve_option(false, &prefs, "execute", "git status", &only_always),
            Some("a".into())
        );
        let only_reject: Vec<(String, String, String)> =
            vec![("r".into(), "R".into(), "reject_once".into())];
        assert_eq!(auto_approve_option(true, &prefs, "execute", "ls", &only_reject), None);
    }

    fn tool() -> Item {
        Item::Tool(ToolCall {
            id: "t1".into(),
            title: "Read main.rs".into(),
            kind: "read".into(),
            status: "completed".into(),
            output: "fn main() {}".into(),
        })
    }

    fn thread() -> Vec<Item> {
        vec![
            Item::User("Fix the bug".into()),
            Item::Thought("Looking at it\ncarefully".into()),
            tool(),
            Item::Agent("Done, **fixed**.".into()),
        ]
    }

    #[test]
    fn export_markdown_includes_all_roles() {
        let md = export_markdown("My Session", &thread());
        assert!(md.starts_with("# My Session\n"));
        assert!(md.contains("## User\n\nFix the bug"));
        assert!(md.contains("## Thinking\n\n> Looking at it\n> carefully"));
        assert!(md.contains("## Tool: Read main.rs (read, completed)"));
        assert!(md.contains("```\nfn main() {}\n```"));
        assert!(md.contains("## Agent\n\nDone, **fixed**."));
    }

    #[test]
    fn export_json_roundtrips() {
        let out = export_json("abc-123", &thread());
        let v: Value = serde_json::from_str(&out).expect("valid JSON");
        assert_eq!(v["sessionId"], "abc-123");
        let items = v["items"].as_array().expect("items array");
        assert_eq!(items.len(), 4);
        assert_eq!(items[0]["role"], "user");
        assert_eq!(items[2]["role"], "tool");
        assert_eq!(items[2]["status"], "completed");
        assert_eq!(items[3]["content"], "Done, **fixed**.");
    }

    #[test]
    fn export_filename_sanitizes_session_and_appends_date() {
        assert_eq!(export_filename("sess/1:2", "2026-06-11", "md"), "sess-1-2-2026-06-11.md");
        assert_eq!(export_filename("", "2026-06-11", "json"), "session-2026-06-11.json");
        assert_eq!(export_filename("abc", "2026-06-11", "json"), "abc-2026-06-11.json");
    }

    #[test]
    fn item_matches_is_case_insensitive_and_covers_tools() {
        let items = thread();
        assert!(item_matches(&items[0], "BUG"));
        assert!(item_matches(&items[3], "fixed"));
        assert!(item_matches(&items[2], "main.rs"));
        assert!(!item_matches(&items[0], "nothing"));
        assert!(!item_matches(&items[0], "  "));
    }

    #[test]
    fn mention_token_detects_trailing_at_token() {
        assert_eq!(mention_token("look at @src/ma"), Some((8, "src/ma".into())));
        assert_eq!(mention_token("@"), Some((0, String::new())));
        assert_eq!(mention_token("a@b"), None); // mid-word @ is not a mention
        assert_eq!(mention_token("@file done"), None); // token already closed
        assert_eq!(mention_token("no mentions"), None);
    }

    #[test]
    fn filter_mentions_substring_and_cap() {
        let cands: Vec<String> = (0..20).map(|i| format!("src/file{i}.rs")).collect();
        assert_eq!(filter_mentions(&cands, "file1").len(), 8); // file1, file10..file19 capped
        assert_eq!(filter_mentions(&cands, "FILE19"), vec!["src/file19.rs".to_string()]);
        assert!(filter_mentions(&cands, "zzz").is_empty());
    }

    #[test]
    fn mention_candidates_parsed_from_diff_header() {
        let diff = "src/main.rs\nsrc/lib.rs\n\ndiff --git a/src/main.rs ...";
        assert_eq!(
            mention_candidates_from_diff(diff),
            vec!["src/main.rs".to_string(), "src/lib.rs".to_string()]
        );
        assert!(mention_candidates_from_diff("No uncommitted changes.").is_empty());
    }

    #[test]
    fn queue_preserves_fifo_order_and_rejects_empty() {
        let mut q = Vec::new();
        assert!(queue_push(&mut q, "first"));
        assert!(queue_push(&mut q, "  second  ")); // trimmed
        assert!(!queue_push(&mut q, "   ")); // whitespace-only rejected
        assert_eq!(q, vec!["first".to_string(), "second".to_string()]);
        assert_eq!(queue_pop_front(&mut q), Some("first".to_string()));
        assert_eq!(queue_pop_front(&mut q), Some("second".to_string()));
        assert_eq!(queue_pop_front(&mut q), None);
    }

    #[test]
    fn queue_remove_supports_edit_and_delete_semantics() {
        let mut q = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        // click-to-edit: removal returns the text to load into the composer
        assert_eq!(queue_remove(&mut q, 1), Some("b".to_string()));
        assert_eq!(q, vec!["a".to_string(), "c".to_string()]); // order preserved
        assert_eq!(queue_remove(&mut q, 5), None); // out of bounds is a no-op
        assert_eq!(q.len(), 2);
    }

    #[test]
    fn parse_timestamp_handles_rfc3339_offsets_and_epochs() {
        assert_eq!(parse_timestamp("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(parse_timestamp("2026-06-11T12:00:00Z"), Some(1_781_179_200));
        // Fractional seconds and offsets
        assert_eq!(parse_timestamp("2026-06-11T12:00:00.123Z"), Some(1_781_179_200));
        assert_eq!(parse_timestamp("2026-06-11T14:00:00+02:00"), Some(1_781_179_200));
        assert_eq!(parse_timestamp("2026-06-11T10:00:00-02:00"), Some(1_781_179_200));
        // Bare epoch seconds and milliseconds
        assert_eq!(parse_timestamp("1781179200"), Some(1_781_179_200));
        assert_eq!(parse_timestamp("1781179200000"), Some(1_781_179_200));
        // Garbage
        assert_eq!(parse_timestamp(""), None);
        assert_eq!(parse_timestamp("yesterday"), None);
    }

    #[test]
    fn relative_label_thresholds() {
        let now = 1_781_179_200;
        assert_eq!(relative_label(now - 5, now), "just now");
        assert_eq!(relative_label(now - 300, now), "5m ago");
        assert_eq!(relative_label(now - 2 * 3600, now), "2h ago");
        assert_eq!(relative_label(now - 3 * 86_400, now), "3d ago");
        assert_eq!(relative_label(now - 14 * 86_400, now), "2w ago");
        assert_eq!(relative_label(now + 100, now), "just now"); // clock skew clamps
    }

    #[test]
    fn format_updated_at_falls_back_to_date_prefix() {
        let now = 1_781_179_200;
        assert_eq!(format_updated_at("2026-06-11T11:59:00Z", now), "1m ago");
        assert_eq!(format_updated_at("2026-06-11-broken", now), "2026-06-11");
        assert_eq!(format_updated_at("", now), "");
    }

    #[test]
    fn usage_color_thresholds() {
        assert_eq!(usage_color(0.0), Colors::SUCCESS);
        assert_eq!(usage_color(0.49), Colors::SUCCESS);
        assert_eq!(usage_color(0.5), Colors::WARNING);
        assert_eq!(usage_color(0.8), Colors::WARNING);
        assert_eq!(usage_color(0.81), Colors::ERROR);
        assert_eq!(usage_color(1.0), Colors::ERROR);
    }

    #[test]
    fn parse_context_usage_shapes() {
        // Fraction
        let v = json!({"update": {"contextUsage": 0.42}});
        assert_eq!(parse_context_usage(&v), Some(0.42));
        // Percent normalizes
        let v = json!({"update": {"contextUsage": 75}});
        assert_eq!(parse_context_usage(&v), Some(0.75));
        // used/total token pair
        let v = json!({"usage": {"usedTokens": 50_000, "totalTokens": 200_000}});
        assert_eq!(parse_context_usage(&v), Some(0.25));
        // zero total is ignored
        let v = json!({"usage": {"used": 5, "total": 0}});
        assert_eq!(parse_context_usage(&v), None);
        // absent
        assert_eq!(parse_context_usage(&json!({"update": {"sessionUpdate": "plan"}})), None);
    }

    fn meta(id: &str, cwd: &str) -> SessionMeta {
        SessionMeta { id: id.into(), cwd: cwd.into(), title: id.into(), updated_at: String::new() }
    }

    #[test]
    fn group_sessions_nests_by_project_and_keeps_remainder_flat() {
        let projects = vec!["/a".to_string(), "/b".to_string()];
        let sessions = vec![meta("s1", "/a"), meta("s2", "/c"), meta("s3", "/a"), meta("s4", "/b")];
        let (groups, rest) = group_sessions_by_project(&projects, &sessions);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].0, "/a");
        assert_eq!(
            groups[0].1.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
            vec!["s1", "s3"]
        );
        assert_eq!(groups[1].0, "/b");
        assert_eq!(groups[1].1.len(), 1);
        assert_eq!(rest.len(), 1);
        assert_eq!(rest[0].id, "s2");
    }

    #[test]
    fn group_sessions_with_no_projects_leaves_all_flat() {
        let sessions = vec![meta("s1", "/a"), meta("s2", "/b")];
        let (groups, rest) = group_sessions_by_project(&[], &sessions);
        assert!(groups.is_empty());
        assert_eq!(rest.len(), 2);
    }

    #[test]
    fn can_compact_gating() {
        assert!(can_compact(true, true, false));
        assert!(!can_compact(true, true, true)); // disabled while a turn runs
        assert!(!can_compact(true, false, false)); // needs a session
        assert!(!can_compact(false, true, false)); // needs a connection
    }

    #[test]
    fn running_sessions_track_start_finish_and_supersession() {
        let mut map = HashMap::new();
        turn_started(&mut map, "s1", 1, 100);
        turn_started(&mut map, "s2", 2, 200);
        // A steer on s1 claims a newer epoch before the old turn resolves…
        turn_started(&mut map, "s1", 3, 300);
        // …so the old turn's completion must not clear the running marker.
        turn_finished(&mut map, "s1", 1);
        assert!(map.contains_key("s1"));
        // The current turn's completion does clear it.
        turn_finished(&mut map, "s1", 3);
        assert!(!map.contains_key("s1"));
        turn_finished(&mut map, "s2", 2);
        assert!(map.is_empty());
    }

    #[test]
    fn background_sessions_exclude_current_and_sort_by_recency() {
        let mut map = HashMap::new();
        turn_started(&mut map, "cur", 1, 999);
        turn_started(&mut map, "old", 2, 100);
        turn_started(&mut map, "new", 3, 500);
        let bg = background_sessions(Some("cur"), &map);
        assert_eq!(
            bg,
            vec![("new".to_string(), 500), ("old".to_string(), 100)]
        );
        // With no current session, everything running is "background".
        assert_eq!(background_sessions(None, &map).len(), 3);
    }

    #[test]
    fn should_warn_resume_thresholds_and_own_session_exemption() {
        assert!(should_warn_resume(Some(0), false));
        assert!(should_warn_resume(Some(29), false));
        assert!(!should_warn_resume(Some(30), false)); // at/over threshold: quiet
        assert!(!should_warn_resume(Some(3600), false));
        assert!(!should_warn_resume(None, false)); // unknown activity: no warning
        assert!(!should_warn_resume(Some(0), true)); // our own session never warns
    }

    // ---------- F-002.7 message editing ----------

    #[test]
    fn edit_truncates_items_at_index_and_replaces_text() {
        let items = vec![
            Item::User("hello".into()),
            Item::Agent("hi".into()),
            Item::User("old".into()),
            Item::Agent("response".into()),
        ];
        let result = edit_and_resend(&items, 2, "new");
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], Item::User("hello".into()));
        assert_eq!(result[1], Item::Agent("hi".into()));
        assert_eq!(result[2], Item::User("new".into()));
    }

    #[test]
    fn edit_at_index_zero_replaces_first_message() {
        let items = vec![
            Item::User("first".into()),
            Item::Agent("reply".into()),
        ];
        let result = edit_and_resend(&items, 0, "edited");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Item::User("edited".into()));
    }

    #[test]
    fn edit_last_user_message_truncates_nothing_after_it() {
        let items = vec![
            Item::User("a".into()),
            Item::Agent("b".into()),
            Item::User("c".into()),
        ];
        let result = edit_and_resend(&items, 2, "c2");
        assert_eq!(result.len(), 3);
        assert_eq!(result[2], Item::User("c2".into()));
    }
}

    // ---------- F-007.1 memory preferences ----------

    #[test]
    fn app_settings_defaults_memory_fields_to_empty() {
        let settings = crate::state::AppSettings::default();
        assert_eq!(settings.tech_stack, "");
        assert_eq!(settings.coding_style, "");
        assert_eq!(settings.naming_conventions, "");
    }

    #[test]
    fn app_settings_deserializes_missing_memory_fields_as_empty() {
        let json = serde_json::json!({
            "kimiBinOverride": null,
            "thinkingDefault": "ask",
            "approvals": {"shell": "ask", "fileEdit": "ask", "mcp": "ask", "git": "ask"},
            "yolo": false
        });
        let settings: crate::state::AppSettings = serde_json::from_value(json).unwrap();
        assert_eq!(settings.tech_stack, "");
        assert_eq!(settings.coding_style, "");
        assert_eq!(settings.naming_conventions, "");
    }

    #[test]
    fn app_settings_roundtrips_memory_fields() {
        let settings = crate::state::AppSettings {
            tech_stack: "Rust, TypeScript".into(),
            coding_style: "snake_case, 2-space indent".into(),
            naming_conventions: "PascalCase for components".into(),
            ..crate::state::AppSettings::default()
        };
        let json = serde_json::to_value(&settings).unwrap();
        let restored: crate::state::AppSettings = serde_json::from_value(json).unwrap();
        assert_eq!(restored.tech_stack, "Rust, TypeScript");
        assert_eq!(restored.coding_style, "snake_case, 2-space indent");
        assert_eq!(restored.naming_conventions, "PascalCase for components");
    }
