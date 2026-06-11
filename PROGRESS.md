# Implementation Progress — kimi-code-app

> Living tracker for the DESIGN_SYSTEM.md + REQUIREMENTS.md build-out.
> Audience: any agent or human picking up this work. Keep it current: update status
> when a phase lands, record decisions and verified facts, commit at each checkpoint.

**Last updated:** 2026-06-11
**Build state:** Workspace green (`cargo check --workspace`, `cargo test --workspace` = 37 passed). Frontend hot-reloads via `cargo tauri dev`. Screenshots taken of MCP Servers UI + Add Server modal.

## How to work this plan

- TDD where logic is testable (protocol parsing, queues, reducers). Manual `cargo tauri dev` for visuals.
- Verify with: `cargo check --workspace`, `cargo check --target wasm32-unknown-unknown`, `cargo test --workspace`.
- Git commit per checkpoint with a green workspace. Never commit a red build.
- User has pre-approved adding any dependency.
- Frontend: Dioxus 0.7 RSX, state via GlobalSignals in `src/state/signals.rs`, styles in `assets/main.css` (NO Tailwind pipeline — use semantic classes), tokens in `src/design_tokens/`.
- Known rsx quirk: interpolating module-level consts/fns inside attribute strings triggers false-positive `dead_code`; annotate `#[allow(dead_code)] // used via rsx attribute interpolation`.

## Verified protocol facts (do NOT re-derive; live-tested against kimi CLI 0.14.0)

- ACP `protocolVersion` is **integer 1** (REQUIREMENTS.md's "2025-03-26" is wrong — that's an MCP version).
- initialize response advertises `loadSession: true`, `sessionCapabilities: {list:{}, resume:{}}`, `promptCapabilities: {image:true, embeddedContext:true}`, `mcpCapabilities: {http:true, sse:false}`.
- Sessions persist in `~/.kimi-code/`: `session_index.jsonl` (sessionId/sessionDir/workDir per line) + `sessions/wd_<dir>_<hash>/session_<uuid>/{state.json, agents/main/wire.jsonl}`.
- Cancel: send `session/cancel` notification → in-flight `session/prompt` resolves with `stopReason: "cancelled"` (not an error). Sessions can't be loaded mid-turn (TURN_AGENT_BUSY).
- Exactly one in-flight `session/prompt` per session; a second is rejected ("another turn is active"). Steering over ACP = cancel, await cancelled, re-prompt. Kimi's native steer API is not exposed over ACP stdio.
- Full notes in basic-memory: `solutions/kimi-acp-capabilities-cli-0-14-0-verified-facts`.

## Checkpoints

| # | Scope | Status | Commit |
|---|-------|--------|--------|
| 1 | Design system wired into build (tokens, 11 base components, icons, token tests) | DONE | `fd67389` |
| 2 | Design system 18/18 acceptance criteria + F-001 ACP core TDD (25 backend tests: protocol routing, integer-v1 negotiation + capability capture, MessageQueue + per-session TurnQueue, crash Supervisor w/ backoff+replay, SessionRegistry, acp_cancel command) | DONE | `db5d161` |
| 3 | F-002 chat UI (shortcuts, status bar w/ context colors, copy, role colors, search, MD/JSON export, @mention scaffold; 19 UI tests) + F-012 backend (store parsing, list/load commands, sessions:changed watcher; 37 backend tests) | DONE | `2bdfde0` |
| 4 | F-013 stop (Stop button, Esc, cancelled marker) + F-014 queue (chips, ⌥⏎, FIFO dispatch) + F-015 steer (acp_steer cancel→reprompt w/ 5s timeout; default on send-while-running) + F-012 frontend (synced sidebar list, sessions:changed listener, load replay, relative times). 62 tests. NOTE: steer timing needs live smoke test in `cargo tauri dev`. | DONE | `41b4509` |
| 5 | F-003: AGENTS.md detect+preview (`read_agents_md`), NewSessionModal (name/cwd/initial prompt/AGENTS.md preview), project-grouped session tree, manual /compact w/ confirm, background-sessions panel, resume-conflict guard (`kimi_session_activity`, 30s threshold). 74 tests. Live-verify: /compact handling, wire.jsonl layout. | DONE | `c01aa4a` |
| 6 | F-011: binary autodetect (`detect_kimi_binary` + override applied to all spawns), auth status (file-presence check; CLI has no logout), model selector (config.toml parse + set_default_model), thinking default (always/never/ask), per-tool approval prefs + YOLO (auto-approve short-circuit), app_settings.json atomic store loaded at startup. 90 tests. | DONE | `69a33ab` |
| 6b | Smoke test round 1: caught + fixed startup wasm abort (ipc.rs threw synchronously when Tauri bridge absent/throwing → poisoned wasm-bindgen futures executor → "RefCell already borrowed"; now resolved via js_sys::Reflect, all errors flow through Err paths). Headless UI renders clean. Still TODO live (needs unlocked Mac + real backend): steer timing, session/list shape, /compact, wire.jsonl layout, settings flows, real-window screenshots. | PARTIAL | `1dc1c10` |
| 7 | F-005 MCP backend (mcp.rs: parse/upsert/remove servers, transport detection, validation, filtering) + F-010 Terminal backend (portable-pty PTY spawning, Registry, event streaming). | DONE | `32645ef` |
| 8 | F-005 MCP frontend (structured server list, add/edit modal, status badges, enabled toggle) + backend wiring for mcp + terminal commands. Screenshots verified in `cargo tauri dev`. | DONE | `7c2f215` |
| 9 | F-010 terminal frontend (PTY panel, streaming output, input, Clear/Close) verified in `cargo tauri dev`. | DONE | `TBD` |
| 10 | P1/P2 remaining: F-002 P1 leftovers (search, @mentions, checkpoint), F-007 memory, F-004 multi-agent, F-006 browser, F-009 automations | TODO | — |

## Phase detail

### Design system (DESIGN_SYSTEM.md) — 18/18 criteria PASS (uncommitted)
Agent-verified checklist: tokens match spec (tests in `src/design_tokens/tests.rs`); KimiIcon 4 variants + pulse dot; KimiButton 4 variants/3 sizes/loading/disabled rewritten to `.kimi-btn` CSS (Tailwind classes were dead — no pipeline); KimiInput focus/error/disabled; KimiCard states; KimiToggle 150ms; KimiDropdown open/close anims (gloo-timers unmount); KimiToast auto-dismiss, integrated in `app.rs`; breakpoints 1280px (sidebar→64px icons) / 1440px (right panel overlay) in `assets/main.css`; custom scrollbar; prefers-reduced-motion; all keyframes; no `#4a9eff`; no visible Codex/GPT strings.

### F-001 ACP client core — IN PROGRESS (agent "acp-core" in src-tauri/)
Scope: protocol module with typed message routing (text/tool_call/tool_result/error/status), integer version negotiation + capability capture, MessageQueue (doubles as per-session turn serializer), crash supervisor with injectable transport, concurrent session registry. Corrections about integer version + cancel semantics already sent to the agent.

### F-012 session sync (user requirement, not in REQUIREMENTS.md)
Backend DONE: `acp/store.rs` (index/state.json parsing, workDir filter, title derivation, updatedAt-desc sort; 12 tests), `commands/sessions.rs` (`kimi_list_sessions` ACP-first w/ disk fallback, `kimi_load_session` w/ TURN_AGENT_BUSY mapped error), 2s mtime watcher emitting `sessions:changed`. 37 backend tests green.
Frontend DONE: sidebar session list consuming `kimi_list_sessions` + `sessions:changed`, open-session → `kimi_load_session` replay into thread. Project-grouped tree with collapsible folders, background sessions panel, session search.

### F-013 stop (user requirement)
Backend: `acp_cancel(session_id)` command → `session/cancel` notification; handle `stopReason: cancelled`. Frontend: stop button in composer while turn active; Escape shortcut; preserve partial output.

### F-014 message queueing (user requirement)
Frontend queue UI (pending list, editable/removable) + backend per-session prompt serialization (from F-001.8 queue). Dispatch FIFO on turn end.

### F-015 steering (user requirement)
Send-while-running defaults to steer: `session/cancel` → await cancelled stopReason → immediately `session/prompt` with new message (+ prior context preserved by session). Alt-send or queue toggle enqueues instead (F-014).

### F-002 chat interface — existing: thread.rs (streaming, thoughts, tool calls, plan), composer.rs (slash autocomplete), permission_modal.rs. Gaps: copy-to-clipboard, keyboard shortcuts (Cmd+Enter etc.), in-conversation search, Markdown/JSON export, @mentions, checkpoint/restore, status bar (model, context %, current op).

### F-003 sessions/projects — existing: sidebar projects/sessions, commands/projects.rs. Gaps: AGENTS.md auto-detect+preview at session init, session creation dialog, context usage bar w/ color coding, manual compact, background sessions panel.

### F-011 settings — DONE: settings.rs with Preferences pane (binary autodetect, auth status, model selector, thinking default, per-tool approval prefs, YOLO mode) plus raw config editors for config.toml/tui.toml/mcp.json/AGENTS.md. F-005 MCP Servers structured UI added as a dedicated tab alongside raw mcp.json editor.

### P1/P2 status
- **F-005 MCP** — backend + frontend DONE. Structured server management UI live.
- **F-010 Terminal** — backend + frontend DONE. PTY spawning via portable-pty, Registry, event streaming. Frontend: embedded panel below composer with streaming output, input line with prompt, Clear/Close controls, auto-scroll. Verified live in `cargo tauri dev`. No full xterm emulation — simple text renderer sufficient for command execution and output review.
- **F-002 chat P1** — gaps: in-conversation search, @mentions, checkpoint/restore.
- **F-007 Memory** — not started.
- **F-004 Multi-agent** — not started (worktrees, decomposition, merge UI).
- **F-006 Browser** — not started.
- **F-009 Automations** — not started.
- **F-008 Preview iteration** — P2, optional.

## Decisions log

- 2026-06-11: ACP version negotiation uses integer 1; REQUIREMENTS.md F-001.7 date string treated as spec erratum.
- 2026-06-11: Steering implemented as cancel+reprompt (kimi exposes no steer over ACP stdio).
- 2026-06-11: Session sync built on kimi's own store (ACP session/list + load, disk index as fallback) instead of a parallel app database.
- 2026-06-11: session_index.jsonl mtime watcher (2s) is required for cross-process liveness — ACP has no push notification for sessions created by other processes (CLI runs its own kimi process; session/list is poll-only). Delete the watcher if kimi ever adds a sessions-changed ACP notification.
- 2026-06-11: No Tailwind build step — design system uses semantic CSS classes in assets/main.css.
