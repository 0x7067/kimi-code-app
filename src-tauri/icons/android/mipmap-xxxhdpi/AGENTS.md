<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-06-11 | Updated: 2026-06-11 -->

# mipmap-xxxhdpi

## Purpose
Android launcher icon assets for extra-extra-extra-high-density (xxxhdpi) screens. These PNGs are bundled by Tauri when generating the Android application package.

## Key Files

| File | Description |
|------|-------------|
| `ic_launcher.png` | Square launcher icon at xxxhdpi density |
| `ic_launcher_foreground.png` | Adaptive icon foreground layer at xxxhdpi density |
| `ic_launcher_round.png` | Circular launcher icon variant at xxxhdpi density |

## Subdirectories

None.

## For AI Agents

### Working In This Directory
- This directory only contains generated Android icon assets. Do not edit these files by hand; regenerate icons from the source asset using the Tauri icon tooling.
- Keep all three icon variants present and correctly sized for xxxhdpi (launcher icons are typically 192×192 dp at 640 dpi).

### Testing Requirements
- Verify the Android build still packages the icons by running `cargo tauri android build` or inspecting the generated Android project under `src-tauri/gen/android`.
- No automated unit tests exist for static icon assets.

### Common Patterns
- Icons are generated from a single source image via `cargo tauri icon <source>` and should not be modified individually.
- New density buckets are added by Tauri's icon generator, not created manually.

## Dependencies

### Internal
- `src-tauri/icons/` — sibling density buckets and source icon assets (see `src-tauri/icons/AGENTS.md`)
- `src-tauri/gen/android/` — generated Android project that consumes these assets

### External
- **Tauri 2 icon tooling** — generates Android mipmap densities from a source image
- **Android adaptive icons** — uses `ic_launcher_foreground.png` for adaptive icon composition

<!-- MANUAL: -->
