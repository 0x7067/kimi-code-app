<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-06-11 | Updated: 2026-06-11 -->

# components

## Purpose
Dioxus UI components, one file per major screen region. Each file defines a `#[component]` function that returns an `Element`. The `mod.rs` barrel-exports everything so `main.rs` can mount the root `App` component and nested views can import siblings.

## Key Files

| File | Description |
|------|-------------|
| `mod.rs` | Barrel export — re-exports all components for the crate |
| `app.rs` | Root `App` component — sets up Tauri event subscriptions and top-level layout |
| `composer.rs` | Message input box with slash-command autocomplete, send, cancel, and attachment support |
| `thread.rs` | Main chat thread — renders streaming agent messages, thoughts, tool calls, and plan tracking |
| `sidebar.rs` | Left sidebar — project/session list, search, new-session button |
| `topbar.rs` | Header bar — project picker, diff toggle, settings button |
| `diff_pane.rs` | Git diff review panel — shows working-tree changes |
| `settings.rs` | Config editor — reads/writes `config.toml`, `tui.toml`, `mcp.json`, `AGENTS.md` |
| `login_modal.rs` | OAuth login modal — streams `kimi login` output lines |
| `permission_modal.rs` | Tool-call approval modal — displays tool title, raw input, and allow/deny options |

## Subdirectories

*None.*

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
- `document::eval(...)` is used sparingly for scroll-to-bottom and other DOM effects.
- Components import actions from `crate::actions` rather than calling `invoke` directly.

## Dependencies

### Internal
- `src/state/` — global signals and domain types consumed by every component
- `src/actions.rs` — async action helpers (connect, send prompt, etc.)
- `src/ipc.rs` — Tauri invoke/listen bridge (used mainly in `app.rs`)
- `src/markdown.rs` — `md_to_html` for rendering agent messages in `thread.rs`

### External
- `dioxus` — UI framework (RSX, signals, effects, document API)
- `serde_json` — JSON parsing for ACP payloads and command data

<!-- MANUAL: -->
