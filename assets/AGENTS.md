<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-06-11 | Updated: 2026-06-11 -->

# assets

## Purpose
Static assets bundled into the Dioxus webview frontend. The `main.css` stylesheet is referenced from `src/main.rs` via `asset!("/assets/main.css")` and loaded at app startup.

## Key Files

| File | Description |
|------|-------------|
| `main.css` | Global application stylesheet — layout, components, modals, diff syntax highlighting |
| `dioxus.png` | Dioxus framework logo/asset |
| `tauri.svg` | Tauri framework logo/asset |

## Subdirectories

*None.*

## For AI Agents

### Working In This Directory
- `main.css` is the primary stylesheet for the entire app. Changes here affect all UI components.
- The CSS is loaded as a Dioxus `Asset` and injected into the document head.
- Keep styles scoped via class names; the app does not use CSS-in-JS or CSS modules.

### Testing Requirements
- Visual testing via `cargo tauri dev` is required after any CSS change.

### Common Patterns
- Class naming follows BEM-like conventions: `.composer-box`, `.session-item`, `.diff-pane`, etc.
- Color tokens and spacing are hand-coded; there is no CSS preprocessor configured.

## Dependencies

### Internal
- `src/main.rs` — loads `main.css` at startup
- `src/app.rs` — all component class names referenced here

### External
*None.*

<!-- MANUAL: -->
