<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-06-11 | Updated: 2026-06-11 -->

# src

## Purpose
Tauri 2 backend source code. Defines the application runtime (`lib.rs`, `main.rs`), the JSON-RPC ACP client (`acp/`), and supporting native/desktop services: terminal PTYs, automation scheduling, session checkpoints, cross-session memory, multi-agent orchestration, browser live-reload, and path resolution. Tauri command handlers live under `commands/`.

## Key Files

| File | Description |
|------|-------------|
| `lib.rs` | Tauri app builder — plugin initialization, `AppState`/`TermState` management, command routing, session-index watcher, automation scheduler tick, page-load JS error forwarding |
| `main.rs` | Binary entry point — delegates to `kimi_code_app_lib::run()` |
| `automation.rs` | Cron automation scheduler and execution history (F-009) |
| `browser.rs` | Browser preview live-reload file watcher (F-006) |
| `checkpoint.rs` | Session checkpoint save/restore as JSON snapshots (F-002.6) |
| `headless.rs` | Headless ACP execution for automations via short-lived `kimi acp` processes (F-009) |
| `memory.rs` | Cross-session memory store with keyword-based retrieval (F-007) |
| `multi_agent.rs` | Multi-agent orchestration state tracker for parallel ACP sessions (F-004) |
| `paths.rs` | Resolution of the `kimi` binary, `kimi_home()`, and cross-platform config paths |
| `terminal.rs` | PTY-backed embedded terminal registry and shell spawning (F-010) |

## Subdirectories

| Directory | Purpose |
|-----------|---------|
| `acp/` | JSON-RPC 2.0 ACP client modules: protocol routing, message/turn queues, session registry, on-disk session store, and connection supervisor (see `acp/AGENTS.md`) |
| `commands/` | Tauri command handlers grouped by concern (see `commands/AGENTS.md`) |

## For AI Agents

### Working In This Directory
- Commands are `async` Rust functions with `#[tauri::command]` and take `AppHandle`, `State<AppState>`, `State<TermState>`, or plain arguments.
- `AppState` holds an `Arc<AcpClient>` behind a `tokio::sync::Mutex`; clone the `Arc` before dropping the lock.
- The `AcpClient` owns the child process stdin/stdout and maps JSON-RPC ids to oneshot channels for request/response pairing.
- Pure utility modules (`paths`, `acp::protocol`, `acp::queue`, `acp::sessions`, `acp::store`, `acp::supervisor`, `checkpoint`, `memory`, `multi_agent`) contain unit tests and avoid Tauri I/O where possible.

### Testing Requirements
- Run unit tests with `cargo test -p kimi-code-app`.
- Validate UI integration by running `cargo tauri dev` and exercising frontend flows.

### Common Patterns
- Use `json!({...})` from `serde_json` for constructing JSON-RPC payloads and error values.
- File paths are resolved via `dirs::home_dir()` and the `KIMI_CODE_HOME` env var (default `~/.kimi-code`).
- `kimi_bin()` searches the configured override, then PATH, then known install locations.
- Permission requests from the agent are bridged to frontend events (`acp:permission_request`) and resolved asynchronously.
- Disconnected outbound notifications are queued in `MessageQueue` and flushed on reconnect (F-001.8).
- Only one `session/prompt` may be in flight per session; later prompts wait on `TurnQueue`.

## Dependencies

### Internal
- `src/` (frontend) — invokes commands defined here

### External
- **tauri** v2 — App framework, `AppHandle`, `Emitter`, `State`
- **tauri-plugin-dialog** v2 — File/folder picker dialogs
- **tauri-plugin-opener** v2 — External URL opener
- **tokio** — Async process spawning, buffered I/O, file system, channels, time
- **serde / serde_json** — Serialization and JSON-RPC wire format
- **dirs** — Home directory discovery
- **base64** — Image encoding for `pick_image`
- **portable-pty** — PTY-backed embedded terminals
- **cron** — Cron expression parsing for automations
- **chrono** — UTC timestamps for automation history
- **notify** — File-system watcher for browser live-reload

<!-- MANUAL: -->
