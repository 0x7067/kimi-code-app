//! Project discovery: recent projects and MCP server config merging.

use crate::paths::kimi_home;
use serde_json::{json, Value};

/// List recently used project directories derived from the global session index.
#[tauri::command]
pub async fn recent_projects() -> Result<Vec<Value>, String> {
    let path = kimi_home().join("session_index.jsonl");
    let content = tokio::fs::read_to_string(&path).await.unwrap_or_default();
    let mut seen = std::collections::HashSet::new();
    let mut projects = Vec::new();
    for line in content.lines().rev() {
        let Ok(v) = serde_json::from_str::<Value>(line) else { continue };
        let Some(cwd) = v
            .get("cwd")
            .or_else(|| v.get("workDir"))
            .or_else(|| v.get("work_dir"))
            .and_then(|c| c.as_str())
        else {
            continue;
        };
        if seen.insert(cwd.to_string()) {
            projects.push(json!({ "path": cwd, "exists": std::path::Path::new(cwd).is_dir() }));
        }
        if projects.len() >= 30 {
            break;
        }
    }
    Ok(projects)
}

/// Resolve the project-instructions file for a project root (F-003.9).
///
/// Checks `AGENTS.md` first, then `CLAUDE.md` (commonly a symlink to
/// AGENTS.md). Symlinks are followed and the resolved target path is
/// returned, so a `CLAUDE.md -> AGENTS.md` link reports the real AGENTS.md.
pub(crate) fn resolve_agents_md(root: &std::path::Path) -> Option<std::path::PathBuf> {
    for name in ["AGENTS.md", "CLAUDE.md"] {
        let candidate = root.join(name);
        // `is_file` follows symlinks, so a CLAUDE.md link to AGENTS.md counts.
        if candidate.is_file() {
            return Some(std::fs::canonicalize(&candidate).unwrap_or(candidate));
        }
    }
    None
}

/// F-003.9 — AGENTS.md auto-detection for the session-creation dialog.
///
/// NOTE: the kimi CLI auto-injects AGENTS.md into the session context itself
/// (server-side) when a session starts in `work_dir`, so the app's job here
/// is DETECTION + PREVIEW only — we never inject the content into prompts.
#[tauri::command]
pub async fn read_agents_md(work_dir: String) -> Result<Option<Value>, String> {
    let Some(path) = resolve_agents_md(std::path::Path::new(&work_dir)) else {
        return Ok(None);
    };
    let content = tokio::fs::read_to_string(&path).await.map_err(|e| e.to_string())?;
    Ok(Some(json!({ "path": path.to_string_lossy(), "content": content })))
}

/// Convert merged mcp.json entries into the ACP `mcpServers` array format.
///
/// F-005: disabled servers and unsupported transports are dropped here —
/// kimi's mcpCapabilities advertise stdio + HTTP only (no SSE, no ACP
/// transport), so anything else must never reach `session/new`.
pub(crate) fn to_acp_servers(merged: std::collections::BTreeMap<String, Value>) -> Vec<Value> {
    let to_pairs = |v: Option<&Value>| -> Vec<Value> {
        v.and_then(|x| x.as_object())
            .map(|o| {
                o.iter()
                    .map(|(k, val)| json!({"name": k, "value": val.as_str().unwrap_or_default()}))
                    .collect()
            })
            .unwrap_or_default()
    };
    merged
        .into_iter()
        .filter_map(|(name, cfg)| {
            if !super::mcp::is_enabled(&cfg) {
                return None;
            }
            match super::mcp::transport_of(&cfg).as_str() {
                "http" => cfg.get("url").and_then(|u| u.as_str()).map(|url| {
                    json!({
                        "type": "http",
                        "name": name,
                        "url": url,
                        "headers": to_pairs(cfg.get("headers")),
                    })
                }),
                "stdio" => cfg.get("command").and_then(|c| c.as_str()).map(|command| {
                    json!({
                        "name": name,
                        "command": command,
                        "args": cfg.get("args").cloned().unwrap_or(json!([])),
                        "env": to_pairs(cfg.get("env")),
                    })
                }),
                _ => None, // sse / acp transports: not supported by kimi
            }
        })
        .collect()
}

/// Merge user-level and project-level mcp.json into the ACP `mcpServers` array format.
#[tauri::command]
pub async fn mcp_servers(cwd: String) -> Result<Vec<Value>, String> {
    let mut merged: std::collections::BTreeMap<String, Value> = Default::default();
    for path in [
        kimi_home().join("mcp.json"),
        std::path::Path::new(&cwd).join(".kimi-code/mcp.json"),
    ] {
        let Ok(content) = tokio::fs::read_to_string(&path).await else { continue };
        let Ok(v) = serde_json::from_str::<Value>(&content) else { continue };
        if let Some(servers) = v.get("mcpServers").and_then(|s| s.as_object()) {
            for (name, cfg) in servers {
                merged.insert(name.clone(), cfg.clone());
            }
        }
    }
    Ok(to_acp_servers(merged))
}

/// Walk `cwd` and return up to `limit` file paths (relative to `cwd`),
/// respecting `.gitignore` and skipping hidden dirs / common build artefacts.
/// `max_depth` defaults to 4 when None.
pub fn list_project_files(cwd: &str, max_depth: Option<usize>, limit: Option<usize>) -> Vec<String> {
    let root = std::path::Path::new(cwd);
    let max_depth = max_depth.unwrap_or(4);
    let limit = limit.unwrap_or(500);
    let gitignore = read_gitignore(root);
    let mut out = Vec::new();
    walk(root, root, 0, max_depth, limit, &gitignore, &mut out);
    out
}

fn read_gitignore(root: &std::path::Path) -> Vec<String> {
    let path = root.join(".gitignore");
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    content
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect()
}

fn walk(
    root: &std::path::Path,
    dir: &std::path::Path,
    depth: usize,
    max_depth: usize,
    limit: usize,
    gitignore: &[String],
    out: &mut Vec<String>,
) {
    if depth >= max_depth || out.len() >= limit {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        if name.starts_with('.') {
            continue; // hidden
        }
        if is_ignored(&name, gitignore) {
            continue;
        }
        if path.is_dir() {
            if name == "node_modules" || name == "target" || name == ".kimi-code" || name == "dist" || name == "build" {
                continue;
            }
            walk(root, &path, depth + 1, max_depth, limit, gitignore, out);
        } else if path.is_file() {
            if let Ok(rel) = path.strip_prefix(root) {
                out.push(rel.to_string_lossy().into_owned());
            }
        }
        if out.len() >= limit {
            break;
        }
    }
}

fn is_ignored(name: &str, gitignore: &[String]) -> bool {
    gitignore.iter().any(|pat| {
        let pat = pat.trim_end_matches('/');
        if pat.starts_with("*") && name.ends_with(&pat[1..]) {
            return true;
        }
        if pat.ends_with("*") && name.starts_with(&pat[..pat.len() - 1]) {
            return true;
        }
        name == pat || name.ends_with(&format!("/{pat}"))
    })
}

#[tauri::command]
pub async fn list_files(cwd: String) -> Result<Vec<String>, String> {
    Ok(list_project_files(&cwd, None, None))
}

// ---------- F-007.2: project memory index ----------

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct FileTreeNode {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<FileTreeNode>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct ProjectIndex {
    pub root: String,
    pub key_files: Vec<String>,
    pub dependencies: Vec<String>,
    pub languages: std::collections::HashMap<String, usize>,
    pub total_files: usize,
    pub total_dirs: usize,
    pub file_tree: Vec<FileTreeNode>,
}

#[derive(Default)]
struct TreeBuilder {
    children: std::collections::BTreeMap<String, TreeBuilder>,
    is_file: bool,
}

fn tree_insert(builder: &mut TreeBuilder, path: &str) {
    let mut current = builder;
    let parts: Vec<&str> = path.split('/').collect();
    for (i, part) in parts.iter().enumerate() {
        current = current.children.entry(part.to_string()).or_default();
        if i == parts.len() - 1 {
            current.is_file = true;
        }
    }
}

fn tree_to_nodes(builder: TreeBuilder) -> Vec<FileTreeNode> {
    let mut nodes: Vec<FileTreeNode> = builder
        .children
        .into_iter()
        .map(|(name, child)| {
            let kind = if child.is_file && child.children.is_empty() {
                "file"
            } else {
                "dir"
            };
            FileTreeNode {
                name,
                kind: kind.to_string(),
                children: if child.children.is_empty() {
                    None
                } else {
                    Some(tree_to_nodes(child))
                },
            }
        })
        .collect();
    nodes.sort_by(|a, b| match (a.kind.as_str(), b.kind.as_str()) {
        ("dir", "file") => std::cmp::Ordering::Less,
        ("file", "dir") => std::cmp::Ordering::Greater,
        _ => a.name.cmp(&b.name),
    });
    nodes
}

fn detect_key_files(files: &[String]) -> Vec<String> {
    let names: std::collections::HashSet<&str> = [
        "Cargo.toml",
        "package.json",
        "pyproject.toml",
        "requirements.txt",
        "go.mod",
        "Gemfile",
        "build.gradle",
        "pom.xml",
        "Makefile",
        "CMakeLists.txt",
        "README.md",
        "README",
        "AGENTS.md",
        "CLAUDE.md",
        "LICENSE",
        "Dockerfile",
        "docker-compose.yml",
        ".gitignore",
    ]
    .iter()
    .cloned()
    .collect();
    files
        .iter()
        .filter(|f| {
            std::path::Path::new(f)
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| names.contains(n))
                .unwrap_or(false)
        })
        .cloned()
        .collect()
}

fn count_languages(files: &[String]) -> std::collections::HashMap<String, usize> {
    let mut map = std::collections::HashMap::new();
    for f in files {
        if let Some(ext) = std::path::Path::new(f).extension().and_then(|e| e.to_str()) {
            *map.entry(ext.to_lowercase()).or_insert(0) += 1;
        }
    }
    map
}

fn parse_cargo_deps(content: &str) -> Vec<String> {
    let mut deps = Vec::new();
    let mut in_deps = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[dependencies]"
            || trimmed == "[dev-dependencies]"
            || trimmed == "[workspace.dependencies]"
            || trimmed.starts_with("[dependencies.")
        {
            in_deps = true;
            continue;
        }
        if trimmed.starts_with('[') && !trimmed.starts_with("[dependencies") {
            in_deps = false;
            continue;
        }
        if in_deps {
            if let Some(name) = trimmed.split('=').next().map(|s| s.trim()) {
                if !name.is_empty() && !name.starts_with('#') {
                    deps.push(name.to_string());
                }
            }
        }
    }
    deps
}

fn parse_package_json_deps(content: &str) -> Vec<String> {
    let mut deps = Vec::new();
    if let Ok(v) = serde_json::from_str::<Value>(content) {
        for key in ["dependencies", "devDependencies", "peerDependencies"] {
            if let Some(obj) = v.get(key).and_then(|d| d.as_object()) {
                for name in obj.keys() {
                    deps.push(name.clone());
                }
            }
        }
    }
    deps
}

fn parse_pyproject_deps(content: &str) -> Vec<String> {
    let mut deps = Vec::new();
    let mut in_project = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[project]" || trimmed == "[project.dependencies]" {
            in_project = true;
            continue;
        }
        if trimmed.starts_with('[') {
            in_project = false;
            continue;
        }
        if in_project && trimmed.starts_with("dependencies") {
            if let Some(arr_start) = trimmed.find('[') {
                let arr_part = &trimmed[arr_start..];
                for item in arr_part.split(',') {
                    let item = item.trim().trim_matches(|c| c == '[' || c == ']' || c == '"');
                    if !item.is_empty() {
                        deps.push(item.to_string());
                    }
                }
            }
        }
    }
    deps
}

fn count_dirs(files: &[String]) -> usize {
    let mut dirs = std::collections::HashSet::new();
    for f in files {
        let p = std::path::Path::new(f);
        if let Some(parent) = p.parent() {
            let mut current = parent;
            while let Some(_name) = current.file_name() {
                dirs.insert(current.to_string_lossy().into_owned());
                current = current.parent().unwrap_or(std::path::Path::new(""));
            }
        }
    }
    dirs.len()
}

pub fn index_project_memory(cwd: &str) -> Result<ProjectIndex, String> {
    let files = list_project_files(cwd, Some(6), Some(2000));
    let key_files = detect_key_files(&files);
    let languages = count_languages(&files);
    let total_dirs = count_dirs(&files);

    let mut dependencies = Vec::new();
    let root = std::path::Path::new(cwd);

    if let Some(cargo) = key_files.iter().find(|f| f.ends_with("Cargo.toml")) {
        let path = root.join(cargo);
        if let Ok(content) = std::fs::read_to_string(&path) {
            dependencies.extend(parse_cargo_deps(&content));
        }
    }
    if let Some(pkg) = key_files.iter().find(|f| f.ends_with("package.json")) {
        let path = root.join(pkg);
        if let Ok(content) = std::fs::read_to_string(&path) {
            dependencies.extend(parse_package_json_deps(&content));
        }
    }
    if let Some(py) = key_files.iter().find(|f| f.ends_with("pyproject.toml")) {
        let path = root.join(py);
        if let Ok(content) = std::fs::read_to_string(&path) {
            dependencies.extend(parse_pyproject_deps(&content));
        }
    }
    dependencies.sort();
    dependencies.dedup();

    let mut builder = TreeBuilder::default();
    for f in &files {
        tree_insert(&mut builder, f);
    }
    let file_tree = tree_to_nodes(builder);

    Ok(ProjectIndex {
        root: cwd.to_string(),
        key_files,
        dependencies,
        languages,
        total_files: files.len(),
        total_dirs,
        file_tree,
    })
}

#[tauri::command]
pub async fn index_project(cwd: String) -> Result<Value, String> {
    let idx = index_project_memory(&cwd)?;
    serde_json::to_value(idx).map_err(|e| e.to_string())
}

#[cfg(test)]
mod project_memory_tests {
    use super::index_project_memory;
    use std::fs;

    fn temp_root(name: &str) -> (tempfile::TempDir, String) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(name);
        fs::create_dir_all(&path).unwrap();
        (dir, path.to_string_lossy().into_owned())
    }

    #[test]
    fn index_detects_cargo_project() {
        let (_tmp, root) = temp_root("cargo");
        fs::write(
            format!("{root}/Cargo.toml"),
            "[package]\nname = \"app\"\n\n[dependencies]\ntokio = \"1\"\nserde = { version = \"1\" }\n",
        )
        .unwrap();
        fs::create_dir_all(format!("{root}/src")).unwrap();
        fs::write(format!("{root}/src/main.rs"), "fn main() {}").unwrap();
        fs::write(format!("{root}/README.md"), "# App").unwrap();

        let idx = index_project_memory(&root).unwrap();
        assert!(idx.key_files.contains(&"Cargo.toml".into()));
        assert!(idx.key_files.contains(&"README.md".into()));
        assert!(idx.dependencies.contains(&"tokio".into()));
        assert!(idx.dependencies.contains(&"serde".into()));
        assert_eq!(idx.languages.get("rs").copied().unwrap_or(0), 1);
        assert_eq!(idx.total_files, 3);
        assert!(idx.total_dirs >= 1);
    }

    #[test]
    fn index_detects_npm_project() {
        let (_tmp, root) = temp_root("npm");
        fs::write(
            format!("{root}/package.json"),
            r#"{"dependencies":{"react":"^18","lodash":"^4"},"devDependencies":{"jest":"^29"}}"#,
        )
        .unwrap();
        fs::create_dir_all(format!("{root}/src")).unwrap();
        fs::write(format!("{root}/src/index.js"), "console.log(1)").unwrap();

        let idx = index_project_memory(&root).unwrap();
        assert!(idx.key_files.contains(&"package.json".into()));
        assert!(idx.dependencies.contains(&"react".into()));
        assert!(idx.dependencies.contains(&"lodash".into()));
        assert!(idx.dependencies.contains(&"jest".into()));
        assert_eq!(idx.languages.get("js").copied().unwrap_or(0), 1);
    }

    #[test]
    fn index_builds_file_tree() {
        let (_tmp, root) = temp_root("tree");
        fs::create_dir_all(format!("{root}/src/components")).unwrap();
        fs::write(format!("{root}/src/main.rs"), "").unwrap();
        fs::write(format!("{root}/src/components/mod.rs"), "").unwrap();

        let idx = index_project_memory(&root).unwrap();
        let src = idx
            .file_tree
            .iter()
            .find(|n| n.name == "src" && n.kind == "dir")
            .expect("src dir in tree");
        let children = src.children.as_ref().unwrap();
        assert!(children.iter().any(|n| n.name == "main.rs" && n.kind == "file"));
        assert!(children.iter().any(|n| n.name == "components" && n.kind == "dir"));
    }

    #[test]
    fn index_respects_gitignore() {
        let (_tmp, root) = temp_root("gitignore");
        fs::write(format!("{root}/.gitignore"), "*.log\nbuild/\n").unwrap();
        fs::write(format!("{root}/keep.txt"), "").unwrap();
        fs::write(format!("{root}/noise.log"), "").unwrap();
        fs::create_dir_all(format!("{root}/build")).unwrap();
        fs::write(format!("{root}/build/out.js"), "").unwrap();

        let idx = index_project_memory(&root).unwrap();
        assert!(idx.file_tree.iter().any(|n| n.name == "keep.txt"));
        assert!(!idx.file_tree.iter().any(|n| n.name == "noise.log"));
        assert!(!idx.file_tree.iter().any(|n| n.name == "build"));
    }

    #[test]
    fn index_counts_languages() {
        let (_tmp, root) = temp_root("langs");
        fs::write(format!("{root}/a.rs"), "").unwrap();
        fs::write(format!("{root}/b.rs"), "").unwrap();
        fs::write(format!("{root}/c.js"), "").unwrap();

        let idx = index_project_memory(&root).unwrap();
        assert_eq!(idx.languages.get("rs").copied().unwrap_or(0), 2);
        assert_eq!(idx.languages.get("js").copied().unwrap_or(0), 1);
    }
}

#[cfg(test)]
mod file_list_tests {
    use super::list_project_files;
    use std::fs;

    fn temp_dir(name: &str) -> (tempfile::TempDir, String) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(name);
        fs::create_dir_all(&path).unwrap();
        (dir, path.to_string_lossy().into_owned())
    }

    #[test]
    fn lists_files_relative_to_root() {
        let (_tmp, root) = temp_dir("list");
        fs::write(format!("{root}/a.txt"), "").unwrap();
        fs::write(format!("{root}/b.rs"), "").unwrap();
        let files = list_project_files(&root, None, None);
        assert!(files.contains(&"a.txt".into()));
        assert!(files.contains(&"b.rs".into()));
    }

    #[test]
    fn respects_max_depth() {
        let (_tmp, root) = temp_dir("depth");
        fs::create_dir_all(format!("{root}/src/deep")).unwrap();
        fs::write(format!("{root}/src/deep/file.txt"), "").unwrap();
        let shallow = list_project_files(&root, Some(2), None);
        assert!(!shallow.contains(&"src/deep/file.txt".into()));
        let deep = list_project_files(&root, Some(3), None);
        assert!(deep.contains(&"src/deep/file.txt".into()));
    }

    #[test]
    fn respects_limit() {
        let (_tmp, root) = temp_dir("limit");
        for i in 0..10 {
            fs::write(format!("{root}/{i}.txt"), "").unwrap();
        }
        let files = list_project_files(&root, None, Some(5));
        assert_eq!(files.len(), 5);
    }

    #[test]
    fn skips_hidden_and_build_dirs() {
        let (_tmp, root) = temp_dir("skip");
        fs::create_dir_all(format!("{root}/.hidden")).unwrap();
        fs::create_dir_all(format!("{root}/node_modules/pkg")).unwrap();
        fs::write(format!("{root}/.hidden/x.txt"), "").unwrap();
        fs::write(format!("{root}/node_modules/pkg/y.js"), "").unwrap();
        fs::write(format!("{root}/visible.txt"), "").unwrap();
        let files = list_project_files(&root, None, None);
        assert!(files.contains(&"visible.txt".into()));
        assert!(!files.iter().any(|f| f.contains(".hidden")));
        assert!(!files.iter().any(|f| f.contains("node_modules")));
    }

    #[test]
    fn respects_gitignore() {
        let (_tmp, root) = temp_dir("gitignore");
        fs::write(format!("{root}/.gitignore"), "*.log\nbuild/\n").unwrap();
        fs::write(format!("{root}/keep.txt"), "").unwrap();
        fs::write(format!("{root}/noise.log"), "").unwrap();
        fs::create_dir_all(format!("{root}/build")).unwrap();
        fs::write(format!("{root}/build/out.js"), "").unwrap();
        let files = list_project_files(&root, None, None);
        assert!(files.contains(&"keep.txt".into()));
        assert!(!files.contains(&"noise.log".into()));
        assert!(!files.iter().any(|f| f.starts_with("build/")));
    }
}

#[cfg(test)]
mod acp_servers_tests {
    use super::to_acp_servers;
    use serde_json::{json, Value};
    use std::collections::BTreeMap;

    fn merged(entries: &[(&str, Value)]) -> BTreeMap<String, Value> {
        entries.iter().map(|(k, v)| (k.to_string(), v.clone())).collect()
    }

    #[test]
    fn stdio_and_http_servers_pass_through() {
        let out = to_acp_servers(merged(&[
            ("files", json!({"command": "mcp-files", "args": ["-r"], "env": {"K": "v"}})),
            ("web", json!({"url": "https://mcp.example.com"})),
        ]));
        assert_eq!(out.len(), 2);
        assert_eq!(out[0]["command"], "mcp-files");
        assert_eq!(out[1]["type"], "http");
        assert_eq!(out[1]["url"], "https://mcp.example.com");
    }

    #[test]
    fn disabled_servers_are_dropped() {
        let out = to_acp_servers(merged(&[
            ("off", json!({"command": "x", "enabled": false})),
            ("on", json!({"command": "y", "enabled": true})),
        ]));
        assert_eq!(out.len(), 1);
        assert_eq!(out[0]["name"], "on");
    }

    #[test]
    fn sse_and_acp_transports_are_dropped() {
        let out = to_acp_servers(merged(&[
            ("sse", json!({"type": "sse", "url": "https://old.example.com"})),
            ("acp", json!({"type": "acp", "command": "agent"})),
            ("ok", json!({"type": "http", "url": "https://x.dev"})),
        ]));
        assert_eq!(out.len(), 1);
        assert_eq!(out[0]["name"], "ok");
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_agents_md;
    use std::fs;
    use std::path::PathBuf;

    fn temp_root(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("kimi-agents-md-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn resolve_agents_md_returns_none_when_absent() {
        let root = temp_root("none");
        assert_eq!(resolve_agents_md(&root), None);
    }

    #[test]
    fn resolve_agents_md_prefers_agents_md() {
        let root = temp_root("prefers");
        fs::write(root.join("AGENTS.md"), "# agents").unwrap();
        fs::write(root.join("CLAUDE.md"), "# claude").unwrap();
        let found = resolve_agents_md(&root).unwrap();
        assert!(found.ends_with("AGENTS.md"), "got {found:?}");
    }

    #[test]
    fn resolve_agents_md_falls_back_to_claude_md() {
        let root = temp_root("fallback");
        fs::write(root.join("CLAUDE.md"), "# claude").unwrap();
        let found = resolve_agents_md(&root).unwrap();
        assert!(found.ends_with("CLAUDE.md"), "got {found:?}");
    }

    #[cfg(unix)]
    #[test]
    fn resolve_agents_md_follows_claude_md_symlink_to_target() {
        let root = temp_root("symlink");
        fs::write(root.join("instructions.md"), "# real").unwrap();
        std::os::unix::fs::symlink(root.join("instructions.md"), root.join("CLAUDE.md")).unwrap();
        let found = resolve_agents_md(&root).unwrap();
        assert!(found.ends_with("instructions.md"), "got {found:?}");
    }

    #[cfg(unix)]
    #[test]
    fn resolve_agents_md_ignores_dangling_symlink() {
        let root = temp_root("dangling");
        std::os::unix::fs::symlink(root.join("missing.md"), root.join("AGENTS.md")).unwrap();
        assert_eq!(resolve_agents_md(&root), None);
    }
}
