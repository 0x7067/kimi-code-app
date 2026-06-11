<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-06-11 | Updated: 2026-06-11 -->

# commands

## Purpose
Tauri command handlers, grouped by concern. Each file contains one or more `#[tauri::command]` async functions that the Dioxus frontend invokes via `tauri::invoke`. The `mod.rs` registers all commands in a single `tauri::generate_handler!` macro call consumed by `lib.rs`.

## Key Files

| File | Description |
|------|-------------|
| `mod.rs` | Command registry — `handlers()` returns the invoke handler with all commands registered |
| `acp.rs` | ACP lifecycle — `acp_connect`, `acp_request`, `acp_notify`, `acp_respond_permission`; also defines `AppState` |
| `kimi.rs` | Kimi CLI wrappers — `kimi_login` (OAuth streaming), `kimi_version`, `js_log` |
| `config.rs` | Config file I/O — `read_kimi_config` / `write_kimi_config` for allowed files in `~/.kimi-code` |
| `projects.rs` | Project discovery — `recent_projects` from session index, `mcp_servers` merging user + project-level MCP configs |
| `dialogs.rs` | Native dialogs — `pick_folder`, `pick_image` (returns base64-encoded image data) |
| `git.rs` | Git integration — `git_diff` for working-tree review |

## Subdirectories

*None.*

## For AI Agents

### Working In This Directory
- Every public async function must be listed in `mod.rs`'s `generate_handler!` macro or it won't be callable from the frontend.
- Commands take `AppHandle`, `State<'_, AppState>`, or plain serializable arguments.
- Return `Result<Ok, Err>` where `Err` is usually `String` or `serde_json::Value` for JSON-RPC-shaped errors.
- Keep commands thin; heavy logic belongs in `crate::acp` or `crate::paths` modules.

### Testing Requirements
- No automated tests. Validate by running `cargo tauri dev` and exercising each UI flow that hits a command.

### Common Patterns
- `AppState` holds `Mutex<Option<Arc<AcpClient>>>`; always lock, clone the `Arc`, then drop the guard before awaiting.
- `json!({...})` from `serde_json` is used heavily for constructing JSON-RPC payloads and ad-hoc return values.
- `kimi_home()` and `kimi_bin()` from `crate::paths` resolve cross-platform paths.

## Dependencies

### Internal
- `src-tauri/src/lib.rs` — consumes `handlers()` to build the Tauri app
- `src-tauri/src/acp.rs` — `AcpClient` used by `acp.rs` commands
- `src-tauri/src/paths.rs` — `kimi_home()` and `kimi_bin()` helpers
- `src/` (frontend) — invokes these commands via `tauri::invoke`

### External
- `tauri` — `AppHandle`, `State`, `Emitter`
- `tauri-plugin-dialog` — native file/folder pickers
- `tokio` — async fs, process spawning, oneshot channels
- `serde_json` — JSON construction and parsing
- `base64` — image encoding for `pick_image`
- `dirs` — home directory lookup

<!-- MANUAL: -->
