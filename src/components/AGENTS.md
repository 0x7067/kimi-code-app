<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-06-11 | Updated: 2026-06-11 -->

# components

## Purpose
Dioxus UI components for the Kimi Code desktop app, organized as one file per major screen region. The `mod.rs` barrel-exports all components so `main.rs` can mount the root `App` and nested views can import siblings. Shared primitive components live in `base/`, and the inline Lucide-style SVG icon set lives in `icons/`.

## Key Files

| File | Description |
|------|-------------|
| `mod.rs` | Barrel export — re-exports all top-level components plus `base` and `icons` modules |
| `app.rs` | Root `App` component — sets up Tauri event subscriptions and top-level layout |
| `composer.rs` | Message input box with slash-command autocomplete, send, cancel, and attachment support |
| `thread.rs` | Main chat thread — renders streaming agent messages, thoughts, tool calls, and plan tracking |
| `sidebar.rs` | Left sidebar — project/session list, search, and new-session button |
| `topbar.rs` | Header bar — project picker, diff toggle, export, and settings button |
| `status_bar.rs` | Bottom status bar — connection indicator, model name, current operation, and context-usage bar |
| `diff_pane.rs` | Git diff review panel — shows working-tree changes |
| `browser_pane.rs` | Browser preview pane with device toggles, live reload, and URL sharing |
| `multi_agent_pane.rs` | Multi-agent orchestration — task decomposition, parallel execution, and progress dashboard |
| `memory_pane.rs` | Memory panel — project index, user preferences, and stored memory snippets |
| `automation_pane.rs` | Automations panel — create, edit, delete, and run automations |
| `terminal_pane.rs` | Embedded terminal panel — PTY-backed shell streaming through backend events |
| `settings.rs` | Config editor — reads/writes `config.toml`, `tui.toml`, `mcp.json`, and `AGENTS.md` |
| `session_modals.rs` | Session-management modals — creation, compact confirmation, and resume-conflict guard |
| `login_modal.rs` | OAuth login modal — streams `kimi login` output lines |
| `permission_modal.rs` | Tool-call approval modal — displays tool title, raw input, and allow/deny options |

## Subdirectories

| Directory | Purpose |
|-----------|---------|
| `base/` | Primitive design-system components — buttons, inputs, badges, icons, dropdowns, toasts, tooltips, etc. (see `base/AGENTS.md`) |
| `icons/` | Inline SVG Lucide-style icon components generated in `lucide.rs` (see `icons/AGENTS.md`) |

## For AI Agents

### Working In This Directory
- Components are plain Rust functions annotated with `#[component]`; they return `Element` via Dioxus RSX.
- State is read from `GlobalSignal`s in `crate::state` — mutate only through `.write()`.
- Async backend calls are wrapped in `spawn(...)` inside event handlers.
- Prefer `class: "foo"` in RSX over inline styles.

### Testing Requirements
- No unit tests for UI components. Validate visually with `cargo tauri dev`.

### Common Patterns
- `use_effect` subscribes to Tauri events once on mount.
- `use_signal` holds local ephemeral state (e.g., draft text).
- `document::eval(...)` is used sparingly for scroll-to-bottom, file downloads, and other DOM effects.
- Components import actions from `crate::actions` rather than calling `invoke` directly.

## Dependencies

### Internal
- `src/state/` — global signals and domain types consumed by every component
- `src/actions.rs` — async action helpers (connect, send prompt, etc.)
- `src/ipc.rs` — Tauri invoke/listen bridge (used mainly in `app.rs`)
- `src/markdown.rs` — `md_to_html` for rendering agent messages in `thread.rs`
- `src/conversation.rs` — context-usage helpers and permission auto-approval logic
- `src/design_tokens/` — colors and design-token constants used by `base/` primitives

### External
- `dioxus` — UI framework (RSX, signals, effects, document API)
- `serde_json` — JSON parsing for ACP payloads and command data
- `js-sys` — Web API access for client-side date handling in `topbar.rs`

<!-- MANUAL: -->
