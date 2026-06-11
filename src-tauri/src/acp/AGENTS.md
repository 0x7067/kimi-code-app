<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-06-11 | Updated: 2026-06-11 -->

# acp

## Purpose
The ACP (Agent Communication Protocol) client module. Manages the lifecycle of the `kimi acp` JSON-RPC 2.0 subprocess over stdio, routing messages between the Dioxus frontend and the Kimi CLI agent. Handles connection setup, request/response pairing, session streaming, permission prompts, file I/O passthrough, crash recovery, turn serialization, and reading the on-disk session store.

## Key Files

| File | Description |
|------|-------------|
| `mod.rs` | `AcpClient` struct: spawns `kimi acp`, sends requests/notifications, routes inbound messages, handles reconnects and steering. |
| `protocol.rs` | Pure protocol helpers: message routing, version negotiation, capability parsing, update classification, and prompt helpers. Includes unit tests. |
| `queue.rs` | Bounded outbound message queue for disconnect buffering and per-session `TurnQueue` for serializing `session/prompt` calls. Includes unit tests. |
| `sessions.rs` | In-memory `SessionRegistry` that records per-session update history across reconnects. Includes unit tests. |
| `store.rs` | Parsing for Kimi's on-disk session store (`session_index.jsonl`, `state.json`) and `SessionSummary` construction. Includes unit tests. |
| `supervisor.rs` | Connection-health supervisor with exponential backoff and restart budget for crash recovery. Includes unit tests. |

## Subdirectories

_None._

## For AI Agents

### Working In This Directory
- This is pure Rust backend code inside `src-tauri`. Keep it independent of the frontend; only `mod.rs` touches Tauri APIs (`AppHandle`, `Emitter`).
- `protocol.rs`, `queue.rs`, `sessions.rs`, `store.rs`, and `supervisor.rs` are designed to be I/O-free and unit-testable. Avoid introducing Tauri or tokio process dependencies into those files.
- Any change to JSON-RPC routing, capability parsing, or the `initialize` handshake must be reflected in both `mod.rs` and `protocol.rs`.

### Testing Requirements
- Run unit tests with `cargo test -p kimi-code-app-tauri` (or `cargo test` from `src-tauri/`).
- The module contains `#[cfg(test)]` blocks in every file; new protocol or parsing logic should include tests.
- Manual integration testing requires `kimi` on PATH and running the app via `cargo tauri dev`.

### Common Patterns
- JSON-RPC messages are represented as `serde_json::Value`.
- Requests use `tokio::sync::oneshot` channels to match async responses by numeric id.
- Agent->client requests (permission, fs read/write) are handled inline and replied via `stdin_tx`.
- Disconnected notifications are queued in `MessageQueue` and flushed on reconnect.
- `session/prompt` calls are serialized per session through `TurnQueue`; steering cancels the active turn before enqueuing the new prompt.

## Dependencies

### Internal
- `src-tauri/src/commands/` — `AppState` owns the shared `AcpClient`, `MessageQueue`, `SessionRegistry`, and capability slot.
- `src-tauri/src/paths.rs` — `crate::paths::kimi_bin()` resolves the `kimi` executable path.
- Frontend (via Tauri events) — emits `acp:disconnected`, `acp:crashed`, `acp:restarted`, `acp:update`, `acp:notification`, `acp:permission_request`, `acp:version_mismatch`, `acp:restart_failed`.

### External
- **tokio** — async process I/O, channels, and timeouts.
- **serde / serde_json** — serialization and dynamic JSON-RPC values.
- **tauri** — `AppHandle`, `Emitter`, `Manager`, and the async runtime.

<!-- MANUAL: -->
