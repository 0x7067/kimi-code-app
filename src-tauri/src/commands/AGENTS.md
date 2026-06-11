<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-06-11 | Updated: 2026-06-11 -->

# commands

## Purpose
Tauri command handlers, grouped by concern. Each file contains one or more `#[tauri::command]` async functions that the Dioxus frontend invokes via `tauri::invoke`. The `mod.rs` registers every command in a single `tauri::generate_handler!` macro call consumed by `src-tauri/src/lib.rs`, and defines the shared `AppState` struct used across the backend.

## Key Files

| File | Description |
|------|-------------|
| `mod.rs` | Command registry — module declarations and `handlers()` returning the invoke handler for all commands |
| `acp.rs` | ACP client lifecycle — `acp_connect`, `acp_request`, `acp_notify`, `acp_cancel`, `acp_steer`, `acp_respond_permission`; also defines `AppState` |
| `automation.rs` | Automation execution — `list_automation_runs` and `run_automation_now` (headless prompt runner wrapper) |
| `browser.rs` | Browser preview — `start_browser_watcher` / `stop_browser_watcher` for live-reload (F-006) |
| `checkpoint.rs` | Session checkpoint CRUD — `save_checkpoint`, `list_checkpoints`, `load_checkpoint`, `delete_checkpoint` |
| `config.rs` | Kimi config file I/O — `read_kimi_config` / `write_kimi_config` for allowed files in `~/.kimi-code`; also `list_kimi_models` and `set_default_model` |
| `dialogs.rs` | Native dialogs — `pick_file`, `pick_folder`, `pick_image` (returns base64-encoded image data) |
| `git.rs` | Git integration — `git_diff`, `list_worktrees`, `create_worktree`, `remove_worktree` |
| `kimi.rs` | Kimi CLI wrappers — `kimi_login` (streaming OAuth), `kimi_version`, `detect_kimi_binary`, `kimi_auth_status`, `js_log` |
| `memory.rs` | Memory snippet commands — `list_memories`, `save_memory`, `delete_memory`, `pin_memory`, `retrieve_memories`, `build_memory_context` |
| `mcp.rs` | MCP server management — `list_mcp_servers`, `save_mcp_server`, `delete_mcp_server` over `~/.kimi-code/mcp.json` with validation |
| `multi_agent.rs` | Multi-agent orchestration — `create_multi_agent_run`, `list_multi_agent_runs`, `get_multi_agent_run`, `set_task_session`, `set_task_status` |
| `projects.rs` | Project discovery — `recent_projects`, `list_files`, `index_project`, `read_agents_md`, `mcp_servers` |
| `sessions.rs` | Session sync — `kimi_list_sessions`, `kimi_load_session`, `kimi_session_activity`, plus `spawn_session_index_watcher` |
| `settings_store.rs` | App settings JSON store in the Tauri app-config dir — `read_app_settings` / `write_app_settings` (atomic) |
| `terminal.rs` | Embedded terminal — `term_open`, `term_write`, `term_resize`, `term_close` |

## Subdirectories

*None.*

## For AI Agents

### Working In This Directory
- Every public command function must be listed in `mod.rs`'s `generate_handler!` macro or it will not be callable from the frontend.
- Commands take `AppHandle`, `State<'_, AppState>` (or `State<'_, TermState>` for terminal commands), or plain serializable arguments.
- Return `Result<Ok, Err>` where `Err` is usually `String` or `serde_json::Value` for JSON-RPC-shaped errors.
- Keep commands thin; heavy logic belongs in the corresponding `crate::*` modules (e.g., `crate::acp`, `crate::memory`, `crate::multi_agent`).

### Testing Requirements
- Unit tests exist in `config.rs`, `projects.rs`, `mcp.rs`, and `git.rs`; run them with `cargo test -p kimi-code-app-tauri`.
- Full validation is manual: run `cargo tauri dev` and exercise each UI flow that hits a command.

### Common Patterns
- `AppState` holds `Mutex<Option<Arc<AcpClient>>>`; always lock, clone the `Arc`, then drop the guard before awaiting.
- `json!({...})` from `serde_json` is used heavily for constructing JSON-RPC payloads and ad-hoc return values.
- `kimi_home()` and `kimi_bin()` from `crate::paths` resolve cross-platform paths.
- Dialog commands use a `tokio::sync::oneshot` channel to adapt Tauri's callback-based file picker API into an async/await command.
- Settings commands write atomically (temp file + rename) and apply backend-side values (e.g., the kimi binary override) immediately.

## Dependencies

### Internal
- `src-tauri/src/lib.rs` — consumes `handlers()` to build the Tauri app
- `src-tauri/src/acp/` — `AcpClient` used by `acp.rs` commands
- `src-tauri/src/paths.rs` — `kimi_home()`, `kimi_bin()`, and binary override helpers
- `src-tauri/src/memory.rs` — memory store operations used by `memory.rs` commands
- `src-tauri/src/multi_agent.rs` — multi-agent state and task model used by `multi_agent.rs`
- `src-tauri/src/terminal.rs` — PTY spawning and `Registry<Term>` used by `terminal.rs`
- `src-tauri/src/browser.rs` — live-reload watcher used by `browser.rs`
- `src-tauri/src/checkpoint.rs` — checkpoint persistence used by `checkpoint.rs`
- `src-tauri/src/automation.rs` / `src-tauri/src/headless.rs` — automation logging and headless execution used by `automation.rs`
- `src/` (frontend) — invokes these commands via `tauri::invoke`

### External
- `tauri` — `AppHandle`, `State`, `Emitter`, `Manager`
- `tauri-plugin-dialog` — native file/folder pickers
- `tokio` — async fs, process spawning, oneshot channels
- `serde_json` — JSON construction and parsing
- `base64` — image encoding for `pick_image`
- `dirs` — home directory lookup
- `chrono` — timestamps for automation runs
- `notify` — filesystem watcher for browser live-reload (stored in `AppState`)
- `tempfile` — dev-dependency for unit tests

<!-- MANUAL: -->
