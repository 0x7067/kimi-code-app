<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-06-11 | Updated: 2026-06-11 -->

# src-tauri

## Purpose
Tauri 2 backend crate for the Kimi Code desktop app. Handles native desktop concerns: spawning the `kimi acp` CLI process, JSON-RPC 2.0 communication over stdio, file system access, native dialogs, git operations, config file editing, and OAuth login flow. Exposes all capabilities to the Dioxus frontend via Tauri commands and events.

## Key Files

| File | Description |
|------|-------------|
| `Cargo.toml` | Backend crate manifest — Tauri 2, tokio, dialog/opener plugins |
| `tauri.conf.json` | Tauri app config — window size, dev/build commands, CSP, bundle settings |
| `build.rs` | Tauri build script (`tauri_build::build()`) |

## Subdirectories

| Directory | Purpose |
|-----------|---------|
| `src/` | Rust backend source — commands, ACP client, app setup (see `src/AGENTS.md`) |
| `capabilities/` | Tauri capability definitions for permission scoping (see `capabilities/AGENTS.md`) |
| `gen/` | Tauri-generated schemas and bindings (see `gen/AGENTS.md`) |
| `icons/` | Platform app icons in multiple resolutions and formats (see `icons/AGENTS.md`) |

## For AI Agents

### Working In This Directory
- This is a standard Tauri 2 project layout. Commands are Rust `async fn`s annotated with `#[tauri::command]`.
- Shared mutable state is stored in `AppState` and accessed via Tauri `State<AppState>`.
- The backend spawns `kimi acp` as a child process and speaks JSON-RPC 2.0 over its stdio.

### Testing Requirements
- No automated tests are present. Test by running `cargo tauri dev` and exercising UI flows.

### Common Patterns
- Commands return `Result<Ok, Err>` where `Err` is usually `String` or `Value` for JSON-RPC errors.
- Events are emitted to the frontend with `app.emit("event:name", payload)`.
- File I/O uses `tokio::fs` for async operations.

## Dependencies

### Internal
- `src/` — frontend source that invokes these backend commands

### External
- **tauri** v2 — Desktop app framework
- **tauri-plugin-dialog** v2 — Native file/folder picker dialogs
- **tauri-plugin-opener** v2 — Open external URLs
- **tokio** — Async runtime (process, io-util, sync, fs)
- **serde / serde_json** — Serialization
- **dirs** — Cross-platform home directory lookup
- **base64** — Image encoding for ACP image attachments

<!-- MANUAL: -->
