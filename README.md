# Kimi Code App

A native desktop GUI for [Kimi Code](https://www.kimi.com/code/docs/en/) (Moonshot AI's coding agent), built with **Tauri 2** and **Dioxus 0.7** — pure Rust, no JS framework.

Inspired by the OpenAI Codex desktop app: project/session sidebar, streaming agent thread, tool-call approvals, plan tracking, diff review, and config editing.

## How it works

The Tauri backend spawns `kimi acp` (Agent Client Protocol — JSON-RPC 2.0 over stdio, the same integration Zed uses) and bridges it to the webview:

- `acp_request` / `acp_notify` — generic JSON-RPC passthrough (`session/new`, `session/load`, `session/prompt`, `session/cancel`, `session/set_mode`, `session/set_config_option`, …)
- `acp:update` events — streaming `session/update` notifications (message/thought chunks, tool calls, plans, slash commands)
- `acp:permission_request` events + `acp_respond_permission` — interactive tool approvals
- `fs/read_text_file` / `fs/write_text_file` reverse-RPC handled natively
- `kimi_login` — device-code OAuth with streamed output
- Config editing for `~/.kimi-code/{config.toml,tui.toml,mcp.json,AGENTS.md}`
- `git_diff` — working-tree review pane

## Features

- Project picker (recent projects from Kimi's session index + native folder dialog)
- Session list, resume with full history replay, new sessions per project
- Streaming markdown agent replies, collapsible thinking + tool calls with live status
- Plan panel, slash-command autocomplete, model/thinking/mode selectors
- Permission approval modal (manual / auto / yolo / plan modes supported)
- Git diff pane, settings editor, login flow

## Development

```sh
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli tauri-cli

cargo tauri dev          # dev with hot-reload (dx serve on :1420)
cargo tauri build        # release bundle (.app/.dmg)
```

Requires the `kimi` CLI on PATH (`curl -fsSL https://code.kimi.com/kimi-code/install.sh | bash`).
