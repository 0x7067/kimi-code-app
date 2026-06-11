<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-06-11 | Updated: 2026-06-11 -->

# mipmap-xxhdpi

## Purpose
Android launcher icon assets for the extra-extra-high-density (xxhdpi) screen density bucket (~480 dpi). These PNGs are bundled into the Android app package and referenced by the Android launcher manifest.

## Key Files

| File | Description |
|------|-------------|
| `ic_launcher.png` | Square launcher icon at 144 × 144 px for xxhdpi devices |
| `ic_launcher_round.png` | Circular launcher icon variant at 144 × 144 px |
| `ic_launcher_foreground.png` | Adaptive icon foreground layer at 324 × 324 px |

## Subdirectories

None.

## For AI Agents

### Working In This Directory
This directory contains generated image assets. Do not edit these PNGs directly; update the source icon artwork and regenerate the Android icon set instead. Keep file names and dimensions aligned with Android adaptive/legacy launcher icon conventions.

### Testing Requirements
Verify that `ic_launcher.png` and `ic_launcher_round.png` remain 144 × 144 px and `ic_launcher_foreground.png` remains 324 × 324 px. Confirm the app icon displays correctly on an xxhdpi Android device or emulator after rebuilding the Tauri Android package.

### Common Patterns
- Icons are organized under `src-tauri/icons/android/mipmap-<density>/` matching Android resource qualifiers.
- Foreground assets are larger than legacy icons to support adaptive icon masking and scaling.

## Dependencies

### Internal
- `src-tauri/icons/` — source icon set and other platform densities (see `src-tauri/icons/AGENTS.md`)
- `src-tauri/` — Tauri configuration that references these icons for the Android bundle

### External
- Android SDK build tools — package these assets into the APK/AAB

<!-- MANUAL: -->
