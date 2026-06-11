//! F-007: Cross-session memory store — JSON-backed snippets per project.
//!
//! Memories are persisted in `<app_config_dir>/memories/<project_hash>.json`.
//! Each snippet has an id, content, source session, pinned flag, and timestamp.
//! Retrieval uses simple keyword overlap scoring (no embeddings required).

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::Manager;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MemorySnippet {
    pub id: String,
    pub content: String,
    pub source: String, // session id or "user"
    pub pinned: bool,
    pub created_at: i64,
    pub relevance_score: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MemoryFile {
    pub snippets: Vec<MemorySnippet>,
}

fn project_hash(cwd: &str) -> String {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    use std::hash::{Hash, Hasher};
    cwd.hash(&mut h);
    format!("{:x}", h.finish())
}

fn memory_path(app: &tauri::AppHandle, cwd: &str) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| e.to_string())?
        .join("memories");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join(format!("{}.json", project_hash(cwd))))
}

fn load_file(path: &PathBuf) -> Result<MemoryFile, String> {
    match std::fs::read_to_string(path) {
        Ok(s) => serde_json::from_str(&s).map_err(|e| e.to_string()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(MemoryFile::default()),
        Err(e) => Err(e.to_string()),
    }
}

fn save_file(path: &PathBuf, file: &MemoryFile) -> Result<(), String> {
    let body = serde_json::to_string_pretty(file).map_err(|e| e.to_string())?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, body).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, path).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn list_memories(app: &tauri::AppHandle, cwd: &str) -> Result<Vec<MemorySnippet>, String> {
    let path = memory_path(app, cwd)?;
    let file = load_file(&path)?;
    Ok(file.snippets)
}

pub fn save_memory(
    app: &tauri::AppHandle,
    cwd: &str,
    content: String,
    source: String,
) -> Result<MemorySnippet, String> {
    let path = memory_path(app, cwd)?;
    let mut file = load_file(&path)?;
    let id = format!(
        "mem-{}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis(),
        std::process::id()
    );
    let snippet = MemorySnippet {
        id,
        content,
        source,
        pinned: false,
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
        relevance_score: None,
    };
    file.snippets.push(snippet.clone());
    save_file(&path, &file)?;
    Ok(snippet)
}

pub fn delete_memory(app: &tauri::AppHandle, cwd: &str, id: &str) -> Result<(), String> {
    let path = memory_path(app, cwd)?;
    let mut file = load_file(&path)?;
    file.snippets.retain(|s| s.id != id);
    save_file(&path, &file)?;
    Ok(())
}

pub fn pin_memory(
    app: &tauri::AppHandle,
    cwd: &str,
    id: &str,
    pinned: bool,
) -> Result<(), String> {
    let path = memory_path(app, cwd)?;
    let mut file = load_file(&path)?;
    for s in &mut file.snippets {
        if s.id == id {
            s.pinned = pinned;
            break;
        }
    }
    save_file(&path, &file)?;
    Ok(())
}

/// Simple keyword-overlap retrieval: split query and content into lowercase
/// words, count intersections, normalize by content length. Returns top-k.
pub fn retrieve_memories(
    app: &tauri::AppHandle,
    cwd: &str,
    query: &str,
    top_k: usize,
) -> Result<Vec<MemorySnippet>, String> {
    let path = memory_path(app, cwd)?;
    let file = load_file(&path)?;
    if file.snippets.is_empty() {
        return Ok(Vec::new());
    }
    let query_words: std::collections::HashSet<String> = query
        .to_lowercase()
        .split_whitespace()
        .map(|w| w.to_string())
        .collect();
    let mut scored: Vec<(f64, MemorySnippet)> = file
        .snippets
        .into_iter()
        .map(|mut s| {
            let content_words: std::collections::HashSet<String> = s
                .content
                .to_lowercase()
                .split_whitespace()
                .map(|w| w.to_string())
                .collect();
            let overlap = query_words.intersection(&content_words).count() as f64;
            let score = if content_words.is_empty() {
                0.0
            } else {
                overlap / content_words.len().max(query_words.len()) as f64
            };
            // Boost pinned memories.
            let boosted = if s.pinned { score + 1.0 } else { score };
            s.relevance_score = Some(boosted);
            (boosted, s)
        })
        .collect();
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    Ok(scored.into_iter().take(top_k).map(|(_, s)| s).collect())
}

/// Build a memory injection string from top-k retrieved memories.
pub fn build_memory_context(
    app: &tauri::AppHandle,
    cwd: &str,
    query: &str,
    top_k: usize,
) -> Result<String, String> {
    let memories = retrieve_memories(app, cwd, query, top_k)?;
    if memories.is_empty() {
        return Ok(String::new());
    }
    let mut lines = vec!["## Relevant memories from previous sessions".to_string()];
    for m in memories {
        lines.push(format!("- {}", m.content));
    }
    lines.push(String::new());
    Ok(lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retrieve_ranks_pinned_higher() {
        let snippets = vec![
            MemorySnippet {
                id: "a".into(),
                content: "use dioxus for ui".into(),
                source: "s1".into(),
                pinned: false,
                created_at: 0,
                relevance_score: None,
            },
            MemorySnippet {
                id: "b".into(),
                content: "use react for ui".into(),
                source: "s2".into(),
                pinned: true,
                created_at: 0,
                relevance_score: None,
            },
        ];
        let query = "dioxus ui";
        let query_words: std::collections::HashSet<String> =
            query.to_lowercase().split_whitespace().map(|w| w.to_string()).collect();
        let mut scored: Vec<(f64, &MemorySnippet)> = snippets
            .iter()
            .map(|s| {
                let cw: std::collections::HashSet<String> =
                    s.content.to_lowercase().split_whitespace().map(|w| w.to_string()).collect();
                let overlap = query_words.intersection(&cw).count() as f64;
                let score = overlap / cw.len().max(query_words.len()) as f64;
                let boosted = if s.pinned { score + 1.0 } else { score };
                (boosted, s)
            })
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        assert_eq!(scored[0].1.id, "b"); // pinned wins despite weaker match
        assert_eq!(scored[1].1.id, "a");
    }
}
