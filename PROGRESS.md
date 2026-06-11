# Implementation Progress — kimi-code-app

> Living tracker for the DESIGN_SYSTEM.md + REQUIREMENTS.md build-out.
> Audience: any agent or human picking up this work. Keep it current: update status
> when a phase lands, record decisions and verified facts, commit at each checkpoint.

**Last updated:** 2026-06-11
**Build state:** UI crate (`kimi-code-app-ui`) green on native + wasm32, 0 warnings, 7/7 tests. `src-tauri` under active modification (F-001 TDD work in flight).

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
| 4 | F-013 stop + F-014 queueing + F-015 steering + F-012 frontend (sidebar session list, load replay) | TODO | — |
| 6 | F-003 session/project management gaps | TODO | — |
| 7 | F-011 settings & configuration | TODO | — |
| 8 | P1/P2: F-004 multi-agent, F-005 MCP, F-006 browser, F-007 memory, F-009 automations, F-010 terminal | TODO | — |

## Phase detail

### Design system (DESIGN_SYSTEM.md) — 18/18 criteria PASS (uncommitted)
Agent-verified checklist: tokens match spec (tests in `src/design_tokens/tests.rs`); KimiIcon 4 variants + pulse dot; KimiButton 4 variants/3 sizes/loading/disabled rewritten to `.kimi-btn` CSS (Tailwind classes were dead — no pipeline); KimiInput focus/error/disabled; KimiCard states; KimiToggle 150ms; KimiDropdown open/close anims (gloo-timers unmount); KimiToast auto-dismiss, integrated in `app.rs`; breakpoints 1280px (sidebar→64px icons) / 1440px (right panel overlay) in `assets/main.css`; custom scrollbar; prefers-reduced-motion; all keyframes; no `#4a9eff`; no visible Codex/GPT strings.

### F-001 ACP client core — IN PROGRESS (agent "acp-core" in src-tauri/)
Scope: protocol module with typed message routing (text/tool_call/tool_result/error/status), integer version negotiation + capability capture, MessageQueue (doubles as per-session turn serializer), crash supervisor with injectable transport, concurrent session registry. Corrections about integer version + cancel semantics already sent to the agent.

### F-012 session sync (user requirement, not in REQUIREMENTS.md)
Backend DONE (uncommitted): `acp/store.rs` (index/state.json parsing, workDir filter, title derivation, updatedAt-desc sort; 12 tests), `commands/sessions.rs` (`kimi_list_sessions` ACP-first w/ disk fallback, `kimi_load_session` w/ TURN_AGENT_BUSY mapped error), 2s mtime watcher emitting `sessions:changed`. 37 backend tests green.
Frontend TODO: sidebar session list consuming `kimi_list_sessions` + `sessions:changed`, open-session → `kimi_load_session` replay into thread. NOTE: `session/list` request param shape unverified live — verify during `cargo tauri dev` smoke test; fallback covers failures.

### F-013 stop (user requirement)
Backend: `acp_cancel(session_id)` command → `session/cancel` notification; handle `stopReason: cancelled`. Frontend: stop button in composer while turn active; Escape shortcut; preserve partial output.

### F-014 message queueing (user requirement)
Frontend queue UI (pending list, editable/removable) + backend per-session prompt serialization (from F-001.8 queue). Dispatch FIFO on turn end.

### F-015 steering (user requirement)
Send-while-running defaults to steer: `session/cancel` → await cancelled stopReason → immediately `session/prompt` with new message (+ prior context preserved by session). Alt-send or queue toggle enqueues instead (F-014).

### F-002 chat interface — existing: thread.rs (streaming, thoughts, tool calls, plan), composer.rs (slash autocomplete), permission_modal.rs. Gaps: copy-to-clipboard, keyboard shortcuts (Cmd+Enter etc.), in-conversation search, Markdown/JSON export, @mentions, checkpoint/restore, status bar (model, context %, current op).

### F-003 sessions/projects — existing: sidebar projects/sessions, commands/projects.rs. Gaps: AGENTS.md auto-detect+preview at session init, session creation dialog, context usage bar w/ color coding, manual compact, background sessions panel.

### F-011 settings — existing: settings.rs edits config.toml/tui.toml/mcp.json/AGENTS.md; login_modal.rs. Gaps: binary autodetect, model selector w/ feature indicators, thinking default, per-tool approval prefs, YOLO mode, apply-without-restart audit.

### P1/P2 (F-004..F-010) — not started. Implement after the above, priority order: F-005 MCP config UI (mcp.json editing exists), F-010 terminal, F-002 P1 leftovers, F-007 memory, F-004 multi-agent (worktrees), F-006 browser preview, F-009 automations. F-008 (P2 preview iteration) last / optional.

## Decisions log

- 2026-06-11: ACP version negotiation uses integer 1; REQUIREMENTS.md F-001.7 date string treated as spec erratum.
- 2026-06-11: Steering implemented as cancel+reprompt (kimi exposes no steer over ACP stdio).
- 2026-06-11: Session sync built on kimi's own store (ACP session/list + load, disk index as fallback) instead of a parallel app database.
- 2026-06-11: No Tailwind build step — design system uses semantic CSS classes in assets/main.css.
