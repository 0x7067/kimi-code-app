<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-06-11 | Updated: 2026-06-11 -->

# src

## Purpose
Dioxus 0.7 frontend source for the Kimi Code desktop app. This directory contains the entire webview UI: the root launcher, global state management, component tree, design tokens, and wasm-bindgen glue for Tauri IPC. The code compiles to WASM and runs inside the Tauri webview.

## Key Files

| File | Description |
|------|-------------|
| `main.rs` | Entry point — loads CSS assets and mounts the root `App` component |
| `actions.rs` | Async action helpers — connect, send prompt, refresh sessions/projects, set config |
| `conversation.rs` | Conversation domain types and message handling logic |
| `ipc.rs` | wasm-bindgen wrappers around `window.__TAURI__.core.invoke` and event listening |
| `markdown.rs` | Markdown → HTML rendering for agent messages using `pulldown-cmark` |

## Subdirectories

| Directory | Purpose |
|-----------|---------|
| `components/` | Dioxus UI components — one file per screen region (see `components/AGENTS.md`) |
| `design_tokens/` | Visual design system constants — colors, typography, spacing, animation, elevation |
| `state/` | Global signals, domain types, and ACP update reducer (see `state/AGENTS.md`) |

## For AI Agents

### Working In This Directory
- All files are compiled to WASM for the webview — use `web-sys` and `wasm-bindgen` for browser APIs.
- Dioxus uses RSX syntax (`rsx! { ... }`) for templating; state mutations must go through `.write()` on signals.
- The Tauri bridge is in `ipc.rs` — prefer `invoke()` and `listen_forever()` over raw JS calls.
- Visual constants live in `design_tokens/`; import them instead of hard-coding colors or spacing.

### Testing Requirements
- Run `cargo test` to execute the unit tests in `markdown.rs`.
- Validate UI changes by running `cargo tauri dev`.

### Common Patterns
- Global state is declared as `pub static FOO: GlobalSignal<T> = Signal::global(...)` in `state/signals.rs`.
- Async actions are spawned with `spawn(async { ... })` inside event handlers.
- Markdown rendering uses `pulldown_cmark` with `Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH` and wraps code blocks with a copy button.

## Dependencies

### Internal
- `src-tauri/src/commands/` — backend commands invoked from the frontend
- `src-tauri/src/acp/` — ACP client that handles the JSON-RPC wire protocol

### External
- `dioxus` — UI framework (web feature)
- `serde` / `serde_json` — Serialization and JSON parsing for ACP payloads
- `pulldown-cmark` — Markdown to HTML
- `wasm-bindgen`, `wasm-bindgen-futures`, `js-sys`, `web-sys`, `serde-wasm-bindgen` — WASM/JS bridge
- `gloo-timers` — Timer utilities for async delays

<!-- MANUAL: -->
