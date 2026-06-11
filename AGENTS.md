<!-- Generated: 2026-06-11 | Updated: 2026-06-11 -->

# kimi-code-app

## Purpose
A native desktop GUI for the Kimi Code CLI's ACP (Agent Client Protocol), built with **Tauri 2** and **Dioxus 0.7** — pure Rust, no JS framework. The app bridges a webview frontend to `kimi acp` via JSON-RPC 2.0 over stdio, providing a project/session sidebar, streaming agent thread, tool-call approvals, plan tracking, diff review, settings, memory, automations, MCP server integration, terminal, browser preview, and multi-agent orchestration.

## Key Files

| File | Description |
|------|-------------|
| `Cargo.toml` | Workspace manifest — Dioxus UI crate dependencies, WASM profile configs, workspace lints, `src-tauri` member |
| `Cargo.lock` | Locked dependency resolution for the workspace |
| `Dioxus.toml` | Dioxus app config — web platform, `out_dir = dist`, `asset_dir = assets`, dev watcher |
| `README.md` | Human-facing project overview, features, and development instructions |
| `DESIGN_SYSTEM.md` | Complete design-system spec: tokens, base components, layout, icons, animations |
| `REQUIREMENTS.md` | Functional/non-functional requirements, feature list, and acceptance criteria |
| `PROGRESS.md` | Living implementation tracker, verified protocol facts, and checkpoint status |
| `LICENSE` | MIT License |
| `.gitignore` | Ignore rules: `/dist/`, `/target/`, `/Cargo.lock` |
| `.taurignore` | Tauri build ignore rules: `/src`, `/assets`, `/Cargo.toml` |
| `rustfmt.toml` | Rustfmt config: edition 2021, `max_width = 110` |

## Subdirectories

| Directory | Purpose |
|-----------|---------|
| `.vscode/` | VS Code workspace recommendations (see `.vscode/AGENTS.md`) |
| `assets/` | Static assets shipped to the webview — CSS, icons, images (see `assets/AGENTS.md`) |
| `scripts/` | Development automation scripts (see `scripts/AGENTS.md`) |
| `src/` | Dioxus frontend source — `main.rs`, components, state, IPC, markdown (see `src/AGENTS.md`) |
| `src-tauri/` | Tauri 2 backend — `lib.rs`, commands, ACP client, native integrations (see `src-tauri/AGENTS.md`) |

## For AI Agents

### Working In This Directory
- This is a Rust workspace with two crates: the root Dioxus UI crate and `src-tauri` as a workspace member.
- Run `cargo tauri dev` for development with hot-reload (`dx serve` on `:1420`).
- Run `cargo tauri build` for release bundling (`.app`/`.dmg`).
- Requires `rustup target add wasm32-unknown-unknown` and `cargo install dioxus-cli tauri-cli`.
- Do not commit `dist/` or `target/`; `Cargo.lock` is gitignored at the root.

### Testing Requirements
- Run `cargo check --workspace`, `cargo check --target wasm32-unknown-unknown`, and `cargo test --workspace` for automated validation.
- Manual smoke-testing via `cargo tauri dev` is required for UI/visual changes.
- Ensure the `kimi` CLI is on PATH before running the app.

### Common Patterns
- Frontend/backend communication uses Tauri invoke/listen APIs over `window.__TAURI__`.
- All UI state lives in Dioxus `GlobalSignal`s defined in `src/state/signals.rs`, updated through the reducer in `src/state/reducer.rs`.
- Backend commands are grouped in `src-tauri/src/commands/mod.rs` and registered in `src-tauri/src/lib.rs`.
- Styling uses modular semantic CSS in `assets/css/*.css`; there is no Tailwind build pipeline.
- Design tokens (colors, typography, spacing, animation, elevation) live in `src/design_tokens/`.

## Dependencies

### Internal
- `src/` → Dioxus web frontend
- `src-tauri/` → Tauri desktop backend

### External
- **Dioxus 0.7** — Rust UI framework (web target)
- **Tauri 2** — Native desktop shell and Rust backend
- **wasm-bindgen / web-sys / js-sys** — WASM/JS interop for frontend IPC
- **serde / serde_json / serde-wasm-bindgen** — Serialization everywhere
- **pulldown-cmark** — Markdown → HTML rendering in the frontend
- **gloo-timers** — Timer helpers for frontend animations
- **tokio** — Async runtime in the backend
- **dirs / base64 / portable-pty / cron / chrono / notify** — Backend utilities (paths, encoding, PTY, scheduling, file watching)

<!-- MANUAL: -->
