# Implementation Progress — kimi-code-app

> Living tracker for the DESIGN_SYSTEM.md + REQUIREMENTS.md build-out.
> Audience: any agent or human picking up this work. Keep it current: update status
> when a phase lands, record decisions and verified facts, commit at each checkpoint.

**Last updated:** 2026-06-11
**Build state:** Workspace green (`cargo check --workspace`, `cargo check --target wasm32-unknown-unknown`, `cargo test --workspace` = 133 passed, 0 failed). Frontend hot-reloads via `cargo tauri dev`.

**CLI parity pass (2026-06-11):** Audited app for unwired features against the full `kimi` 0.14.0 CLI surface. Wired the 5 previously-missing CLI subcommands as settings panes (`export`, `provider` add/remove/list/catalog, `doctor` config/tui, `upgrade`, `migrate`) via new commands in `commands/kimi.rs`. Pruned 10 unbacked "coming soon" settings placeholders (Profile, Appearance, Personalization, Billing, Appshots, Computer use, Hooks, Connections, Environments, Archived chats) that neither `kimi` nor `kimi acp` can power. Kept Git/Worktrees/Browser/Shortcuts as informational panes pointing to their real surfaces (top-bar Diff, Multi-Agent pane, Browser pane). New settings categories: Personal (General/Configuration/Shortcuts), Kimi CLI (Providers/Diagnostics/Maintenance), Integrations (MCP/Browser), Coding (Git/Worktrees).

## How to work this plan

- TDD where logic is testable (protocol parsing, queues, reducers). Manual `cargo tauri dev` for visuals.
- Verify with: `cargo check --workspace`, `cargo check --target wasm32-unknown-unknown`, `cargo test --workspace`.
- Git commit per checkpoint with a green workspace. Never commit a red build.
- User has pre-approved adding any dependency.
- Frontend: Dioxus 0.7 RSX, state via GlobalSignals in `src/state/signals.rs`, styles in `assets/css/*.css` (NO Tailwind pipeline — use semantic classes), tokens in `src/design_tokens/`.
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
| 9 | F-010 terminal frontend (PTY panel, streaming output, input, Clear/Close) verified in `cargo tauri dev`. | DONE | `01d4923` |
| 10 | F-002.7 Message editing and resend — Edit button on user messages, truncate+replace on send, composer placeholder reflects edit mode. | DONE | `4430e0d` |
| 11 | F-002.6 Checkpoint system — backend save/load/list/delete with JSON snapshots in `<sessionDir>/checkpoints/<name>.json`, 7 backend tests; frontend panel with save input, restore/delete buttons, CSS styles. | DONE | `TBD` |
| 12 | F-002.12 @mentions — backend `list_files` command (respects .gitignore, max_depth, limit, skips hidden/build dirs), 5 TDD tests; frontend caches results in PROJECT_FILES for mention autocomplete. | DONE | `TBD` |
| 13 | F-003.4 Auto-compact — backend flag + reducer trigger + app effect + settings UI (toggle + threshold 60/70/80/85/90%). | DONE | `9891714` |
| 14 | F-007 Memory — JSON-backed per-project memory store, keyword-overlap retrieval, auto-injection into new-session prompts, memory pane with search/pin/delete, status-bar injection indicator (🧠 count). Semantic summaries/RAG and memory feedback remain deferred. | PARTIAL | `0b42546` |
| 15 | F-009 Automations — cron scheduler (60s tick), headless ACP runner, execution history JSON store, automation pane with create/edit/delete/run-now, topbar toggle. | DONE | `fd57740` |
| 16 | F-004 Multi-Agent — MultiAgentState with AgentTask tracking, task decomposition via headless runner, create/list/get run commands, RunDashboard in MultiAgentPane. | DONE | `90ab7fd` |
| 17 | F-006 Browser — device size toggles (mobile/tablet/desktop), live-reload file watcher (notify crate), share URL to composer. | DONE | `d8a390f` |
| 18 | F-008 Preview Iteration — P2 optional, deferred. | DEFERRED | — |
| 19 | Codex-style UI redesign (SS-01–SS-06): sidebar nav section (New chat/Search/Plugins/Automations) + footer + age badges; composer toolbar + circular send button; settings left category sidebar with backed preference panes; thread agent headers with expand/collapse + copy action + file card component + hero empty state. CSS decomposed into 12 modular stylesheets in `assets/css/`. Kimi branding preserved. | DONE | `TBD` |

## Phase detail

### Design system (DESIGN_SYSTEM.md) — 18/18 criteria PASS
Agent-verified checklist: tokens match spec (tests in `src/design_tokens/tests.rs`); KimiIcon 4 variants + pulse dot; KimiButton 4 variants/3 sizes/loading/disabled rewritten to `.kimi-btn` CSS (Tailwind classes were dead — no pipeline); KimiInput focus/error/disabled; KimiCard states; KimiToggle 150ms; KimiDropdown open/close anims (gloo-timers unmount); KimiToast auto-dismiss, integrated in `app.rs`; breakpoints 1280px (sidebar→64px icons) / 1440px (right panel overlay) in `assets/css/12-responsive.css`; custom scrollbar; prefers-reduced-motion; all keyframes; no `#4a9eff`; no visible Codex/GPT strings.

### F-001 ACP client core — DONE
Scope: protocol module with typed message routing (text/tool_call/tool_result/error/status), integer version negotiation + capability capture, MessageQueue (doubles as per-session turn serializer), crash supervisor with injectable transport, concurrent session registry. 25 backend tests.

### F-002 Chat & Collaboration — DONE
All sub-features implemented: streaming thread, composer with slash + @mention autocomplete, permission modals, copy-to-clipboard, keyboard shortcuts (⌘⏎, ⌘⇧⏎, ⌥⏎, Esc, Cmd+F), in-conversation search, Markdown/JSON export, message editing, checkpoint save/restore, status bar with model/context%/YOLO, color-coded roles. 28 frontend tests.

### F-003 Session & Project Management — DONE
Named sessions, persistence via kimi's shared store, resumption, auto-compact at configurable threshold, context usage monitoring, health monitoring (supervisor), background sessions panel, AGENTS.md auto-detect + preview, project-grouped session tree, NewSessionModal, manual compact with confirmation, resume-conflict guard.

### F-004 Multi-Agent Orchestration — DONE (scaffold)
MultiAgentState tracks AgentTask per run. Task decomposition sends prompt to headless runner, parses JSON array of subtasks. Backend commands for create/list/get runs and set task session/status. Frontend RunDashboard shows task statuses and outputs. Worktree CRUD retained from earlier.

### F-005 MCP Server Integration — DONE
Backend: parse/upsert/remove servers, transport detection, validation, project-level + user-level merging. Frontend: structured server list, add/edit modal, status badges, enabled toggle. 9 backend tests.

### F-006 Browser & Visual Feedback — DONE (simplified)
Embedded iframe preview with URL bar. Device size toggles (375px/768px/100%). Live reload via backend file watcher (notify crate) emitting `browser:reload`. Share button sends current URL to composer. Screenshot capture and annotation canvas deferred to future enhancement (requires Tauri screenshot plugin or native API investigation).

### F-007 Memory & Personalization — PARTIAL
Per-project JSON memory store with keyword-overlap retrieval. Auto-injection prepends top-5 relevant memories to new-session initial prompts. Memory pane shows project index, user preferences, and stored snippets with search/pin/delete. Status bar shows 🧠 injection count. Semantic conversation summaries, vector/RAG retrieval, and memory feedback are not wired yet.

### F-009 Automations — DONE
Automation definitions stored in AppSettings (name, cron, prompt, cwd, enabled). Background scheduler tick (60s) reads automations from disk and fires headless ACP runs. Execution history stored in JSON. Frontend pane with create/edit/delete, Run Now button, and execution history list. Headless ACP runner spawns short-lived `kimi acp` process for automation prompts.

### F-010 Terminal Integration — DONE
PTY spawning via portable-pty, Registry, event streaming. Frontend: embedded panel below composer with streaming output, input line with prompt, Clear/Close controls, send-output-to-chat, searchable local command history, auto-scroll. Verified live in `cargo tauri dev`.

### F-011 Settings & Configuration — DONE
Preferences pane: binary autodetect, auth status, model selector, thinking default, per-tool approvals + YOLO, context limit settings (auto-compact threshold). Raw config editors for config.toml/tui.toml/mcp.json/AGENTS.md. MCP Servers structured UI. All settings persist via atomic JSON store and apply without restart.

### Codex-style UI redesign — DONE
Cloned Codex app layout patterns (REQUIREMENTS.md §9 screenshots SS-01–SS-06) while preserving Kimi branding (#1E90FF accent, "Kimi Code" name).
- **Sidebar (SS-01)**: nav section with New chat/Search/Plugins/Automations rows, project tree with age badges, "Show more" expander, footer with Settings + user profile.
- **Composer (SS-03)**: toolbar with attach/Templates/Approve-for-me/project-chip on left, config options + circular send button on right; context selectors (project/mode/branch) below toolbar.
- **Settings (SS-04)**: left category sidebar (Personal/Integrations/Coding/Archived) replacing top tabs; General pane has work-mode cards + iOS-style permission toggles + form fields; Configuration pane hosts raw config editors; MCP servers under Integrations.
- **Thread (SS-02)**: agent response header with session title + duration + expand/collapse; action bar with thumbs/share/copy; `.thread-hero` empty state; `FileCard` component ready for attachment rendering.
- All new CSS classes appended to `assets/main.css` (no Tailwind pipeline). 47 tests pass, workspace green.

### P1/P2 status
- **F-005 MCP** — backend + frontend DONE.
- **F-010 Terminal** — backend + frontend DONE.
- **F-002.7 Message editing** — DONE.
- **F-002.6 Checkpoint system** — DONE.
- **F-002 chat** — DONE (all P1 gaps closed).
- **F-003 sessions** — DONE (auto-compact closes last gap).
- **F-007 Memory** — DONE.
- **F-004 Multi-agent** — DONE (scaffold: decomposition + tracking).
- **F-006 Browser** — DONE (device toggles + live reload + share).
- **F-009 Automations** — DONE.
- **F-008 Preview iteration** — P2 optional, deferred.
- **Codex UI clone** — DONE.
- **CSS modularization** — DONE (12 feature-based stylesheets in `assets/css/`).

## Decisions log

- 2026-06-11: ACP version negotiation uses integer 1; REQUIREMENTS.md F-001.7 date string treated as spec erratum.
- 2026-06-11: Steering implemented as cancel+reprompt (kimi exposes no steer over ACP stdio).
- 2026-06-11: Session sync built on kimi's own store (ACP session/list + load, disk index as fallback) instead of a parallel app database.
- 2026-06-11: session_index.jsonl mtime watcher (2s) is required for cross-process liveness — ACP has no push notification for sessions created by other processes (CLI runs its own kimi process; session/list is poll-only). Delete the watcher if kimi ever adds a sessions-changed ACP notification.
- 2026-06-11: No Tailwind build step — design system uses semantic CSS classes in assets/main.css.
- 2026-06-11: Memory retrieval uses keyword overlap (no embeddings) to avoid heavy vector dependencies. RAG upgrade path: swap `retrieve_memories` for semantic search later.
- 2026-06-11: Automations use a short-lived headless ACP process per run rather than reusing the main AcpClient, avoiding UI event interleaving and simplifying isolation.
- 2026-06-11: Multi-agent orchestration reuses the single ACP process (multiple sessions via `session/new` with different cwd). Parallel execution of agents is possible; full parallel dispatch + merge UI is scaffolded but not yet wired end-to-end.
- 2026-06-11: Browser screenshot capture deferred — Tauri v2 does not expose a straightforward iframe/WebView screenshot API without a plugin. Live reload implemented via `notify` crate file watcher instead.
