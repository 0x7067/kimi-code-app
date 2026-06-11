<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-06-11 | Updated: 2026-06-11 -->

# state

## Purpose
Global application state for the Dioxus frontend. Contains domain types (`model.rs`), the global `GlobalSignal` declarations (`signals.rs`), and the ACP `session/update` reducer (`reducer.rs`). UI components read from these signals; the reducer and user actions mutate them in response to backend events.

## Key Files

| File | Description |
|------|-------------|
| `mod.rs` | Barrel export — re-exports `model::*`, `reducer::*`, `signals::*` |
| `model.rs` | Domain types — `Item`, `ToolCall`, `PlanEntry`, `SessionMeta`, `PermissionRequest`, `SlashCommand`, `Attachment`, `View`, `TaskTemplate`, `Automation`, `ApprovalPrefs`, `AppSettings` |
| `signals.rs` | Global `GlobalSignal` declarations — connection, login, project/session lists, thread items, plan, commands, running state, permissions, attachments, settings, diff/terminal/browser/memory panels, scrollback cache |
| `reducer.rs` | `apply_update` — parses ACP `session/update` payloads and mutates signals (message chunks, tool calls, plan updates, context usage, etc.) |

## Subdirectories

*None.*

## For AI Agents

### Working In This Directory
- All signals are `pub static` and must be mutated through `.write()`.
- The reducer is the single source of truth for turning ACP notifications into UI state changes.
- When adding new domain concepts, define the type in `model.rs`, add a signal in `signals.rs`, and handle updates in `reducer.rs`.

### Testing Requirements
- No automated tests. Validate by running `cargo tauri dev` and exercising ACP flows.

### Common Patterns
- `reset_thread()` in `signals.rs` clears items, plan, commands, permission, running state, context usage, and injected-memory count — used when switching sessions.
- `content_text()` in `reducer.rs` recursively flattens ACP content blocks (string, array, or object with `text`/`content`) into plain text.
- `push_chunk()` appends streaming text to the last message of the same kind, avoiding a new element per token.

## Dependencies

### Internal
- `src/components/` — reads from these signals to render UI
- `src/actions.rs` — mutates signals directly for user-initiated actions
- `src/conversation.rs` — used by the reducer to parse context-usage payloads

### External
- `dioxus` — `GlobalSignal`, `Signal::global`, `ReadableExt`
- `serde_json` — parsing ACP notification payloads

<!-- MANUAL: -->
