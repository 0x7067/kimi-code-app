# Kimi ACP Desktop Programming App — Requirements Document

**Version:** 1.0  
**Date:** 2026-06-10  
**Status:** Draft  
**Target:** Desktop application wrapping Kimi Code CLI's ACP (Agent Client Protocol) mode

---

## Table of Contents

1. [Scope & Objectives](#1-scope--objectives)
2. [Actors & Use Cases](#2-actors--use-cases)
3. [Functional Requirements](#3-functional-requirements)
4. [Non-Functional Requirements](#4-non-functional-requirements)
5. [Constraints & Assumptions](#5-constraints--assumptions)
6. [Dependencies](#6-dependencies)
7. [Acceptance Criteria Summary](#7-acceptance-criteria-summary)
8. [Glossary](#8-glossary)

---

## 0. Technology Stack & Implementation Order

This application is built with **Tauri** (Rust backend) and **Dioxus** (Rust frontend UI framework). All UI components, state management, and business logic are implemented in Rust. The frontend compiles to WebAssembly and runs inside Tauri's WebView.

| Layer | Technology | Language |
|-------|-----------|----------|
| Desktop shell | Tauri | Rust |
| UI framework | Dioxus | Rust |
| State management | Dioxus Signals | Rust |
| Styling | Tailwind CSS (via Dioxus) | CSS |
| Icons | Lucide (Dioxus bindings) | Rust/SVG |
| Protocol | ACP JSON-RPC over stdio | Rust |

**Key implications:**
- No JavaScript/TypeScript in the frontend — all components are Rust RSX
- No React hooks — use Dioxus signals (`use_signal`, `use_memo`, `use_context`)
- No npm packages — Rust crates via Cargo
- Frontend compiles to WebAssembly via `wasm-bindgen`
- Tauri commands bridge between Dioxus frontend and Rust backend

### Implementation Order (Mandatory)

**Step 1: DESIGN_SYSTEM.md** — Implement the complete design system first (tokens, base components, layout, icons, animations). This is a dependency for all feature work.

**Step 2: REQUIREMENTS.md** — After design system is complete, implement features in priority order (F-001 through F-011).

**Reference documents:**
- [DESIGN_SYSTEM.md](sandbox:///mnt/agents/output/DESIGN_SYSTEM.md) — Design tokens, base components, layout, icons, animations
- [REQUIREMENTS.md](sandbox:///mnt/agents/output/REQUIREMENTS.md) — Feature requirements, use cases, acceptance criteria

---

## 1. Scope & Objectives

### 1.1 Purpose
Build a desktop application that provides an autonomous coding agent experience using Kimi Code CLI's ACP (Agent Client Protocol) mode. The app shall enable developers to interact with Kimi as an autonomous agent capable of reading repositories, executing shell commands, editing files, running tests, and iterating until tasks complete — comparable to OpenAI Codex but powered by Kimi.

### 1.2 Objectives
- Provide a native desktop interface for Kimi ACP that replaces or augments terminal-based interaction
- Enable persistent, resumable agent sessions across app restarts
- Support multi-agent parallel execution for complex tasks
- Integrate external tools via Model Context Protocol (MCP) servers
- Offer visual feedback through embedded browser preview and screenshot-to-vision workflows
- Learn and adapt to user preferences across sessions via a memory system

### 1.3 Out of Scope
- **Computer Use / Desktop Automation:** Kimi ACP does not support OS-level cursor control, screen capture, or desktop application manipulation. This feature (present in Codex) is not implementable.
- **Image Generation:** Kimi models do not generate images. UI mockup/asset generation requires third-party API integration, which is deferred.
- **Mobile Application:** This is a desktop-only application.

---

## 2. Actors & Use Cases

### 2.1 Actors

| Actor | Description |
|-------|-------------|
| **Developer** | Primary user who writes prompts, reviews agent output, approves tool calls, and manages sessions |
| **Agent (Kimi)** | The AI system operating via ACP that executes tasks, requests approvals, and streams responses |
| **MCP Server** | External tool provider (database, browser, documentation) that exposes tools via Model Context Protocol |
| **Automation Scheduler** | Background service that triggers recurring tasks without user intervention |

### 2.2 Use Cases

#### UC-001: Initialize Agent Session
**Actor:** Developer  
**Trigger:** Developer opens a project and starts a new session  
**Flow:**
1. Developer selects project folder
2. App detects or reads `AGENTS.md` for project-specific instructions
3. App spawns Kimi ACP subprocess with project context
4. Agent initializes and reports ready status
5. Developer receives confirmation with model info and context limit

**Post-conditions:** Active ACP session established, ready to receive prompts

---

#### UC-002: Send Prompt and Receive Streaming Response
**Actor:** Developer, Agent  
**Trigger:** Developer types a prompt and submits  
**Flow:**
1. Developer enters prompt in chat input (with optional slash commands or @mentions)
2. App sends prompt to ACP session
3. Agent streams response tokens in real-time
4. Agent may request tool execution (file read, shell command, MCP call)
5. App pauses stream and presents approval modal for destructive operations
6. Developer approves, rejects, or edits the tool call
7. Agent continues execution with result
8. Final response rendered in chat thread

**Post-conditions:** Task completed or paused awaiting further input

---

#### UC-003: Review and Accept File Changes
**Actor:** Developer, Agent  
**Trigger:** Agent proposes file edits via tool call  
**Flow:**
1. Agent sends `tool_call` with file path and diff
2. App renders inline diff block with syntax highlighting
3. Developer reviews changes (accept, reject, or edit)
4. On accept: app forwards approval to ACP, agent applies changes
5. On reject: app forwards rejection with optional reason
6. On edit: developer modifies diff content, then approves edited version

**Post-conditions:** File system modified only with explicit approval

---

#### UC-004: Resume Session After App Restart
**Actor:** Developer  
**Trigger:** Developer reopens app after previous closure  
**Flow:**
1. App loads previously active sessions from persistent store
2. For each session: attempt to reconnect to existing ACP process or spawn new one with `--session <id>`
3. Conversation history restored in chat thread
4. Developer continues from last checkpoint

**Post-conditions:** Zero data loss; session state fully restored

---

#### UC-005: Execute Multi-Agent Parallel Task
**Actor:** Developer, Multiple Agents  
**Trigger:** Developer requests a complex task (e.g., "refactor auth, add API, update tests")  
**Flow:**
1. App uses Kimi to decompose request into independent subtasks
2. App presents decomposed subtasks for developer review/edit
3. Developer confirms or modifies subtask list
4. App creates isolated Git worktree per subtask
5. App spawns independent ACP session per worktree
6. Agents execute in parallel with real-time progress dashboard
7. On completion: app collects diffs, detects conflicts, presents merge UI
8. Developer resolves conflicts and approves final merge

**Post-conditions:** Changes merged into main branch or discarded

---

#### UC-006: Configure and Use MCP Server
**Actor:** Developer, MCP Server  
**Trigger:** Developer wants to extend agent capabilities with external tools  
**Flow:**
1. Developer opens MCP settings panel
2. Developer adds server via configuration (stdio command or HTTP URL)
3. App validates connection and lists available tools
4. During agent execution, MCP tool call requests appear in approval flow
5. Developer approves tool call; app proxies request to MCP server
6. Tool result formatted and displayed in chat

**Post-conditions:** External tools available to agent with explicit approval control

---

#### UC-007: Preview Web Project with Visual Feedback
**Actor:** Developer, Agent  
**Trigger:** Developer wants to see visual changes or request UI modifications  
**Flow:**
1. Developer opens embedded browser preview pane
2. Preview loads local dev server or static files
3. Developer captures screenshot and draws annotations (rectangles, arrows, text)
4. App sends annotated screenshot + prompt to Kimi via vision-enabled ACP message
5. Agent analyzes image and returns code changes
6. Preview auto-refreshes to show changes
7. Diff overlay highlights modified elements

**Post-conditions:** Visual iteration loop complete

---

#### UC-008: Manage User Preferences and Memory
**Actor:** Developer  
**Trigger:** Developer wants to personalize agent behavior  
**Flow:**
1. Developer sets preferences (tech stack, coding style, framework defaults)
2. App stores preferences and injects into new session prompts
3. After sessions complete, app summarizes conversations into memory snippets
4. For new sessions, app retrieves relevant past memories via semantic search
5. Developer reviews memory panel, provides feedback (thumbs up/down), pins explicit instructions

**Post-conditions:** Agent behavior increasingly personalized across sessions

---

#### UC-009: Schedule and Run Automated Task
**Actor:** Developer, Automation Scheduler  
**Trigger:** Developer configures recurring workflow  
**Flow:**
1. Developer creates automation with cron schedule, task template, and data source
2. App validates cron expression and tests data connector
3. At scheduled time, app spawns headless ACP session
4. Automation fetches data (e.g., GitHub commits), sends to agent, receives summary
5. App routes output to configured destination (Slack, email, PR draft)
6. For code-modifying automations, human approval gate intercepts before execution
7. Execution history logged with status, output, and errors

**Post-conditions:** Recurring workflow executes without manual intervention

---

#### UC-010: Use Embedded Terminal
**Actor:** Developer, Agent  
**Trigger:** Developer needs to run shell commands or review terminal output  
**Flow:**
1. Developer opens terminal pane within app
2. Terminal inherits project environment (PATH, env vars, shell config)
3. Developer runs commands; output streams in real-time
4. Developer sends terminal output to chat for agent analysis
5. Agent suggests commands; developer approves execution in terminal
6. Terminal history searchable and linked to chat context

**Post-conditions:** Seamless terminal integration with agent context sharing

---

## 3. Functional Requirements

### 3.1 ACP Client Core (F-001)

**Priority:** P0 — Critical  
**Description:** Native JSON-RPC client for Kimi ACP protocol enabling bidirectional communication with agent processes.

| ID | Requirement | Priority |
|----|-------------|----------|
| F-001.1 | Implement ACP JSON-RPC 2.0 client over stdin/stdout transport | P0 |
| F-001.2 | Support ACP methods: `initialize`, `agent/run`, `agent/approve`, `agent/reject`, `agent/compact`, `agent/exit` | P0 |
| F-001.3 | Handle streaming delta responses (text tokens, tool calls, reasoning traces, status updates) | P0 |
| F-001.4 | Parse and route ACP message types: `text`, `tool_call`, `tool_result`, `error`, `status` | P0 |
| F-001.5 | Support stdio (local) and HTTP (remote) transport modes | P1 |
| F-001.6 | Implement connection health monitoring with automatic reconnection | P0 |
| F-001.7 | Handle ACP protocol version negotiation (current: 2025-03-26) | P0 |
| F-001.8 | Queue outbound messages during temporary disconnection and flush on reconnect | P1 |
| F-001.9 | Support concurrent ACP sessions (one per project/workspace) | P0 |
| F-001.10 | Recover from ACP subprocess crash without data loss (restart + context replay) | P0 |

**Acceptance Criteria:**
- [ ] Initialize ACP session with Kimi Code CLI within 2 seconds
- [ ] Send prompt and receive first streaming token within 2 seconds
- [ ] Handle tool call request and return approved/rejected response correctly
- [ ] Subprocess crash triggers auto-restart and session resumption within 5 seconds
- [ ] 10 concurrent sessions operate without message cross-contamination

---

### 3.2 Chat & Collaboration Interface (F-002)

**Priority:** P0 — Critical  
**Description:** Primary user interface for agent interaction with streaming chat, inline diff review, approval workflows, and checkpoint management.

| ID | Requirement | Priority |
|----|-------------|----------|
| F-002.1 | Real-time streaming message display with token-by-token rendering at 60fps | P0 |
| F-002.2 | Message threading with distinct roles: user, assistant, system, tool | P0 |
| F-002.3 | Inline code diff blocks within chat bubbles with syntax highlighting | P0 |
| F-002.4 | Approval modals for destructive operations (shell, file, MCP, git) | P0 |
| F-002.5 | Thinking mode toggle (Kimi K2.5 Thinking mode) | P0 |
| F-002.6 | Checkpoint system: save/restore conversation state before/after tool execution | P0 |
| F-002.7 | Message editing and resend (modify previous prompt and retry) | P1 |
| F-002.8 | Copy-to-clipboard for code blocks and full responses | P1 |
| F-002.9 | Search within conversation history | P1 |
| F-002.10 | Export conversation as Markdown or JSON | P2 |
| F-002.11 | Slash command support (`/compact`, `/exit`, `/help`) | P0 |
| F-002.12 | @mention support for referencing files, symbols, or previous messages | P1 |
| F-002.13 | Keyboard shortcuts: send (Cmd+Enter), thinking send (Cmd+Shift+Enter), cancel (Escape) | P0 |
| F-002.14 | Agent status bar showing model name, context usage %, current operation | P0 |
| F-002.15 | Color-coded message roles with visual distinction | P1 |

**Acceptance Criteria:**
- [ ] Stream 1000 tokens in under 5 seconds with smooth 60fps rendering
- [ ] Display 500-line diff with syntax highlighting in under 1 second
- [ ] Approval modal blocks execution until user responds (no timeout bypass)
- [ ] Checkpoint restore returns to exact conversation state including pending approvals
- [ ] Thinking mode produces structured reasoning section before final answer

---

### 3.3 Session & Project Management (F-003)

**Priority:** P0 — Critical  
**Description:** Manage multiple ACP sessions per project with persistence, reconnection, and context management across app restarts.

| ID | Requirement | Priority |
|----|-------------|----------|
| F-003.1 | Create named sessions per project/workspace | P0 |
| F-003.2 | Session persistence: save conversation history and agent state to local database | P0 |
| F-003.3 | Session resumption: reconnect to existing ACP process or restart with `--session <id>` | P0 |
| F-003.4 | Context compression: trigger `/compact` automatically when context usage exceeds 80% | P0 |
| F-003.5 | Context usage monitoring: display real-time percentage from ACP status messages | P0 |
| F-003.6 | Session health monitoring: heartbeat checks, auto-restart on crash | P0 |
| F-003.7 | Background session support: sessions continue when app UI is closed | P1 |
| F-003.8 | Session migration: export/import session data | P2 |
| F-003.9 | `AGENTS.md` auto-detection: parse and inject project-specific instructions on session init | P0 |
| F-003.10 | Project tree sidebar showing all projects and their sessions | P0 |
| F-003.11 | Session creation dialog with name, path, initial prompt, and AGENTS.md preview | P0 |
| F-003.12 | Context usage bar with color coding (green <50%, yellow 50-80%, red >80%) | P0 |
| F-003.13 | Manual compact trigger with confirmation showing summary scope | P1 |
| F-003.14 | Background sessions panel with last activity timestamp | P1 |

**Acceptance Criteria:**
- [ ] Create session, close app, reopen — session state fully restored with zero data loss
- [ ] Context at 85% triggers auto-compact without user intervention
- [ ] Background session survives app restart and reconnects on reopen
- [ ] 50 sessions load in sidebar in under 2 seconds
- [ ] AGENTS.md changes detected and reloaded on next session start
- [ ] Atomic database writes prevent corruption on app crash

---

### 3.4 Multi-Agent Orchestration (F-004)

**Priority:** P1 — High  
**Description:** Run multiple Kimi agents in parallel on isolated Git worktrees for complex tasks.

| ID | Requirement | Priority |
|----|-------------|----------|
| F-004.1 | Task decomposition: use Kimi to break requests into independent subtasks | P1 |
| F-004.2 | Worktree isolation: create per-agent Git worktrees before spawning ACP sessions | P1 |
| F-004.3 | Parallel execution: manage 2-8 concurrent ACP subprocesses | P1 |
| F-004.4 | Cross-agent communication: optional message passing between agents | P2 |
| F-004.5 | Merge resolution: collect diffs, detect conflicts, present unified merge UI | P1 |
| F-004.6 | Agent roles: assign specializations (frontend, backend, tests) with role prompts | P1 |
| F-004.7 | Progress aggregation: real-time dashboard showing status and output per agent | P1 |
| F-004.8 | Conflict detection: identify overlapping file modifications between agents | P1 |
| F-004.9 | Graceful degradation: failed agent does not block others | P1 |
| F-004.10 | Editable subtask preview before execution | P1 |
| F-004.11 | Worktree visualization showing branch structure and agent assignments | P2 |
| F-004.12 | Three-pane merge conflict resolver (base, agent A, agent B) | P1 |
| F-004.13 | Per-agent terminal output stream | P1 |

**Acceptance Criteria:**
- [ ] Decompose "refactor auth + add API + update tests" into 3 parallel agents
- [ ] Each agent operates in isolated worktree without file conflicts
- [ ] Merge UI shows all changes with per-file/per-hunk selection
- [ ] Single agent failure allows others to complete; partial merge succeeds
- [ ] Dashboard updates agent status in real-time with under 1 second latency

---

### 3.5 MCP Server Integration (F-005)

**Priority:** P0 — Critical  
**Description:** Full support for Model Context Protocol servers enabling external tools.

| ID | Requirement | Priority |
|----|-------------|----------|
| F-005.1 | MCP server configuration UI: add, edit, remove with transport settings | P0 |
| F-005.2 | Support stdio and HTTP transports (ACP supports both; SSE/acp dropped) | P0 |
| F-005.3 | MCP server marketplace: curated list with one-click install | P1 |
| F-005.4 | Tool approval flow: surface MCP tool calls with parameter preview | P0 |
| F-005.5 | YOLO mode: per-server toggle for auto-approval of non-destructive tools | P1 |
| F-005.6 | MCP server health monitoring: test connection, show status, auto-restart | P1 |
| F-005.7 | Server isolation: per-project MCP configurations | P1 |
| F-005.8 | Tool result rendering: format outputs (tables, JSON, images) in chat | P1 |
| F-005.9 | Authentication handling: OAuth, API keys, token refresh | P1 |
| F-005.10 | MCP server configuration stored in `~/.config/app/mcp/` or project-local `.mcp/` | P0 |

**Acceptance Criteria:**
- [ ] Add Playwright MCP server and use for browser automation via Kimi
- [ ] Tool approval modal blocks execution until user responds
- [ ] YOLO mode auto-approves read-only tools (e.g., documentation lookup)
- [ ] MCP server crash auto-restarts within 5 seconds
- [ ] 20 MCP servers configured without performance degradation
- [ ] OAuth flow completes without leaving app context

---

### 3.6 In-App Browser & Visual Feedback (F-006)

**Priority:** P1 — High  
**Description:** Embedded browser for previewing web projects with screenshot-to-vision pipeline.

| ID | Requirement | Priority |
|----|-------------|----------|
| F-006.1 | Embedded WebView for live preview | P1 |
| F-006.2 | Screenshot capture: full page, visible viewport, or specific element | P1 |
| F-006.3 | Annotation layer: draw rectangles, arrows, text on screenshots | P1 |
| F-006.4 | Vision-to-code pipeline: send annotated screenshots to Kimi via ACP | P1 |
| F-006.5 | Live reload: auto-refresh preview when files modified | P1 |
| F-006.6 | Responsive preview: toggle device sizes (mobile, tablet, desktop) | P1 |
| F-006.7 | Element inspector: click to get CSS selector for targeted edits | P1 |
| F-006.8 | Diff overlay: highlight changed elements after modifications | P2 |
| F-006.9 | Multi-page support: navigate and preview multiple routes | P2 |
| F-006.10 | Split view: code editor / preview / chat (configurable layout) | P1 |

**Acceptance Criteria:**
- [ ] Capture screenshot of local dev server and send to Kimi with annotation
- [ ] Kimi receives image and produces relevant CSS changes
- [ ] Preview auto-refreshes within 2 seconds of file modification
- [ ] Annotation canvas supports 10+ drawing operations at 60fps
- [ ] Element inspector extracts correct CSS selector for clicked element

---

### 3.7 Memory & Personalization (F-007)

**Priority:** P1 — High  
**Description:** Cross-session memory system learning user preferences and project context.

| ID | Requirement | Priority |
|----|-------------|----------|
| F-007.1 | User preference store: tech stack, coding style, naming conventions | P1 |
| F-007.2 | Project memory: index repository structure, key files, dependencies | P1 |
| F-007.3 | Conversation summarization: use Kimi to summarize sessions into snippets | P1 |
| F-007.4 | RAG retrieval: embed summaries and docs; retrieve relevant context | P1 |
| F-007.5 | Auto-injection: prepend retrieved memories to prompts without user intervention | P1 |
| F-007.6 | Memory feedback: thumbs up/down on injected memories | P1 |
| F-007.7 | Explicit memory: user pins specific instructions | P1 |
| F-007.8 | Cross-project learning: identify patterns across projects | P2 |
| F-007.9 | Memory decay: deprioritize old memories unless reinforced | P2 |
| F-007.10 | Memory panel: searchable list with source and relevance score | P1 |
| F-007.11 | Preference settings form for tech stack and style guides | P1 |
| F-007.12 | Memory injection indicator showing count of prepended memories | P1 |

**Acceptance Criteria:**
- [ ] After 3 sessions on Dioxus project, agent auto-suggests Dioxus patterns without prompting
- [ ] User pins "use Tailwind" — all future sessions inject this preference
- [ ] Memory retrieval finds relevant past solution for similar error message
- [ ] Feedback system improves memory relevance over time
- [ ] Memory panel loads in under 1 second with 1000+ snippets

---

### 3.8 Preview Iteration System (F-008)

**Priority:** P2 — Medium  
**Description:** Generate multiple implementation approaches for comparison before execution.

| ID | Requirement | Priority |
|----|-------------|----------|
| F-008.1 | Multi-approach generation: 2-4 variations with different temperatures/prompts | P2 |
| F-008.2 | Branch isolation: temporary Git branches per approach | P2 |
| F-008.3 | Diff comparison: side-by-side view against base | P2 |
| F-008.4 | Approach metadata: complexity, file count, test impact | P2 |
| F-008.5 | User selection: pick one, discard others | P2 |
| F-008.6 | A/B testing: apply to separate preview instances | P3 |
| F-008.7 | Approach voting for team members | P3 |
| F-008.8 | Rollback to base if none satisfactory | P2 |

**Acceptance Criteria:**
- [ ] Generate 3 approaches for "implement user auth" in parallel
- [ ] Each approach shown with diff, file count, complexity estimate
- [ ] User selects approach 2 — app continues on that branch, discards others
- [ ] Preview pane shows selected approach immediately

---

### 3.9 Automations (F-009)

**Priority:** P2 — Medium  
**Description:** Schedule recurring AI workflows running headlessly via ACP.

| ID | Requirement | Priority |
|----|-------------|----------|
| F-009.1 | Cron-based scheduling with standard cron expressions | P2 |
| F-009.2 | Task templates: pre-defined prompts for common workflows | P2 |
| F-009.3 | Data connectors: GitHub, GitLab, Jira APIs | P2 |
| F-009.4 | Headless ACP execution without UI | P2 |
| F-009.5 | Output routing: Slack, email, PR draft, issue creation | P2 |
| F-009.6 | Human-in-the-loop: approval gate for code-modifying automations | P2 |
| F-009.7 | Execution history: log status, output, errors | P2 |
| F-009.8 | Failure handling: retry with backoff, alert on persistent failure | P2 |
| F-009.9 | Conditional execution: run only if conditions met | P2 |

**Acceptance Criteria:**
- [ ] Schedule "review Friday commits" every Friday at 9am
- [ ] Automation fetches commits, sends to Kimi, posts summary to Slack
- [ ] Failed automation retries 3 times then alerts user
- [ ] User can approve/reject code changes proposed by automation

---

### 3.10 Terminal Integration (F-010)

**Priority:** P1 — High  
**Description:** Embedded terminal with Kimi-aware shell integration.

| ID | Requirement | Priority |
|----|-------------|----------|
| F-010.1 | Embedded terminal in app window | P1 |
| F-010.2 | Kimi shell integration: command suggestions and completions | P1 |
| F-010.3 | Command sharing: send terminal output to chat for analysis | P1 |
| F-010.4 | Auto-execution: Kimi suggests commands, user approves execution | P1 |
| F-010.5 | Terminal history: searchable command history | P1 |
| F-010.6 | Multi-shell support: bash, zsh, fish, PowerShell | P1 |
| F-010.7 | Environment sync: inherit project PATH and env vars | P1 |
| F-010.8 | Split terminal: multiple terminals side-by-side | P2 |

**Acceptance Criteria:**
- [ ] Open terminal, run `npm test`, send output to Kimi for error analysis
- [ ] Kimi suggests fix and offers to run command in terminal
- [ ] Terminal inherits correct Node version from project `.nvmrc`

---

### 3.11 Settings & Configuration (F-011)

**Priority:** P0 — Critical  
**Description:** Comprehensive settings for Kimi binary, auth, models, behavior, and integrations.

| ID | Requirement | Priority |
|----|-------------|----------|
| F-011.1 | Kimi binary path: auto-detect or manual set | P0 |
| F-011.2 | Authentication: manage login state, token refresh, account switching | P0 |
| F-011.3 | Model selection: choose Kimi models with feature indicators | P0 |
| F-011.4 | Thinking mode default: always/never/ask | P0 |
| F-011.5 | Approval preferences: per-tool-type settings | P0 |
| F-011.6 | YOLO mode: global and per-project auto-approval | P1 |
| F-011.7 | Context limit: warning thresholds and auto-compact behavior | P0 |
| F-011.8 | Keyboard shortcuts: customizable | P1 |
| F-011.9 | Theme: light/dark/system with custom accents | P1 |
| F-011.10 | Export/import: backup and restore settings | P2 |
| F-011.11 | Proxy: HTTP/HTTPS proxy for corporate environments | P1 |
| F-011.12 | Telemetry: opt-in usage analytics and error reporting | P2 |
| F-011.13 | Settings apply without app restart | P0 |

**Acceptance Criteria:**
- [ ] Auto-detect Kimi binary in common locations (PATH, /usr/local/bin, etc.)
- [ ] Detect missing auth and prompt user to login
- [ ] Switch model mid-session without data loss
- [ ] All settings persist across app restarts

---

## 4. Non-Functional Requirements

### 4.1 Performance

| ID | Requirement | Target |
|----|-------------|--------|
| NF-001 | App cold start time | < 3 seconds |
| NF-002 | ACP session initialization | < 2 seconds |
| NF-003 | Chat message streaming render | 60 fps |
| NF-004 | File tree load (10k files) | < 1 second |
| NF-005 | Project search (1M LOC) | < 2 seconds |
| NF-006 | Memory retrieval (top-5) | < 200 ms |
| NF-007 | Session state save | < 100 ms |
| NF-008 | Settings apply | < 500 ms |
| NF-009 | MCP tool call (local stdio) | < 2 seconds |
| NF-010 | MCP tool call (HTTP) | < 5 seconds |
| NF-011 | Agent spawn (multi-agent) | < 3 seconds per agent |
| NF-012 | Screenshot capture | < 1 second |
| NF-013 | Preview refresh after file change | < 2 seconds |
| NF-014 | Memory panel load (1000 snippets) | < 1 second |

### 4.2 Reliability

| ID | Requirement | Target |
|----|-------------|--------|
| NF-015 | Crash recovery: zero data loss for active sessions | 100% |
| NF-016 | Auto-save interval | 5 seconds |
| NF-017 | ACP connection uptime | 99.9% |
| NF-018 | Atomic database writes | All writes |
| NF-019 | Offline mode: read-only cached data | Supported |
| NF-020 | Auto-update with rollback on failure | Supported |
| NF-021 | Session restoration after app crash | 100% success |

### 4.3 Security

| ID | Requirement | Target |
|----|-------------|--------|
| NF-022 | Kimi auth tokens stored in OS keychain | Mandatory |
| NF-023 | MCP server credentials isolated per-project | Mandatory |
| NF-024 | Shell command approval for destructive ops (rm, git push, etc.) | Default on |
| NF-025 | Sandbox option: Docker container for untrusted projects | Supported |
| NF-026 | Audit log: all agent actions with timestamp and approval status | Mandatory |
| NF-027 | No plaintext credential storage in logs or database | Mandatory |
| NF-028 | MCP server network access restricted to configured endpoints | Mandatory |

### 4.4 Accessibility

| ID | Requirement | Target |
|----|-------------|--------|
| NF-029 | WCAG 2.1 AA compliance | Mandatory |
| NF-030 | Keyboard-only navigation | Supported |
| NF-031 | Screen reader compatible chat interface | Supported |
| NF-032 | High contrast theme option | Supported |
| NF-033 | Font size adjustment | Supported |

### 4.5 Compatibility

| ID | Requirement | Target |
|----|-------------|--------|
| NF-034 | macOS support (primary) | 12.0+ |
| NF-035 | Windows support | 10/11 |
| NF-036 | Linux support | Ubuntu 22.04+, Fedora 38+ |
| NF-037 | Kimi Code CLI version compatibility | Latest stable |
| NF-038 | MCP protocol version | 2024-11-05 |
| NF-039 | Git version | 2.30+ |

---

## 5. Constraints & Assumptions

### 5.1 Kimi ACP Constraints

| Constraint | Impact | Mitigation |
|-----------|--------|-----------|
| No desktop automation (Computer Use) | Cannot implement Codex's cursor/screen control | Build separate OS-level automation or defer feature |
| No image generation | Cannot generate UI mockups/assets natively | Integrate DALL-E/Stable Diffusion via third-party API |
| MCP SSE and `acp` transport types dropped | Some MCP servers incompatible | Filter marketplace; use stdio/HTTP alternatives |
| Requires prior `kimi login` in terminal | User must authenticate before app use | Detect auth state; prompt terminal login if missing |
| ~200k token context limit (K2.5) | Long sessions need compaction | Auto-compact at 80%; manual compact UI |
| No persistent memory across sessions | Must build custom memory layer | Implement RAG-based memory system |
| ACP JSON-RPC over stdio only | No native HTTP server mode in CLI | Manage subprocess lifecycle carefully |

### 5.2 Technical Assumptions

| ID | Assumption |
|----|-----------|
| A-001 | Kimi Code CLI is installed and available in system PATH or configured path |
| A-002 | User has valid Kimi authentication (via `kimi login`) |
| A-003 | Target projects are Git repositories (required for worktree/multi-agent features) |
| A-004 | Projects have read/write filesystem access |
| A-005 | Network access available for MCP HTTP servers and Kimi API |
| A-006 | Sufficient disk space for SQLite database, session cache, and worktrees |

---

## 6. Dependencies

### 6.1 External Dependencies

| Dependency | Version | Purpose | Required By |
|-----------|---------|---------|-------------|
| Kimi Code CLI | Latest stable | ACP protocol backend | F-001, All features |
| Git | 2.30+ | Worktree management, diff generation | F-004, F-008 |
| Node.js | 18+ | Frontend runtime (if using Electron/Tauri) | All frontend |
| Rust | 1.75+ | Tauri backend compilation | F-001, F-003, F-004 |

### 6.2 Runtime Dependencies

| Dependency | Purpose | Features |
|-----------|---------|----------|
| SQLite | Session persistence, memory storage | F-003, F-007 |
| Vector DB or fastembed | Semantic memory retrieval | F-007 |
| xterm.js | Terminal rendering | F-010 |
| node-pty / portable-pty | Pseudo-terminal backend | F-010 |
| diff2html | Diff visualization | F-002, F-004 |
| shiki / prismjs | Syntax highlighting | F-002 |
| react-virtuoso | Virtualized chat lists | F-002 |
| fabric.js | Screenshot annotation canvas | F-006 |

---

## 7. Acceptance Criteria Summary

### 7.1 MVP (Phase 1) — Must Pass

| Criterion | Test Method |
|-----------|-------------|
| Initialize ACP session in <2s | Timer on session creation |
| Stream 1000 tokens in <5s | Stopwatch on prompt submission |
| Approve/reject tool calls blocks execution | Manual test with shell command request |
| Session survives app restart | Close app mid-conversation, reopen, verify state |
| AGENTS.md auto-loaded | Create AGENTS.md, start session, verify instructions injected |
| Settings persist without restart | Change theme, verify immediate application |

### 7.2 Full Product (Phase 2-4) — Must Pass

| Criterion | Test Method |
|-----------|-------------|
| Multi-agent: 3 parallel agents complete without conflict | Run decomposition task, verify isolated worktrees |
| MCP: Playwright server browses and returns DOM | Add server, request page scrape, verify result |
| Memory: Agent suggests Dioxus patterns after 3 sessions | Complete 3 Dioxus sessions, start 4th, verify suggestion |
| Browser: Screenshot + annotation produces CSS changes | Annotate button size, verify Kimi returns CSS |
| Automation: Weekly commit review posts to Slack | Schedule task, verify Slack message at trigger time |
| Terminal: `npm test` output sent to chat and analyzed | Run command, send output, verify agent analysis |

---

## 8. Glossary

| Term | Definition |
|------|-----------|
| **ACP** | Agent Client Protocol — JSON-RPC 2.0 protocol used by Kimi Code CLI for agent communication |
| **AGENTS.md** | Convention file in project root containing project-specific instructions for the agent |
| **Checkpoint** | Saved conversation state at a specific point, enabling rollback |
| **Context** | The token window available to the model; measured as percentage of maximum |
| **Context Compaction** | Summarizing older conversation turns to free up token space |
| **Git Worktree** | Linked working tree allowing multiple branches checked out simultaneously |
| **MCP** | Model Context Protocol — standard for connecting AI models to external tools |
| **RAG** | Retrieval-Augmented Generation — fetching relevant documents to augment prompts |
| **Session** | A persistent conversation context with an ACP agent, including history and state |
| **SKILL.md** | Convention file defining reusable agent skills (instructions + scripts) |
| **Thinking Mode** | Kimi K2.5 feature producing step-by-step reasoning before final answer |
| **YOLO Mode** | Auto-approval of tool calls without manual confirmation |
| **Worktree** | Isolated Git working directory for parallel agent execution |

---

*End of Requirements Document*

---

## 9. UI Reference: Codex Screenshots

The following screenshots from the Codex application serve as the primary visual reference for UI design, layout, interaction patterns, and feature presentation. All UI requirements in this document should be interpreted through the lens of these reference images.

### 9.1 Screenshot Inventory

| Screenshot | Filename | Primary UI Elements Shown | Referenced By |
|-----------|----------|--------------------------|---------------|
| **SS-01** | `Screenshot-2026-06-10-11.57.40 PM.png` | Main chat interface, sidebar navigation, project tree, chat input with approval controls, usage/billing notification | F-002, F-003, F-011 |
| **SS-02** | `Screenshot-2026-06-10-11.57.47 PM.png` | Agent response rendering, findings section with inline code references, verification section, file attachment card, approval UI state, environment panel | F-002, F-005, F-010 |
| **SS-03** | `Screenshot-2026-06-10-11.57.57 PM.png` | Chat input with context selectors (project, work mode, branch), user profile menu, settings access, usage indicator | F-002, F-003, F-011 |
| **SS-04** | `Screenshot-2026-06-10-11.58.00 PM.png` | Settings panel — General settings, work mode selector, permissions toggles, default open destination, language, menu bar options, terminal location | F-011 |
| **SS-05** | `image.png` | Slash command menu (`/`) with command list, icons, descriptions, keyboard navigation | F-002.11 |
| **SS-06** | `image(1).png` | Mention/plugin menu (`@`) with plugin list, file search section, app icons | F-002.12, F-005 |


### 9.2 Detailed UI Analysis by Screenshot

#### SS-01: Main Chat Interface & Sidebar

**Layout Pattern:**
- **Left sidebar** (dark, ~250px): Navigation + Project tree + Chat history
- **Center panel** (darker): Chat thread with centered content
- **Right panel** (optional): Environment/Changes panel

**Sidebar Structure (top to bottom):**
1. **Navigation section:**
   - "New chat" button with pencil icon
   - "Search" with magnifying glass
   - "Plugins" with plugin icon
   - "Automations" with clock icon

2. **Projects section:**
   - Expandable project folders (e.g., `agent-bridge-mcp`, `coding-agent`, `slack`)
   - Each project contains task items with truncated titles and age badges (e.g., "4d", "5d")
   - "Show more" expander for overflow

3. **Chats section:**
   - Individual chat entries with titles and age

4. **Footer:**
   - "Settings" button with gear icon
   - User profile indicator

**Chat Input Area:**
- Centered input box with rounded corners (dark gray, elevated)
- Placeholder text: "Do anything"
- Bottom toolbar:
  - `+` button (add context)
  - "Approve for me" dropdown (with shield icon)
  - Context selector (e.g., "slack" with folder icon)
  - Right side: model selector ("5.5 Medium"), microphone, send button
- **Notification banner:** Inline usage warning ("You're out of Codex messages") with "Upgrade" and "Add Credits" buttons

**Design Tokens to Clone:**
- Background: `#1a1a1a` (main), `#252525` (sidebar), `#2a2a2a` (input, cards)
- Text: `#e0e0e0` (primary), `#888888` (secondary), `#555555` (disabled)
- Accent: `#4a9eff` (blue, active states, links)
- Border radius: 8px (cards), 12px (input), 4px (buttons)
- Spacing: 16px base unit, 8px for compact elements
- Typography: System sans-serif, 14px base, 12px for metadata
- Shadows: subtle elevation on input and cards (0 2px 8px rgba(0,0,0,0.3))

---

#### SS-02: Agent Response & Findings

**Response Structure:**
- **Header:** Task title + "Worked for Xm XXs" duration badge + "Show more" expander
- **Findings section:** Structured analysis with:
  - Inline code references (monospace, blue links to files)
  - File path links (e.g., `server.rs (line 2096)`) — clickable, open in editor
  - Inline code blocks (backtick style, dark background)
  - Numbered lists for sequential findings
- **Verification section:**
  - "Ran:" label with command block (dark background, monospace)
  - "Result:" label with outcome (e.g., "7 passed, 140 filtered out")
- **File attachment card:**
  - Document icon + filename (e.g., "spec.md")
  - File type label ("Document · MD")
  - "Open in" dropdown button
- **Action bar:** Thumbs up/down, share, copy buttons below content
- **Follow-up input:** Same chat input pattern as SS-01, with "5.5 High" priority indicator

**Environment Panel (right sidebar):**
- "Environment" header with settings gear
- "Changes" with +318/-0 diff summary
- "Local" dropdown (branch selector)
- "main" branch indicator with commit icon
- "Commit or push" button
- "Sources" section with file icons

**Interaction Patterns:**
- Clickable file references open file in external editor or inline preview
- Command blocks are copyable on click
- Findings are collapsible ("Show more" / "Show less")
- File cards have hover state with action buttons

---

#### SS-03: Chat Input with Context Selectors

**Enhanced Input Toolbar:**
- Project selector: "agent-bridge-mcp" with folder icon + dropdown chevron
- Work mode selector: "Work locally" with computer icon + dropdown
- Branch selector: "main" with git branch icon + dropdown
- All selectors inline, horizontally arranged below input

**User Profile Menu (bottom-left):**
- Avatar + email
- "Personal account" label
- "Profile" option
- "Settings" option with keyboard shortcut hint
- "Usage remaining 0%" with chevron
- "Log out" option

**Key Pattern:** Context is always visible and switchable from the chat input area — user never loses awareness of what project, mode, and branch the agent is operating on.

---

#### SS-04: Settings Panel — General

**Settings Layout:**
- **Left sidebar:** Settings categories (Personal, Integrations, Coding, Archived)
  - Personal: General, Profile, Appearance, Configuration, Personalization, Keyboard shortcuts, Usage & billing
  - Integrations: Appshots, MCP servers, Browser, Computer use
  - Coding: Hooks, Connections, Git, Environments, Worktrees
  - Archived: Archived chats
- **Right panel:** Settings content for selected category

**General Settings Content:**
1. **Work mode** (card selection):
   - "For coding" — selected, blue dot indicator, "More technical responses and control"
   - "For everyday work" — unselected, "Same power, less technical detail"

2. **Permissions** (toggle list):
   - "Default permissions" — toggle on, description: "By default, Codex can read and edit files in its workspace. It can ask for additional access when needed"
   - "Auto-review" — toggle on, description with "Learn more" link
   - "Full access" — toggle on, warning description about elevated risks, "Learn more" link

3. **General** (form fields):
   - "Default open destination" — dropdown ("Zed")
   - "Language" — dropdown ("Auto Detect")
   - "Show in menu bar" — toggle on, description about macOS menu bar
   - "Bottom panel" — toggle on, description about app header control
   - "Default terminal location" — segmented control ("Bottom" / "Right")
   - "Prevent sleep while running" — toggle (partially visible)

**Design Tokens for Settings:**
- Section headers: 18px, bold, `#e0e0e0`
- Setting labels: 14px, medium weight, `#e0e0e0`
- Descriptions: 12px, `#888888`
- Cards: `#252525` background, 8px radius, 1px `#333333` border
- Toggles: iOS-style, blue when on, gray when off
- Dropdowns: dark field, chevron icon, 8px radius


#### SS-05: Slash Command Menu & Chat Context

**Source:** `image.png` — Chat input with slash command dropdown expanded  
**New UI Elements:**

**Slash Command Menu (dropdown overlay):**
- Triggered by `/` character in input
- Dark overlay panel (elevated above input, `#2a2a2a` background)
- Rounded corners (12px), subtle shadow
- Each command row:
  - Left: Icon (circular or outlined, ~20px, muted color)
  - Center: Command name (bold, 14px, `#e0e0e0`)
  - Right: Description (regular, 13px, `#888888`)
- Commands listed:
  - `Chat` — "Don't work in a project" (chat icon)
  - `Code review` — "Review unstaged changes or compare against a branch" (target icon)
  - `Fast` — "1.5x speed, increased usage" (zap/lightning icon)
  - `Feedback` — "Send feedback about this chat" (message-square icon)
  - `Goal` — "Set a goal that Codex will keep working towards" (target/bullseye icon)
  - `MCP` — "Show MCP server status" (plug icon)
  - `Memories` — "Use on, generate on" (brain/clock icon)
  - `Model` — "GPT-5.5" (cube/box icon)
  - `New worktree` — "Run this chat in a new worktree" (git-branch icon)
  - `Personality` — "Choose how Codex responds" (smile icon)
  - `Pet` — "Wake or tuck away the desktop pet" (cat/paw icon)
- Selected command highlighted with subtle background shift (`#333333`)
- Keyboard navigation: Up/Down arrows, Enter to select, Escape to close

**Chat Input State (with `/` typed):**
- Input shows `/` character with cursor
- Placeholder text disappears
- Bottom toolbar remains visible below input
- Context selectors (project, work mode, branch) still visible below toolbar

**Design Tokens:**
- Command menu max-height: ~400px with scroll
- Row height: 44px
- Icon color: `#888888` (muted)
- Selected row: `#333333` background, `#e0e0e0` text
- Divider: none between rows (clean list)

---

#### SS-06: Plugin/Mention Menu & File Search

**Source:** `image(1).png` — Chat input with `@` mention dropdown and plugin list  
**New UI Elements:**

**Mention/Plugin Menu (dropdown overlay):**
- Triggered by `@` character in input
- Two sections in dropdown:

**Section 1: Plugins**
- Header: "Plugins" (13px, `#888888`, uppercase or small caps)
- Plugin rows:
  - Left: App icon (colored, ~24px, rounded square — e.g., Browser has blue/white globe)
  - Center: Plugin name (bold, 14px, `#e0e0e0`)
  - Right: Description (13px, `#888888`)
- Plugins listed:
  - `Browser` — "Control the in-app browser with Codex" (globe/window icon)
  - `Computer` — "Control Mac apps from Codex" (monitor/desktop icon)
  - `Slack` — "Read and manage Slack" (Slack logo/colored hash icon)

**Section 2: Files**
- Header: "Files" (13px, `#888888`)
- Subtext: "Type to search for files" (13px, `#555555`, italic or muted)
- No file rows shown (empty state awaiting search input)
- Search behavior: typing after `@` filters file list in real-time

**Input State (with `@` typed):**
- Input shows `@` character with cursor
- Mention menu overlays input area
- Same notification banner and toolbar as SS-05

**Interaction Patterns:**
- `@` triggers mention menu immediately
- Typing after `@` filters files (fuzzy search)
- Selecting a plugin or file inserts reference into input
- Enter or click to select, Escape to close
- Arrow keys navigate between sections and items

**Design Tokens:**
- Section header: 12px, `#666666`, letter-spacing 0.5px
- Plugin icon: 24px rounded square, colored per app (Browser=blue, Computer=purple, Slack=multi)
- File row (when populated): icon + filename + path (muted)
- Empty state: centered text, subtle, inviting search
- Menu width: matches input width (~600px) or slightly wider

---

---

### 9.3 UI Cloning Instructions for Each Feature

When implementing any feature in this requirements document, the following UI cloning rules apply:

**Rule 1: Color Palette**
All UI elements must use the dark theme palette derived from Codex:
- Darkest background: `#0d0d0d` (deepest layers)
- Dark background: `#1a1a1a` (main app background)
- Elevated surface: `#252525` (sidebars, cards, inputs)
- Hover/active surface: `#2a2a2a` (hover states, selected items)
- Border/divider: `#333333` (subtle separators)
- Primary text: `#e0e0e0`
- Secondary text: `#888888`
- Tertiary/disabled: `#555555`
- Accent blue: `#4a9eff` (active, links, selected)
- Accent green: `#4ade80` (success, additions)
- Accent red: `#f87171` (error, deletions, warnings)
- Accent yellow: `#facc15` (warnings, pending)

**Rule 2: Typography**
- Font family: System UI stack (`-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif`)
- Base size: 14px / 1.5 line-height
- Small: 12px (metadata, badges, timestamps)
- Large: 16px (section headers)
- XL: 20px (page titles)
- Monospace: `JetBrains Mono, Fira Code, monospace` for code

**Rule 3: Spacing & Layout**
- Base unit: 8px
- Sidebar width: 260px fixed
- Chat max-width: 800px centered
- Card padding: 16px
- Section gap: 24px
- Input height: 48px minimum
- Button height: 36px standard, 28px compact

**Rule 4: Components to Clone**

| Component | Source Screenshot | Key Details |
|-----------|------------------|-------------|
| **Sidebar** | SS-01 | Dark, 260px, icon + text nav, project tree with expand/collapse, age badges |
| **Chat Input** | SS-01, SS-03 | Rounded, elevated, toolbar below with context selectors, approval dropdown, model selector |
| **Message Bubble** | SS-02 | No bubble chrome — text flows directly, code blocks dark, file references blue links |
| **File Card** | SS-02 | Document icon, filename, type label, "Open in" dropdown |
| **Approval Modal** | SS-01 | Inline banner style, not blocking overlay — "Approve for me" dropdown with shield icon |
| **Settings Panel** | SS-04 | Two-pane, left nav, right content, card-based sections, toggle lists |
| **Environment Panel** | SS-02 | Right sidebar, changes summary, branch selector, commit button |
| **Notification Banner** | SS-01 | Inline, dark elevated, icon + text + action buttons |
| **Command Block** | SS-02 | Dark background, monospace, copyable, "Ran:" / "Result:" labels |
| **Context Selector** | SS-03 | Inline dropdowns below input, icon + text + chevron |
| **Slash Command Menu** | SS-05 | Dark overlay, icon + name + description rows, keyboard navigation, 12px rounded |
| **Mention/Plugin Menu** | SS-06 | Two-section dropdown (Plugins + Files), colored app icons, fuzzy search empty state |

**Rule 5: Animation & Motion**
- Streaming text: immediate render (no typewriter effect), cursor blink during generation
- Panel transitions: 200ms ease-out for sidebar collapse/expand
- Modal/dialog: 150ms fade + 8px slide up
- Toast notifications: 300ms slide in from top, auto-dismiss 5s
- Loading states: subtle pulse on buttons, spinner on send icon

**Rule 6: Responsive Behavior**
- Sidebar collapses to icons-only at <1200px width
- Chat panel remains centered, max-width 800px
- Right environment panel hides at <1400px, accessible via toggle
- Input toolbar wraps to multiple lines on narrow widths

---

### 9.4 Feature-to-Screenshot Mapping

| Feature ID | Feature Name | Primary Screenshot | Secondary Screenshot |
|-----------|-------------|-------------------|---------------------|
| F-001 | ACP Client Core | SS-02 (streaming response) | SS-01 (connection status) |
| F-002 | Chat & Collaboration | SS-01 (main chat), SS-02 (findings) | SS-03 (input toolbar) |
| F-003 | Session & Project Management | SS-01 (sidebar projects) | SS-03 (context selectors) |
| F-004 | Multi-Agent Orchestration | SS-01 (sidebar structure) | SS-02 (environment panel) |
| F-005 | MCP Server Integration | SS-04 (Integrations > MCP servers) | SS-02 (tool results) |
| F-006 | Browser & Visual Feedback | — (new feature, no Codex equivalent) | SS-02 (file preview pattern) |
| F-007 | Memory & Personalization | SS-04 (Personalization section) | SS-03 (user menu) |
| F-008 | Preview Iterations | — (new feature) | SS-02 (verification section) |
| F-009 | Automations | SS-01 (Automations nav item) | SS-04 (settings pattern) |
| F-010 | Terminal Integration | SS-04 (Default terminal location) | SS-02 (command blocks) |
| F-011 | Settings & Configuration | SS-04 (full settings) | SS-03 (user profile menu) |

---

### 9.4a Kimi Branding Application Notes

**App Identity:**
- App name: "Kimi Code Desktop" or "Kimi Agent" (not "Codex" or generic names)
- Window title format: "{session_name} — Kimi Code Desktop"
- About dialog: Kimi K Icon + "Kimi Code Desktop" + version + "Built by Moonshot AI"
- Splash screen: Kimi K Icon centered, blue dot pulsing, "Kimi Code Desktop" below
- Tray icon: K Only monogram, 16px

**No Codex Branding:**
- Remove all references to "Codex", "OpenAI", "GPT" in UI copy and code
- Replace with "Kimi", "Moonshot AI", "K2.5", "K2.6" as appropriate
- Model selector shows "Kimi K2.5", "Kimi K2.6" (not GPT-5.5)
- Usage/billing references Kimi platform credits, not Codex messages

**Kimi-Specific Copy:**
- "Approve for me" → "Approve for me" (same pattern, Kimi branded)
- "You're out of Codex messages" → "Usage limit reached" or "Upgrade for more Kimi tokens"
- "GPT-5.5" → "Kimi K2.5" or "Kimi K2.6"
- "Codex can read and edit files" → "Kimi can read and edit files"

---

### 9.4b Slash Command & Mention Component Specifications

#### Slash Command Component (`/`)

**Trigger:** User types `/` in chat input  
**Dismiss:** Escape, Backspace (remove `/`), click outside, or select command  

**Visual Specification:**
- Container: Floating dropdown, positioned above or below input (prevents overflow)
- Background: `#2a2a2a` with 1px border `#333333`
- Border radius: 12px
- Shadow: `0 8px 24px rgba(0,0,0,0.4)`
- Max-height: 320px with scroll
- Width: min(600px, input width + 40px)
- Padding: 8px 0

**Row Specification:**
- Height: 44px
- Padding: 0 16px
- Layout: flex row, align center, gap 12px
- Icon: 20px, `#888888`, Lucide icon mapped to command
- Name: 14px, weight 500, `#e0e0e0`
- Description: 13px, weight 400, `#888888`
- Hover/Selected: background `#333333`, text `#e0e0e0`
- Selected indicator: left border 2px `#4a9eff` or background shift

**Commands to Implement (mapped from SS-05):**

| Command | Icon | Description | Action |
|---------|------|-------------|--------|
| `chat` | `MessageSquare` | "Don't work in a project" | Switch to general chat mode (no project context) |
| `code-review` | `GitPullRequest` | "Review unstaged changes or compare against a branch" | Trigger code review workflow |
| `fast` | `Zap` | "1.5x speed, increased usage" | Enable fast mode (higher token usage, faster responses) |
| `feedback` | `MessageSquarePlus` | "Send feedback about this chat" | Open feedback dialog |
| `goal` | `Target` | "Set a goal that Codex will keep working towards" | Set persistent task goal |
| `mcp` | `Plug` | "Show MCP server status" | Open MCP status panel |
| `memories` | `Brain` | "Use on, generate on" | Toggle memory generation/usage |
| `model` | `Box` | "GPT-5.5" | Open model selector |
| `new-worktree` | `GitBranch` | "Run this chat in a new worktree" | Create new Git worktree for this session |
| `personality` | `Smile` | "Choose how Codex responds" | Open personality settings |
| `pet` | `Cat` | "Wake or tuck away the desktop pet" | Toggle desktop pet (easter egg) |

**Keyboard Navigation:**
- `↑` / `↓`: Navigate commands
- `Enter`: Select highlighted command
- `Escape`: Close menu
- `Tab`: Select + move focus back to input
- Typing after `/` filters commands by name/description

**Accessibility:**
- `role="listbox"` on container
- `role="option"` on each row
- `aria-selected` on highlighted row
- `aria-label` on command name + description

---

#### Mention Component (`@`)

**Trigger:** User types `@` in chat input  
**Dismiss:** Escape, Backspace (remove `@`), click outside, or select item  

**Visual Specification:**
- Container: Same as slash command (floating dropdown, `#2a2a2a`, 12px radius)
- Width: min(600px, input width + 40px)
- Sections: Divided by headers, no divider lines

**Section Header:**
- Text: 12px, `#666666`, uppercase, letter-spacing 0.5px
- Padding: 8px 16px 4px

**Plugin Row Specification:**
- Height: 48px
- Padding: 0 16px
- Layout: flex row, align center, gap 12px
- Icon: 24px rounded square, colored per app:
  - Browser: `#4a9eff` (blue) with globe icon
  - Computer: `#a78bfa` (purple) with monitor icon  
  - Slack: Multi-color Slack logo or `#e01e5a` with hash icon
- Name: 14px, weight 500, `#e0e0e0`
- Description: 13px, weight 400, `#888888`
- Hover: background `#333333`

**File Row Specification (when search active):**
- Height: 40px
- Padding: 0 16px
- Layout: flex row, align center, gap 10px
- Icon: 16px file icon, `#888888` (varies by file type)
- Filename: 14px, weight 400, `#e0e0e0`
- Path: 12px, weight 400, `#555555`, truncated with ellipsis
- Match highlight: search term in filename bolded with `#4a9eff`

**Empty State (Files section before search):**
- Text: "Type to search for files"
- Style: 13px, `#555555`, centered, italic
- Padding: 16px

**Search Behavior:**
- Typing after `@` triggers fuzzy file search
- Results filter in real-time (<100ms)
- Search scope: current project files (respect .gitignore)
- Max results: 50
- Ranking: exact match > prefix match > fuzzy match > recently opened

**Insertion Behavior:**
- Selecting plugin: inserts `@PluginName` as token in input
- Selecting file: inserts `@path/to/file.ext` as token in input
- Token styling: pill/badge in input, `#4a9eff` background, rounded
- Token is removable (click × or Backspace at token position)

**Keyboard Navigation:**
- `↑` / `↓`: Navigate items (crosses sections)
- `Enter`: Select highlighted item
- `Escape`: Close menu
- `Tab`: Select + move focus back to input
- Typing filters across plugins and files simultaneously

**Accessibility:**
- `role="listbox"` on container
- `role="option"` on each row
- `aria-selected` on highlighted
- `aria-label` describing item type (plugin or file)

---

### 9.4b Updated Feature-to-Screenshot Mapping (with Slash/Mention)

| Feature ID | Feature Name | Primary Screenshot | Secondary Screenshot | UI Components |
|-----------|-------------|-------------------|---------------------|--------------|
| F-001 | ACP Client Core | SS-02 (streaming) | SS-01 (status) | Status bar, connection indicator |
| F-002 | Chat & Collaboration | SS-01, SS-02 | SS-05, SS-06 | Chat thread, **slash menu**, **mention menu**, diff blocks, approvals |
| F-003 | Session & Project Management | SS-01 (sidebar) | SS-03 (selectors) | Project tree, session list, context selectors |
| F-004 | Multi-Agent Orchestration | SS-01 (sidebar) | SS-02 (env panel) | Agent dashboard, worktree indicators |
| F-005 | MCP Server Integration | SS-04 (Integrations) | SS-06 (plugin menu) | MCP settings, **plugin mention rows**, tool results |
| F-006 | Browser & Visual Feedback | — (new) | SS-02 (file preview) | Preview pane, annotation canvas |
| F-007 | Memory & Personalization | SS-04 (Personalization) | SS-05 (`memories` command) | Memory panel, **slash command toggle** |
| F-008 | Preview Iterations | — (new) | SS-02 (verification) | Approach comparator, branch selector |
| F-009 | Automations | SS-01 (Automations nav) | SS-04 (settings) | Automation list, scheduler UI |
| F-010 | Terminal Integration | SS-04 (terminal location) | SS-02 (command blocks) | Terminal pane, command blocks |
| F-011 | Settings & Configuration | SS-04 (full settings) | SS-03 (profile menu) | Settings panels, toggles, dropdowns |

---

### 9.5 Assets to Extract from Screenshots

For implementation, the following visual assets should be recreated (not extracted — recreate as SVG or CSS):

| Asset | Location in Screenshot | Recreation Method |
|-------|------------------------|-------------------|
| Navigation icons | SS-01 sidebar | Lucide Dioxus icons: `Pencil`, `Search`, `Plug`, `Clock` |
| Project folder icon | SS-01 sidebar | Lucide `Folder` |
| File icons | SS-02 file card | Lucide `FileText`, `FileCode` |
| Git branch icon | SS-03 branch selector | Lucide `GitBranch` |
| Shield icon | SS-01 approval | Lucide `Shield` |
| Settings gear | SS-04, SS-03 | Lucide `Settings` |
| Toggle switch | SS-04 | CSS custom toggle (iOS style) |
| Chevron dropdown | SS-01, SS-03, SS-04 | Lucide `ChevronDown` |
| Send button | SS-01 | Lucide `ArrowUp` in circle |
| Microphone | SS-01 | Lucide `Mic` |
| Thumbs up/down | SS-02 | Lucide `ThumbsUp`, `ThumbsDown` |
| Copy icon | SS-02 | Lucide `Copy` |
| Share icon | SS-02 | Lucide `Share` |

---

*End of UI Reference Section*
