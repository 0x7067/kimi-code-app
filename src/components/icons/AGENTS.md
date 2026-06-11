<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-06-11 | Updated: 2026-06-11 -->

# icons

## Purpose
Inline SVG icon system for the Dioxus frontend. Every icon is a hand-written, Lucide-style `#[component]` that accepts `size`, `color`, and `stroke_width` props with sensible defaults, ensuring a consistent visual language across the app without external icon fonts or image assets.

## Key Files

| File | Description |
|------|-------------|
| `mod.rs` | Module entry point; re-exports all icon components from `lucide.rs`. |
| `lucide.rs` | Inline SVG icon components (search, navigation, status, chat, file/git, layout, and misc icons). |

## Subdirectories

None.

## For AI Agents

### Working In This Directory
- Add new icons to `lucide.rs` and re-export them from `mod.rs` if they need to be public.
- Keep all icons on the same 24×24 artboard with `view_box: "0 0 24 24"`.
- Use the existing `IconProps` pattern (`size`, `color`, `stroke_width`) so callers can override defaults.
- Prefer `stroke-linecap: "round"` and `stroke-linejoin: "round"` for consistency with the Lucide style.
- Use design-token colors from `crate::design_tokens::Colors` for default props rather than hard-coding hex values.

### Testing Requirements
- Run `cargo check` to verify the component definitions and re-exports compile.
- Verify new icons render correctly by running `cargo tauri dev` and inspecting them in context.

### Common Patterns
- `IconProps` is the shared prop struct with `#[props(default = ...)]` defaults.
- Icons are used as `rsx! { IconSearch { size: 20, color: "#A3A3A3" } }`.
- `IconLoader` adds the `kimi-spinner` CSS class for animated loading states.

## Dependencies

### Internal
- `crate::design_tokens::Colors` — default icon color values.

### External
- `dioxus::prelude::*` — component and `rsx!` macros.

<!-- MANUAL: -->
