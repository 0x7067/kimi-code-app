<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-06-11 | Updated: 2026-06-11 -->

# src

## Purpose
Tauri 2 backend source code. Defines the application runtime (`lib.rs`, `main.rs`), the JSON-RPC ACP client (`acp.rs`), and the full set of Tauri commands (under `commands/`) that the Dioxus frontend invokes. This is where all native/desktop logic lives: process spawning, async I/O, file system, git, dialogs, and config management.

## Key Files

| File | Description |
|------|-------------|
| `lib.rs` | Tauri app builder — plugin initialization, `AppState` management, command routing, page-load JS error forwarding |
| `main.rs` | Binary entry point — delegates to `kimi_code_app_lib::run()` |
| `acp.rs` | `AcpClient` — spawns `kimi acp`, manages JSON-RPC 2.0 request/response mapping, handles server->client requests (permissions, fs ops) |
| `paths.rs` | Path helpers — `kimi_home()`, `kimi_bin()`, cross-platform config directory resolution |

## Subdirectories

| Directory | Purpose |
|-----------|---------|
| `commands/` | Tauri command handlers grouped by concern (see `commands/AGENTS.md`) |

## For AI Agents

### Working In This Directory
- Commands are `async` Rust functions with `#[tauri::command]` and take `AppHandle`, `State<AppState>`, or plain arguments.
- `AppState` holds an `Arc<AcpClient>` behind a `tokio::sync::Mutex`; clone the `Arc` before dropping the lock.
- The `AcpClient` owns the child process stdin/stdout and maps JSON-RPC ids to oneshot channels for request/response pairing.

### Testing Requirements
- No automated tests. Validate by running `cargo tauri dev` and exercising UI flows.

### Common Patterns
- Use `json!({...})` from `serde_json` for constructing JSON-RPC payloads and error values.
- File paths are resolved via `dirs::home_dir()` and the `KIMI_CODE_HOME` env var (default `~/.kimi-code`).
- `kimi_bin()` searches known install locations before falling back to PATH.
- Permission requests from the agent are bridged to frontend events (`acp:permission_request`) and resolved asynchronously.

## Dependencies

### Internal
- `src/` (frontend) — invokes commands defined here

### External
- **tauri** v2 — App framework, `AppHandle`, `Emitter`, `State`
- **tauri-plugin-dialog** v2 — File/folder picker dialogs
- **tokio** — Async process spawning, buffered I/O, file system, channels
- **serde_json** — JSON-RPC wire format
- **dirs** — Home directory discovery
- **base64** — Image encoding for `pick_image`

<!-- MANUAL: -->
