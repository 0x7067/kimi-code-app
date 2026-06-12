//! Development verification mode helpers.

use crate::state::{
    AppSettings, Item, PermissionRequest, PlanEntry, SessionMeta, SlashCommand, ToolCall, View, AGENT_INFO,
    APP_SETTINGS, CHECKPOINTS, COMMANDS, CONNECTED, CONTEXT_USAGE, DIFF, ERROR, ITEMS, MEMORY_SNIPPETS,
    PERMISSION, PLAN, PROJECT, PROJECT_FILES, RECENT_PROJECTS, RUNNING, SESSIONS, SESSION_ID,
    SHOW_AUTOMATIONS, SHOW_BROWSER, SHOW_DIFF, SHOW_MEMORY, SHOW_MULTI_AGENT, TERMINAL_OPEN, VIEW,
};
use dioxus::prelude::ReadableExt;
use js_sys::{eval, Reflect};
use serde_json::json;
use serde_json::Value;
use wasm_bindgen::JsValue;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerifyPanel {
    Terminal,
    Memory,
    Browser,
    Diff,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerifyScenario {
    Empty,
    ActiveSession,
    PermissionRequest,
    LongThread,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifyConfig {
    pub scenario: VerifyScenario,
    pub panels: Vec<VerifyPanel>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VerifyFixture {
    pub connected: bool,
    pub agent_info: String,
    pub project: Option<String>,
    pub recent_projects: Vec<String>,
    pub sessions: Vec<SessionMeta>,
    pub session_id: Option<String>,
    pub items: Vec<Item>,
    pub plan: Vec<PlanEntry>,
    pub commands: Vec<SlashCommand>,
    pub permission: Option<PermissionRequest>,
    pub memories: Vec<Value>,
    pub project_files: Vec<String>,
    pub running: bool,
}

fn parse_scenario(value: &str) -> VerifyScenario {
    match value {
        "empty" => VerifyScenario::Empty,
        "permission-request" => VerifyScenario::PermissionRequest,
        "long-thread" => VerifyScenario::LongThread,
        "active-session" => VerifyScenario::ActiveSession,
        _ => VerifyScenario::ActiveSession,
    }
}

fn parse_panel(value: &str) -> Option<VerifyPanel> {
    match value {
        "terminal" => Some(VerifyPanel::Terminal),
        "memory" => Some(VerifyPanel::Memory),
        "browser" => Some(VerifyPanel::Browser),
        "diff" => Some(VerifyPanel::Diff),
        _ => None,
    }
}

pub fn verify_config_from_search(search: &str) -> Option<VerifyConfig> {
    let query = search.strip_prefix('?').unwrap_or(search);
    let mut verify = false;
    let mut scenario = VerifyScenario::ActiveSession;
    let mut panels = Vec::new();

    for pair in query.split('&').filter(|pair| !pair.is_empty()) {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        match key {
            "verify" if value == "1" || value == "true" => verify = true,
            "scenario" => scenario = parse_scenario(value),
            "panel" => {
                panels.extend(value.split(',').filter_map(parse_panel));
            }
            _ => {}
        }
    }

    verify.then_some(VerifyConfig { scenario, panels })
}

pub fn fixture_for_config(config: &VerifyConfig) -> VerifyFixture {
    let project = "/tmp/kimi-verify-project".to_string();
    let session = SessionMeta {
        id: "verify-session".to_string(),
        cwd: project.clone(),
        title: "Verify app controls".to_string(),
        updated_at: "2026-06-12T12:00:00Z".to_string(),
    };
    let mut fixture = VerifyFixture {
        connected: true,
        agent_info: "Kimi Verify 0.0.0".to_string(),
        project: Some(project.clone()),
        recent_projects: vec![project.clone(), "/tmp/kimi-verify-side-project".to_string()],
        sessions: vec![session],
        session_id: Some("verify-session".to_string()),
        items: vec![
            Item::User("Open the verification surface.".to_string()),
            Item::Agent(
                "This deterministic verification thread is ready for browser-driven checks.".to_string(),
            ),
        ],
        plan: vec![
            PlanEntry {
                content: "Expose a stable browser verification target".to_string(),
                priority: "high".to_string(),
                status: "completed".to_string(),
            },
            PlanEntry {
                content: "Drive panels and modal states from fixtures".to_string(),
                priority: "medium".to_string(),
                status: "in_progress".to_string(),
            },
        ],
        commands: vec![
            SlashCommand {
                name: "compact".to_string(),
                description: "Compact the current context".to_string(),
            },
            SlashCommand {
                name: "help".to_string(),
                description: "Show available Kimi commands".to_string(),
            },
        ],
        permission: None,
        memories: vec![
            json!({
                "id": "verify-memory-1",
                "content": "Use Browser against ?verify=1 for UI smoke checks.",
                "source": "verify",
                "createdAt": 1781230400_i64,
                "pinned": true,
                "relevanceScore": 0.98
            }),
            json!({
                "id": "verify-memory-2",
                "content": "Selectors should use data-testid for stable automation.",
                "source": "verify",
                "createdAt": 1781230450_i64,
                "pinned": false,
                "relevanceScore": 0.86
            }),
        ],
        project_files: vec![
            "src/main.rs".to_string(),
            "src/components/app.rs".to_string(),
            "src/verify.rs".to_string(),
            "assets/css/03-layout.css".to_string(),
        ],
        running: false,
    };

    match config.scenario {
        VerifyScenario::Empty => {
            fixture.session_id = None;
            fixture.items.clear();
            fixture.plan.clear();
        }
        VerifyScenario::ActiveSession => {}
        VerifyScenario::LongThread => {
            for i in 1..=12 {
                fixture.items.push(Item::User(format!("Verify prompt {i}")));
                fixture.items.push(Item::Agent(format!(
                    "Verify response {i} with enough content to exercise scrolling."
                )));
            }
        }
        VerifyScenario::PermissionRequest => {
            fixture.running = true;
            fixture.items.push(Item::Tool(ToolCall {
                id: "verify-tool-1".to_string(),
                title: "Run cargo check".to_string(),
                kind: "shell".to_string(),
                status: "in_progress".to_string(),
                output: "cargo check --workspace".to_string(),
            }));
            fixture.permission = Some(PermissionRequest {
                request_id: 42,
                title: "Run cargo check".to_string(),
                detail: "{\n  \"command\": \"cargo check --workspace\"\n}".to_string(),
                options: vec![
                    ("allow".to_string(), "Allow".to_string(), "allow_once".to_string()),
                    ("deny".to_string(), "Deny".to_string(), "reject_once".to_string()),
                ],
            });
        }
    }

    fixture
}

pub fn mock_invoke_for_fixture(
    fixture: &VerifyFixture,
    cmd: &str,
    args: &Value,
) -> Option<Result<Value, Value>> {
    let sessions = fixture.sessions.iter().map(session_to_json).collect::<Vec<_>>();
    let project = fixture
        .project
        .clone()
        .unwrap_or_else(|| "/tmp/kimi-verify-project".to_string());
    let session_id = fixture
        .session_id
        .clone()
        .unwrap_or_else(|| "verify-session".to_string());
    let value = match cmd {
        "acp_connect" => json!({
            "agentInfo": {
                "name": "Kimi Verify",
                "version": "0.0.0",
            }
        }),
        "recent_projects" => json!(fixture
            .recent_projects
            .iter()
            .map(|path| json!({"path": path, "exists": true}))
            .collect::<Vec<_>>()),
        "read_app_settings" => serde_json::to_value(AppSettings::default()).unwrap_or_else(|_| json!({})),
        "write_app_settings"
        | "acp_cancel"
        | "acp_respond_permission"
        | "term_write"
        | "term_close"
        | "start_browser_watcher"
        | "stop_browser_watcher"
        | "save_memory"
        | "delete_memory"
        | "pin_memory" => {
            json!({})
        }
        "kimi_list_sessions" => json!(sessions),
        "kimi_load_session" => json!({
            "sessionId": args
                .get("sessionId")
                .and_then(|value| value.as_str())
                .unwrap_or(&session_id),
        }),
        "kimi_session_activity" => json!(0),
        "mcp_servers" => json!([]),
        "git_diff" => json!({
            "files": ["src/verify.rs", "src/components/app.rs"],
            "diff": "diff --git a/src/verify.rs b/src/verify.rs\n+verify mode fixture\n",
        }),
        "list_files" => json!(fixture.project_files),
        "list_memories" => json!(fixture.memories),
        "index_project" => json!({
            "totalFiles": 42,
            "totalDirs": 8,
            "keyFiles": ["src/main.rs", "src/components/app.rs", "src/verify.rs"],
            "dependencies": ["dioxus", "tauri", "serde_json"],
            "languages": {"Rust": 42},
        }),
        "build_memory_context" => json!("- Use verify mode fixtures for deterministic checks."),
        "read_agents_md" => json!({
            "path": format!("{project}/AGENTS.md"),
            "content": "# Verify AGENTS\nDeterministic fixture.",
        }),
        "pick_folder" => json!(project),
        "pick_image" => json!({
            "name": "verify.png",
            "mimeType": "image/png",
            "data": "",
        }),
        "term_open" => json!({"id": 1}),
        "acp_steer" => json!({"stopReason": "end_turn"}),
        "acp_request" => match args.get("method").and_then(|value| value.as_str()) {
            Some("session/list") => json!({"sessions": sessions}),
            Some("session/new") => json!({"sessionId": session_id}),
            Some("session/prompt") => json!({"stopReason": "end_turn"}),
            _ => json!({}),
        },
        _ => return None,
    };
    Some(Ok(value))
}

fn current_location_search() -> Option<String> {
    let location = Reflect::get(&js_sys::global(), &JsValue::from_str("location")).ok()?;
    Reflect::get(&location, &JsValue::from_str("search"))
        .ok()?
        .as_string()
}

pub fn mock_invoke_from_location(cmd: &str, args: &Value) -> Option<Result<Value, Value>> {
    let config = current_location_search().and_then(|search| verify_config_from_search(&search))?;
    let fixture = fixture_for_config(&config);
    mock_invoke_for_fixture(&fixture, cmd, args)
}

pub fn is_verify_mode_from_location() -> bool {
    current_location_search()
        .and_then(|search| verify_config_from_search(&search))
        .is_some()
}

fn session_to_json(session: &SessionMeta) -> Value {
    json!({
        "sessionId": session.id,
        "cwd": session.cwd,
        "workDir": session.cwd,
        "title": session.title,
        "updatedAt": session.updated_at,
    })
}

fn fixture_json(fixture: &VerifyFixture) -> Value {
    json!({
        "connected": fixture.connected,
        "agentInfo": {
            "name": fixture.agent_info.split_whitespace().next().unwrap_or("Kimi"),
            "version": fixture.agent_info.split_whitespace().skip(1).collect::<Vec<_>>().join(" "),
        },
        "project": fixture.project,
        "recentProjects": fixture.recent_projects.iter().map(|path| json!({
            "path": path,
            "exists": true,
        })).collect::<Vec<_>>(),
        "sessions": fixture.sessions.iter().map(session_to_json).collect::<Vec<_>>(),
        "sessionId": fixture.session_id,
        "commands": fixture.commands.iter().map(|cmd| json!({
            "name": cmd.name,
            "description": cmd.description,
        })).collect::<Vec<_>>(),
        "memories": fixture.memories,
        "projectFiles": fixture.project_files,
        "settings": serde_json::to_value(AppSettings::default()).unwrap_or_else(|_| json!({})),
        "diff": {
            "files": ["src/verify.rs", "src/components/app.rs"],
            "diff": "diff --git a/src/verify.rs b/src/verify.rs\n+verify mode fixture\n",
        },
        "index": {
            "totalFiles": 42,
            "totalDirs": 8,
            "keyFiles": ["src/main.rs", "src/components/app.rs", "src/verify.rs"],
            "dependencies": ["dioxus", "tauri", "serde_json"],
            "languages": {"Rust": 42},
        },
    })
}

fn apply_fixture(config: &VerifyConfig, fixture: &VerifyFixture) {
    *CONNECTED.write() = fixture.connected;
    *AGENT_INFO.write() = fixture.agent_info.clone();
    *PROJECT.write() = fixture.project.clone();
    *RECENT_PROJECTS.write() = fixture.recent_projects.clone();
    *SESSIONS.write() = fixture.sessions.clone();
    *SESSION_ID.write() = fixture.session_id.clone();
    *ITEMS.write() = fixture.items.clone();
    *PLAN.write() = fixture.plan.clone();
    *COMMANDS.write() = fixture.commands.clone();
    *PERMISSION.write() = fixture.permission.clone();
    *MEMORY_SNIPPETS.write() = fixture.memories.clone();
    *PROJECT_FILES.write() = fixture.project_files.clone();
    *RUNNING.write() = fixture.running;
    *APP_SETTINGS.write() = AppSettings::default();
    *ERROR.write() = None;
    *CONTEXT_USAGE.write() = if fixture.running { 0.72 } else { 0.38 };
    *CHECKPOINTS.write() = vec![json!({
        "name": "verify-start",
        "savedAt": "2026-06-12T12:00:00Z",
    })];
    *VIEW.write() = View::Chat;
    *TERMINAL_OPEN.write() = config.panels.contains(&VerifyPanel::Terminal);
    *SHOW_MEMORY.write() = config.panels.contains(&VerifyPanel::Memory);
    *SHOW_BROWSER.write() = config.panels.contains(&VerifyPanel::Browser);
    *SHOW_DIFF.write() = config.panels.contains(&VerifyPanel::Diff);
    *SHOW_AUTOMATIONS.write() = false;
    *SHOW_MULTI_AGENT.write() = false;
    *DIFF.write() = "src/verify.rs\nsrc/components/app.rs\n\ndiff --git a/src/verify.rs b/src/verify.rs\n+verify mode fixture\n".to_string();
}

fn install_mock_bridge(config: &VerifyConfig, fixture: &VerifyFixture) {
    let fixture_json = serde_json::to_string(&fixture_json(fixture)).unwrap_or_else(|_| "{}".to_string());
    let scenario = format!("{:?}", config.scenario);
    let script = format!(
        r##"
(() => {{
  const fixture = {fixture_json};
  const listeners = new Map();
  const emit = (event, payload) => {{
    for (const cb of listeners.get(event) || []) {{
      cb({{ event, payload }});
    }}
  }};
  const firstSession = () => fixture.sessions[0] || {{
    sessionId: "verify-session",
    cwd: fixture.project || "/tmp/kimi-verify-project",
    workDir: fixture.project || "/tmp/kimi-verify-project",
    title: "Verify session",
    updatedAt: "2026-06-12T12:00:00Z"
  }};
  const ok = (value) => Promise.resolve(value);
  window.__KIMI_VERIFY_FIXTURE__ = fixture;
  window.__KIMI_DEBUG__ = {{
    emit,
    fixture,
    snapshot() {{
      const state = window.__KIMI_DEBUG_STATE__ || {{}};
      return {{
        ...state,
        verifyMode: true,
        scenario: "{scenario}",
        dom: {{
          shell: !!document.querySelector('[data-testid="app-shell"]'),
          composer: !!document.querySelector('[data-testid="composer-input"]'),
          permissionModal: !!document.querySelector('[data-testid="permission-modal"]'),
          terminal: !!document.querySelector('[data-testid="terminal-pane"]'),
          memory: !!document.querySelector('[data-testid="memory-pane"]'),
          browser: !!document.querySelector('[data-testid="browser-pane"]'),
          diff: !!document.querySelector('[data-testid="diff-pane"]')
        }}
      }};
    }}
  }};
  window.__TAURI__ = {{
    core: {{
      invoke(cmd, args = {{}}) {{
        switch (cmd) {{
          case "acp_connect":
            return ok({{ agentInfo: fixture.agentInfo }});
          case "recent_projects":
            return ok(fixture.recentProjects);
          case "read_app_settings":
            return ok(fixture.settings);
          case "write_app_settings":
          case "acp_cancel":
          case "acp_respond_permission":
          case "term_write":
          case "term_close":
          case "start_browser_watcher":
          case "stop_browser_watcher":
          case "save_memory":
          case "delete_memory":
          case "pin_memory":
            return ok({{}});
          case "kimi_list_sessions":
            return ok(fixture.sessions);
          case "kimi_load_session":
            return ok({{ sessionId: args.sessionId || firstSession().sessionId }});
          case "kimi_session_activity":
            return ok(0);
          case "mcp_servers":
            return ok([]);
          case "git_diff":
            return ok(fixture.diff);
          case "list_files":
            return ok(fixture.projectFiles);
          case "list_memories":
            return ok(fixture.memories);
          case "index_project":
            return ok(fixture.index);
          case "build_memory_context":
            return ok("- Use verify mode fixtures for deterministic checks.");
          case "read_agents_md":
            return ok({{ path: `${{args.workDir || fixture.project}}/AGENTS.md`, content: "# Verify AGENTS\nDeterministic fixture." }});
          case "pick_folder":
            return ok(fixture.project || "/tmp/kimi-verify-project");
          case "pick_image":
            return ok({{ name: "verify.png", mimeType: "image/png", data: "" }});
          case "term_open":
            setTimeout(() => emit("term:output", {{ id: 1, data: "$ verify mode\nready\n" }}), 20);
            return ok({{ id: 1 }});
          case "acp_steer":
            emit("acp:update", {{ sessionId: args.sessionId, update: {{ sessionUpdate: "agent_message_chunk", content: "Steered in verify mode." }} }});
            return ok({{ stopReason: "end_turn" }});
          case "acp_request":
            if (args.method === "session/list") return ok({{ sessions: fixture.sessions }});
            if (args.method === "session/new") return ok({{ sessionId: firstSession().sessionId }});
            if (args.method === "session/prompt") {{
              emit("acp:update", {{ sessionId: args.params?.sessionId || firstSession().sessionId, update: {{ sessionUpdate: "agent_message_chunk", content: "Verify-mode response." }} }});
              return ok({{ stopReason: "end_turn" }});
            }}
            return ok({{}});
          default:
            console.warn("verify bridge: unhandled invoke", cmd, args);
            return ok({{}});
        }}
      }}
    }},
    event: {{
      listen(event, cb) {{
        const list = listeners.get(event) || [];
        list.push(cb);
        listeners.set(event, list);
        return ok(() => {{
          const current = listeners.get(event) || [];
          listeners.set(event, current.filter((item) => item !== cb));
        }});
      }}
    }}
  }};
}})();
"##
    );
    if let Err(error) = eval(&script) {
        web_sys::console::error_2(&"verify bridge install failed".into(), &error);
    }
}

pub fn install_verify_mode_from_location() -> Option<VerifyConfig> {
    let config = verify_config_from_search(&current_location_search()?)?;
    let fixture = fixture_for_config(&config);
    install_mock_bridge(&config, &fixture);
    apply_fixture(&config, &fixture);
    Some(config)
}

pub fn sync_debug_snapshot() {
    let Some(config) = current_location_search().and_then(|search| verify_config_from_search(&search)) else {
        return;
    };
    let snapshot = json!({
        "verifyMode": true,
        "scenario": format!("{:?}", config.scenario),
        "project": PROJECT.read().clone(),
        "sessionId": SESSION_ID.read().clone(),
        "connected": *CONNECTED.read(),
        "running": *RUNNING.read(),
        "view": format!("{:?}", *VIEW.read()),
        "items": ITEMS.read().len(),
        "plan": PLAN.read().len(),
        "queue": crate::state::PENDING_QUEUE.read().len(),
        "permission": PERMISSION.read().as_ref().map(|req| req.title.clone()),
        "panels": {
            "terminal": *TERMINAL_OPEN.read(),
            "memory": *SHOW_MEMORY.read(),
            "browser": *SHOW_BROWSER.read(),
            "diff": *SHOW_DIFF.read(),
            "multiAgent": *SHOW_MULTI_AGENT.read(),
            "automations": *SHOW_AUTOMATIONS.read(),
        },
        "error": ERROR.read().clone(),
    });
    if let Ok(raw) = serde_json::to_string(&snapshot) {
        if let Ok(raw_literal) = serde_json::to_string(&raw) {
            let _ = eval(&format!(
                "(() => {{ let el = document.getElementById('kimi-debug-state'); \
                 if (!el) {{ el = document.createElement('script'); el.id = 'kimi-debug-state'; \
                 el.type = 'application/json'; document.body.appendChild(el); }} \
                 el.textContent = {raw_literal}; }})()"
            ));
        }
        if let Err(error) = eval(&format!("window.__KIMI_DEBUG_STATE__ = {raw};")) {
            web_sys::console::error_2(&"verify snapshot sync failed".into(), &error);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_config_requires_verify_flag() {
        assert_eq!(verify_config_from_search("?scenario=active-session"), None);
    }

    #[test]
    fn verify_config_defaults_to_active_session() {
        assert_eq!(
            verify_config_from_search("?verify=1"),
            Some(VerifyConfig {
                scenario: VerifyScenario::ActiveSession,
                panels: Vec::new(),
            })
        );
    }

    #[test]
    fn verify_config_parses_scenario_and_multiple_panels() {
        assert_eq!(
            verify_config_from_search("?verify=1&scenario=permission-request&panel=terminal,memory"),
            Some(VerifyConfig {
                scenario: VerifyScenario::PermissionRequest,
                panels: vec![VerifyPanel::Terminal, VerifyPanel::Memory],
            })
        );
    }

    #[test]
    fn verify_config_ignores_unknown_values() {
        assert_eq!(
            verify_config_from_search("?verify=1&scenario=unknown&panel=terminal,unknown,diff"),
            Some(VerifyConfig {
                scenario: VerifyScenario::ActiveSession,
                panels: vec![VerifyPanel::Terminal, VerifyPanel::Diff],
            })
        );
    }

    #[test]
    fn active_session_fixture_contains_session_thread_and_files() {
        let fixture = fixture_for_config(&VerifyConfig {
            scenario: VerifyScenario::ActiveSession,
            panels: Vec::new(),
        });

        assert!(fixture.connected);
        assert_eq!(fixture.project.as_deref(), Some("/tmp/kimi-verify-project"));
        assert_eq!(fixture.session_id.as_deref(), Some("verify-session"));
        assert!(!fixture.sessions.is_empty());
        assert!(fixture
            .items
            .iter()
            .any(|item| matches!(item, Item::Agent(text) if text.contains("verification"))));
        assert!(fixture.project_files.iter().any(|path| path == "src/main.rs"));
    }

    #[test]
    fn permission_request_fixture_sets_modal_and_running_state() {
        let fixture = fixture_for_config(&VerifyConfig {
            scenario: VerifyScenario::PermissionRequest,
            panels: Vec::new(),
        });

        assert!(fixture.running);
        assert!(fixture.permission.is_some());
        assert!(fixture
            .items
            .iter()
            .any(|item| matches!(item, Item::Tool(tool) if tool.status == "in_progress")));
    }

    #[test]
    fn mock_invoke_returns_recent_projects_and_session_list() {
        let fixture = fixture_for_config(&VerifyConfig {
            scenario: VerifyScenario::ActiveSession,
            panels: Vec::new(),
        });

        let projects = mock_invoke_for_fixture(&fixture, "recent_projects", &json!({}))
            .unwrap()
            .unwrap();
        assert_eq!(projects.as_array().unwrap().len(), 2);

        let sessions = mock_invoke_for_fixture(
            &fixture,
            "acp_request",
            &json!({"method": "session/list", "params": {}}),
        )
        .unwrap()
        .unwrap();
        assert_eq!(sessions["sessions"][0]["sessionId"], "verify-session");
    }

    #[test]
    fn mock_invoke_ignores_unknown_commands() {
        let fixture = fixture_for_config(&VerifyConfig {
            scenario: VerifyScenario::ActiveSession,
            panels: Vec::new(),
        });

        assert!(mock_invoke_for_fixture(&fixture, "unknown_command", &json!({})).is_none());
    }
}
