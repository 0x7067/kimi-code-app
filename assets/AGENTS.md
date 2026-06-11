<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-06-11 | Updated: 2026-06-11 -->

# assets

## Purpose
Static assets bundled into the Dioxus webview frontend. Stylesheets live in `assets/css/` and are loaded individually from `src/main.rs` via `asset!("/assets/css/…")`.

## Key Files

| File | Description |
|------|-------------|
| `main.css` | Stub file documenting the modular CSS structure |
| `css/01-tokens.css` | Design tokens (colors, typography, spacing, motion) |
| `css/02-base.css` | Reset, base styles, scrollbar, utilities |
| `css/03-layout.css` | Shell, buttons, inputs, main, topbar, workspace |
| `css/04-sidebar.css` | Sidebar, navigation, project tree, footer |
| `css/05-thread.css` | Thread, messages, bubbles, markdown, agent headers |
| `css/06-composer.css` | Composer, toolbar, context selectors, slash menu |
| `css/07-panels.css` | Diff, checkpoint, memory, browser, terminal, env |
| `css/08-modals.css` | Overlays, modals, toast, permission modals |
| `css/09-settings.css` | Settings layout, prefs, MCP, toggles, mode cards |
| `css/10-components.css` | Design system: kimi-btn, kimi-input, kimi-card |
| `css/11-animations.css` | Keyframes |
| `css/12-responsive.css` | Breakpoints, reduced motion, status bar, copy/search |
| `dioxus.png` | Dioxus framework logo/asset |
| `tauri.svg` | Tauri framework logo/asset |

## Subdirectories

| Directory | Description |
|-----------|-------------|
| `css/` | Modular stylesheets — one file per feature domain |

## For AI Agents

### Working In This Directory
- Add new styles to the relevant `css/XX-*.css` file rather than creating new files unless the feature is genuinely new.
- `src/main.rs` loads all 12 stylesheets as separate `Stylesheet` components — order matters (tokens → base → layout → components → responsive).
- Keep styles scoped via class names; the app does not use CSS-in-JS or CSS modules.

### Testing Requirements
- Visual testing via `cargo tauri dev` is required after any CSS change.

### Common Patterns
- Class naming follows BEM-like conventions: `.composer-box`, `.session-item`, `.diff-pane`, etc.
- Color tokens and spacing are hand-coded; there is no CSS preprocessor configured.

## Dependencies

### Internal
- `src/main.rs` — loads all `css/*.css` files at startup
- `src/app.rs` — all component class names referenced here

### External
*None.*

<!-- MANUAL: -->
