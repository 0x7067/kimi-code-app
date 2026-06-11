<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-06-11 | Updated: 2026-06-11 -->

# design_tokens

## Purpose
Design tokens for the Kimi Code Desktop design system. This module centralizes all visual constants used across the UI — colors, typography, spacing, animation durations/easings, and elevation shadows — so components can import these values instead of hard-coding hex codes, pixel values, or Tailwind classes.

## Key Files

| File | Description |
|------|-------------|
| `mod.rs` | Module entry point; re-exports `Colors` and `Typography`, declares all submodules, and includes `tests.rs` under `#[cfg(test)]`. |
| `colors.rs` | Dark-mode color palette: Kimi brand blues, backgrounds, borders, text, semantic status colors, scrollbars, and legacy accent aliases. |
| `typography.rs` | Font stacks, type scale sizes, font weights, line heights, and letter spacing constants. |
| `spacing.rs` | 4 px-grid spacing scale and layout dimension constants (sidebar widths, chat max width, button heights, row heights). |
| `animation.rs` | Duration, easing, and ready-made Tailwind transition class tokens. |
| `elevation.rs` | Shadow values for cards, dropdowns, modals, tooltips, input focus glows, and toasts. |
| `tests.rs` | Spec-conformance unit tests for colors, spacing, typography, animation durations, and layout dimensions. |

## Subdirectories

No subdirectories.

## For AI Agents

### Working In This Directory
- This is a data-only module. Add new tokens as `pub const` items on the relevant `struct` (e.g., `Colors`, `Spacing`).
- Keep values aligned with `DESIGN_SYSTEM.md` in the project root; if a token changes, update this file first.
- Prefer extending existing token groups over creating new files. Only add a new submodule if the design system introduces a wholly new category.
- Constants are exposed as `&'static str` for style values and `u32` for pixel dimensions so they work in both inline Tailwind-style classes and layout calculations.

### Testing Requirements
- Run Rust unit tests for this module with `cargo test design_tokens`.
- When adding or modifying tokens, add corresponding assertions in `tests.rs` to ensure values match the spec.

### Common Patterns
- Each token category is implemented as a zero-sized `pub struct` with an associated `impl` block of `pub const` values.
- `#[allow(dead_code)]` is used because many constants are referenced only from Dioxus component code, which the Rust compiler may not see from this crate.

## Dependencies

### Internal
- `DESIGN_SYSTEM.md` (project root) — the authoritative source of truth for token values.
- Referenced by UI components in `src/components/`.

### External
- None beyond the Rust standard library.

<!-- MANUAL: -->
