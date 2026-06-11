//! Read/write access to the kimi config files in ~/.kimi-code.

use crate::paths::kimi_home;
use serde_json::{json, Value};

const ALLOWED_CONFIGS: &[&str] = &["config.toml", "tui.toml", "mcp.json", "AGENTS.md"];

/// Read a kimi config file (config.toml, tui.toml, mcp.json, AGENTS.md) from ~/.kimi-code.
#[tauri::command]
pub async fn read_kimi_config(name: String) -> Result<String, String> {
    if !ALLOWED_CONFIGS.contains(&name.as_str()) {
        return Err("not an allowed config file".into());
    }
    let path = kimi_home().join(&name);
    match tokio::fs::read_to_string(&path).await {
        Ok(s) => Ok(s),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn write_kimi_config(name: String, content: String) -> Result<(), String> {
    if !ALLOWED_CONFIGS.contains(&name.as_str()) {
        return Err("not an allowed config file".into());
    }
    let home = kimi_home();
    tokio::fs::create_dir_all(&home).await.map_err(|e| e.to_string())?;
    tokio::fs::write(home.join(&name), content)
        .await
        .map_err(|e| e.to_string())
}

// ---------- F-011.3: model selection ----------

/// Parse the `[models."id"]` tables out of config.toml (line-based — enough
/// for kimi's generated config; avoids a toml dependency).
pub fn parse_models(toml: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut current: Option<usize> = None;
    for line in toml.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("[models.") {
            // Accept `[models."id"]` exactly; nested tables like
            // `[models."id".oauth]` have a suffix after the closing quote.
            let id = rest
                .strip_prefix('"')
                .and_then(|r| r.split_once('"'))
                .filter(|(_, tail)| tail.trim() == "]")
                .map(|(id, _)| id.to_string());
            match id.filter(|i| !i.is_empty() && !out.iter().any(|(x, _)| x == i)) {
                Some(id) => {
                    out.push((id, String::new()));
                    current = Some(out.len() - 1);
                }
                None => current = None,
            }
        } else if line.starts_with('[') {
            current = None;
        } else if let (Some(i), Some(eq)) = (current, line.find('=')) {
            let (key, val) = line.split_at(eq);
            if key.trim() == "display_name" {
                out[i].1 = val[1..].trim().trim_matches('"').to_string();
            }
        }
    }
    out
}

/// Parse `default_model = "..."` from config.toml.
pub fn parse_default_model(toml: &str) -> Option<String> {
    toml.lines().find_map(|l| {
        let l = l.trim();
        l.strip_prefix("default_model")
            .and_then(|rest| rest.trim_start().strip_prefix('='))
            .map(|v| v.trim().trim_matches('"').to_string())
    })
}

/// Replace (or insert) the top-level `default_model` line in config.toml.
pub fn set_toml_default_model(toml: &str, model: &str) -> String {
    let line = format!("default_model = \"{model}\"");
    let mut replaced = false;
    let mut out: Vec<String> = toml
        .lines()
        .map(|l| {
            let t = l.trim();
            if !replaced
                && t.strip_prefix("default_model")
                    .map(|r| r.trim_start().starts_with('='))
                    .unwrap_or(false)
            {
                replaced = true;
                line.clone()
            } else {
                l.to_string()
            }
        })
        .collect();
    if !replaced {
        out.insert(0, line);
    }
    let mut s = out.join("\n");
    if toml.ends_with('\n') || toml.is_empty() {
        s.push('\n');
    }
    s
}

/// List the models declared in config.toml plus the current default.
#[tauri::command]
pub async fn list_kimi_models() -> Result<Value, String> {
    let toml = read_kimi_config("config.toml".into()).await?;
    let models: Vec<Value> = parse_models(&toml)
        .into_iter()
        .map(|(id, name)| {
            let display = if name.is_empty() { id.clone() } else { name };
            json!({"id": id, "name": display})
        })
        .collect();
    Ok(json!({"models": models, "default": parse_default_model(&toml)}))
}

/// Persist the default model into config.toml (F-011.3).
#[tauri::command]
pub async fn set_default_model(model: String) -> Result<(), String> {
    let model = model.trim().to_string();
    if model.is_empty() {
        return Err("model id is empty".into());
    }
    let toml = read_kimi_config("config.toml".into()).await?;
    write_kimi_config("config.toml".into(), set_toml_default_model(&toml, &model)).await
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"default_model = "kimi-code/kimi-for-coding"
default_thinking = true

[models."kimi-code/kimi-for-coding"]
provider = "managed:kimi-code"
model = "kimi-for-coding"
display_name = "Kimi-k2.6"

[models."other/k2-5"]
model = "k2.5"

[services.moonshot_search]
base_url = "x"
"#;

    #[test]
    fn parses_models_with_display_names() {
        let models = parse_models(SAMPLE);
        assert_eq!(
            models,
            vec![
                ("kimi-code/kimi-for-coding".into(), "Kimi-k2.6".into()),
                ("other/k2-5".into(), String::new()),
            ]
        );
    }

    #[test]
    fn nested_model_tables_are_not_models() {
        let toml = "[models.\"a/b\"]\nmodel = \"b\"\n[models.\"a/b\".oauth]\nkey = \"x\"\n[models.\"kimi-k2.5\"]\n";
        let models = parse_models(toml);
        assert_eq!(models.len(), 2);
        assert_eq!(models[1].0, "kimi-k2.5"); // dotted ids are real models
    }

    #[test]
    fn parses_default_model() {
        assert_eq!(parse_default_model(SAMPLE).as_deref(), Some("kimi-code/kimi-for-coding"));
        assert_eq!(parse_default_model(""), None);
    }

    #[test]
    fn replaces_existing_default_model_line() {
        let out = set_toml_default_model(SAMPLE, "other/k2-5");
        assert_eq!(parse_default_model(&out).as_deref(), Some("other/k2-5"));
        // everything else intact
        assert!(out.contains("default_thinking = true"));
        assert_eq!(out.matches("default_model").count(), 1);
    }

    #[test]
    fn inserts_default_model_when_missing() {
        let out = set_toml_default_model("default_thinking = true\n", "m/x");
        assert!(out.starts_with("default_model = \"m/x\"\n"));
        assert!(out.contains("default_thinking = true"));
    }
}
