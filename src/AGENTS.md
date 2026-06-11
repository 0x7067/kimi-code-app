<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-06-11 | Updated: 2026-06-11 -->

# src

## Purpose
Dioxus 0.7 frontend source for the Kimi Code desktop app. This directory contains the entire webview UI: the root launcher, global state management, component tree, and wasm-bindgen glue for Tauri IPC. The code compiles to WASM and runs inside the Tauri webview.

## Key Files

| File | Description |
|------|-------------|
| `main.rs` | Entry point — loads CSS asset and mounts the root `App` component |
| `app.rs` | Main UI component tree: sidebar, thread view, composer, modals, diff pane, settings editor |
| `state.rs` | Global Dioxus signals (`GlobalSignal`) and the ACP `session/update` reducer (`apply_update`) |
| `tauri.rs` | wasm-bindgen wrappers around `window.__TAURI__.core.invoke` and event listening |

## Subdirectories

*None.*

## For AI Agents

### Working In This Directory
- All files are compiled to WASM for the webview — use `web-sys` and `wasm-bindgen` for browser APIs.
- Dioxus uses RSX syntax (`rsx! { ... }`) for templating; state mutations must go through `.write()` on signals.
- The Tauri bridge is in `tauri.rs` — prefer `invoke()` and `listen_forever()` over raw JS calls.

### Testing Requirements
- No unit tests exist here. Validate UI changes by running `cargo tauri dev`.

### Common Patterns
- Global state is declared as `pub static FOO: GlobalSignal<T> = Signal::global(...)` in `state.rs`.
- Async actions are spawned with `spawn(async { ... })` inside event handlers.
- Markdown rendering uses `pulldown_cmark` with `Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH`.

## Dependencies

### Internal
- `src-tauri/src/commands.rs` — backend commands invoked from the frontend
- `src-tauri/src/acp.rs` — ACP client that handles the JSON-RPC wire protocol

### External
- `dioxus` — UI framework (web feature)
- `serde_json` — JSON parsing for ACP payloads
- `pulldown-cmark` — Markdown to HTML
- `wasm-bindgen`, `wasm-bindgen-futures`, `js-sys`, `web-sys`, `serde-wasm-bindgen` — WASM/JS bridge

<!-- MANUAL: -->
