<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-06-11 | Updated: 2026-06-11 -->

# src-tauri

## Purpose
Tauri 2 backend crate for the Kimi Code desktop app. Handles native desktop concerns: spawning the `kimi acp` CLI process, JSON-RPC 2.0 communication over stdio, native file/folder/image dialogs, app settings storage, `kimi` config editing, OAuth login, session/project indexing, git diff and worktree operations, checkpoints, long-term memory, MCP server management, terminal emulation, an automation scheduler, multi-agent runs, and browser watcher orchestration. All capabilities are exposed to the Dioxus frontend via Tauri commands and events.

## Key Files

| File | Description |
|------|-------------|
| `Cargo.toml` | Backend crate manifest — Tauri 2, tokio, dialog/opener plugins, pty, cron, chrono, notify |
| `tauri.conf.json` | Tauri app config — window size, dev/build commands, CSP, bundle settings, icon list |
| `build.rs` | Tauri build script (`tauri_build::build()`) |
| `.gitignore` | Excludes generated `gen/schemas` artifacts and build output |

## Subdirectories

| Directory | Purpose |
|-----------|---------|
| `src/` | Rust backend source — commands, ACP client, app setup (see `src/AGENTS.md`) |
| `capabilities/` | Tauri capability definitions for permission scoping (see `capabilities/AGENTS.md`) |
| `gen/` | Tauri-generated schemas and bindings (see `gen/AGENTS.md`) |
| `icons/` | Platform app icons in multiple resolutions and formats (see `icons/AGENTS.md`) |

## For AI Agents

### Working In This Directory
- This is a standard Tauri 2 project layout. Commands are Rust `async fn`s annotated with `#[tauri::command]` and registered in `src/lib.rs` via `commands::handlers()`.
- Shared mutable state lives in `AppState` and `TermState`; access it through Tauri `State<AppState>` / `State<TermState>`.
- The backend spawns `kimi acp` as a child process and speaks JSON-RPC 2.0 over its stdio via the `src/acp/` module.
- `lib.rs` starts a file-system watcher for the shared session index and the automation scheduler inside `.setup()`.

### Testing Requirements
- No automated tests are present. Validate with `cargo check` / `cargo clippy`, then run `cargo tauri dev` and exercise the relevant UI flows end-to-end.

### Common Patterns
- Commands return `Result<T, String>` (or `serde_json::Value` for JSON-RPC-shaped responses).
- Events are emitted to the frontend with `app.emit("event:name", payload)` or `window.emit(...)`.
- Async file I/O uses `tokio::fs`; cross-platform home directory lookups use `dirs`.
- Paths sent to the frontend are typically returned as strings; the frontend resolves them as needed.

## Dependencies

### Internal
- `src/` — Dioxus frontend that invokes these backend commands and listens for backend events

### External
- **tauri** v2 — Desktop app framework
- **tauri-plugin-dialog** v2 — Native file/folder/image picker dialogs
- **tauri-plugin-opener** v2 — Open external URLs/files
- **tokio** — Async runtime (process, io-util, sync, fs, macros, time)
- **serde / serde_json** — Serialization
- **dirs** — Cross-platform home directory lookup
- **base64** — Image encoding for ACP image attachments
- **portable-pty** — Terminal emulation pty
- **cron / chrono** — Automation scheduler and datetime handling
- **notify** — File-system watching for the session index
- **tauri-build** — Build-time Tauri code generation

<!-- MANUAL: -->
