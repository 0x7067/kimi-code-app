<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-06-11 | Updated: 2026-06-11 -->

# base

## Purpose
Primitive, reusable UI components for the Kimi Code Desktop design system. These are the lowest-level building blocks — buttons, inputs, badges, cards, loading states, etc. — consumed by higher-level layout and feature components in `src/components/`.

## Key Files

| File | Description |
|------|-------------|
| `mod.rs` | Barrel export — declares modules and re-exports the public component API |
| `kimi_badge.rs` | Status/label badges with color variants (blue, green, yellow, red, gray) |
| `kimi_button.rs` | Button component with primary/secondary/ghost/danger variants and sizes |
| `kimi_card.rs` | Elevated surface container with hover, active, padding, and radius options |
| `kimi_dropdown.rs` | Dropdown menu with trigger, items, dividers, and close animation |
| `kimi_empty_state.rs` | Centered empty-state placeholder with icon, title, description, and action |
| `kimi_icon.rs` | Brand logo SVG with blue dot and rounded/round/k-only variants |
| `kimi_input.rs` | Text input and textarea with optional leading/trailing icons |
| `kimi_loading.rs` | Loading indicators: spinner, pulsing dots, and animated skeleton |
| `kimi_toast.rs` | Fixed-position notification toast with auto-dismiss and close button |
| `kimi_toggle.rs` | iOS-style toggle switch with checked/disabled/onchange support |
| `kimi_tooltip.rs` | Hover tooltip with top/bottom/left/right positioning |

## Subdirectories

*None.*

## For AI Agents

### Working In This Directory
- Each component is a Dioxus `#[component]` function returning `Element`.
- Components should depend only on `crate::design_tokens` for colors/typography and Dioxus itself.
- Prefer `class: "..."` for styling; use inline styles only for dynamic values (radius, padding, color tokens interpolated via `Colors` constants).
- Props should have sensible defaults via `#[props(default = ...)]`.
- Use `use_signal` for local ephemeral state and `gloo_timers` for timeouts/animations.

### Testing Requirements
- No unit tests for UI components. Validate visually with `cargo tauri dev`.

### Common Patterns
- `#[allow(dead_code)]` is used on `Colors`/`Typography` constants because they are referenced only inside RSX string interpolation.
- `std::mem::forget(handle)` is used with `gloo_timers::callback::Timeout` to keep the timer alive for animations.
- Components accept `children: Element` and optional `EventHandler` callbacks.

## Dependencies

### Internal
- `src/design_tokens/` — `Colors` and `Typography` tokens used by almost every component

### External
- `dioxus` — UI framework (RSX, signals, effects)
- `gloo-timers` — Browser timers for dropdown close animation and toast auto-dismiss

<!-- MANUAL: -->
