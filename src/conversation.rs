//! Pure conversation logic: export formatting, in-conversation search,
//! @mention candidates, and context-usage parsing/thresholds.
//!
//! Everything here is side-effect free so it can be unit-tested natively.

use crate::design_tokens::Colors;
use crate::state::Item;
use serde_json::{json, Value};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::ToolCall;

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
}
