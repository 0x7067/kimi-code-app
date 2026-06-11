<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-06-11 | Updated: 2026-06-11 -->

# capabilities

## Purpose
Tauri 2 capability definitions. Capabilities declare which Tauri APIs and plugin features the main window is allowed to use. This is the security boundary for the webview frontend.

## Key Files

| File | Description |
|------|-------------|
| `default.json` | Main window capability — grants `core:default`, `core:event:default`, `opener:default`, and `dialog:default` permissions |

## Subdirectories

*None.*

## For AI Agents

### Working In This Directory
- Only modify `default.json` to add or remove Tauri permissions.
- The `$schema` field points to the generated desktop schema for IDE validation.
- The `windows` array lists which window labels this capability applies to (`main`).

### Testing Requirements
- Missing permissions will cause runtime errors in the frontend when invoking Tauri APIs. Test via `cargo tauri dev`.

### Common Patterns
- Permissions follow the pattern `plugin:feature` or `core:feature`.
- If adding new Tauri plugin usage (e.g., shell, fs), the corresponding permission must be declared here.

## Dependencies

### Internal
- `src-tauri/src/lib.rs` — registers plugins that require these capabilities
- `src-tauri/tauri.conf.json` — references capabilities in the app security config

### External
*None.*

<!-- MANUAL: -->
