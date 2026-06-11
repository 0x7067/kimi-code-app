<!-- Generated: 2026-06-11 | Updated: 2026-06-11 -->

# kimi-code-app

## Purpose
A native desktop GUI for [Kimi Code](https://www.kimi.com/code/docs/en/) (Moonshot AI's coding agent), built with **Tauri 2** and **Dioxus 0.7** — pure Rust, no JS framework. The app bridges a webview frontend to the `kimi acp` CLI via JSON-RPC 2.0 over stdio, providing a project/session sidebar, streaming agent thread, tool-call approvals, plan tracking, diff review, and config editing.

## Key Files

| File | Description |
|------|-------------|
| `Cargo.toml` | Workspace manifest — Dioxus UI crate dependencies, WASM profile configs, Tauri member |
| `Dioxus.toml` | Dioxus app config — web platform, dev server on port 1420, asset bundling |
| `README.md` | Human-facing project overview, features, and development instructions |
| `.gitignore` | Git ignore rules |
| `.taurignore` | Tauri build ignore rules |

## Subdirectories

| Directory | Purpose |
|-----------|---------|
| `.vscode/` | VS Code workspace recommendations (see `.vscode/AGENTS.md`) |
| `assets/` | Static assets shipped to the webview (see `assets/AGENTS.md`) |
| `scripts/` | Development automation scripts (see `scripts/AGENTS.md`) |
| `src/` | Dioxus frontend source — UI components, state, Tauri IPC glue (see `src/AGENTS.md`) |
| `src-tauri/` | Tauri 2 backend — Rust commands, ACP JSON-RPC client, native integrations (see `src-tauri/AGENTS.md`) |

## For AI Agents

### Working In This Directory
- This is a Rust workspace with two crates: the root Dioxus UI crate and `src-tauri` as a workspace member.
- Run `cargo tauri dev` for development with hot-reload.
- Run `cargo tauri build` for release bundling (`.app`/`.dmg`).
- Requires `rustup target add wasm32-unknown-unknown` and `cargo install dioxus-cli tauri-cli`.

### Testing Requirements
- No automated test suite is currently configured. Manual testing via `cargo tauri dev` is the primary validation path.
- Ensure the `kimi` CLI is on PATH before running.

### Common Patterns
- Frontend/backend communication uses Tauri invoke/listen APIs over `window.__TAURI__`.
- All UI state lives in Dioxus `GlobalSignal`s defined in `src/state/signals.rs`.
- Backend commands are organized in `src-tauri/src/commands/` and registered in `src-tauri/src/lib.rs`.

## Dependencies

### Internal
- `src/` → Dioxus web frontend
- `src-tauri/` → Tauri desktop backend

### External
- **Dioxus 0.7** — Rust UI framework (web target)
- **Tauri 2** — Native desktop shell and Rust backend
- **wasm-bindgen / web-sys** — WASM/JS interop for frontend IPC
- **tokio** — Async runtime in the backend
- **serde / serde_json** — Serialization everywhere
- **pulldown-cmark** — Markdown → HTML rendering in the frontend

<!-- MANUAL: -->
