<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-06-11 | Updated: 2026-06-11 -->

# assets/css

## Purpose
Modular stylesheets for the Dioxus webview frontend. Each file covers a single feature domain, loaded in numbered order from `src/main.rs` via `asset!("/assets/css/XX-*.css")` so tokens and base styles are available before component-specific rules.

## Key Files

| File | Description |
|------|-------------|
| `01-tokens.css` | Design tokens — background layers, borders, text colors, accent palette, shape, shadows, typography, motion easing |
| `02-base.css` | Reset, base HTML/body styles, custom scrollbar, selection color, utility classes |
| `03-layout.css` | App shell, buttons, inputs, topbar, workspace grid, status dot |
| `04-sidebar.css` | Sidebar, session list, project tree, navigation, settings category sidebar, iOS-style toggles |
| `05-thread.css` | Message thread, user/agent bubbles, markdown rendering, tool calls, plan panel, findings, verification, file cards |
| `06-composer.css` | Message composer, slash/mention menus, attachments, pending queue, context selectors, send button |
| `07-panels.css` | Diff pane, terminal panel, checkpoint panel, memory pane, browser pane, multi-agent pane, automation pane, env panel |
| `08-modals.css` | Overlay backdrop, modal dialogs, toast notifications, permission detail blocks |
| `09-settings.css` | Settings layout, preferences form, MCP server cards, mode cards, iOS-style toggles |
| `10-components.css` | Reusable design-system components — `.kimi-btn`, `.kimi-input`, `.kimi-card`, `.kimi-dropdown-item` |
| `11-animations.css` | Shared keyframes and animation utility classes |
| `12-responsive.css` | Breakpoints, reduced-motion support, status bar, copy-to-clipboard buttons |

## Subdirectories

| Directory | Purpose |
|-----------|---------|
| *(none)* | — |

## For AI Agents

### Working In This Directory
- Add new styles to the relevant numbered `XX-*.css` file; keep the modular split intact.
- `src/main.rs` loads all stylesheets as separate `Stylesheet` assets. Order matters — tokens and base must come first, responsive last.
- Styles are plain CSS with no preprocessor or CSS modules; use class-based scoping.

### Testing Requirements
- Visual regression testing via `cargo tauri dev` is required after any CSS change.
- Verify both light/dark-constrained UI and responsive breakpoints at `< 1280px` and `< 1440px`.

### Common Patterns
- Class naming follows BEM-like conventions: `.composer-box`, `.session-item`, `.diff-pane`.
- Variables are hand-coded custom properties from `01-tokens.css`; no build-time token pipeline.
- Animation timing uses the shared cubic-bezier from `--transition-*` tokens.

## Dependencies

### Internal
- `src/main.rs` — loads each `css/*.css` file as a Dioxus `Stylesheet`
- `src/components/` — Rust/Dioxus components apply the class names defined here
- `DESIGN_SYSTEM.md` — source of truth for tokens, colors, and layout specs

### External
*None.*

<!-- MANUAL: -->
