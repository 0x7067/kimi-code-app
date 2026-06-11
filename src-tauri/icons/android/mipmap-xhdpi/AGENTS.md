<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-06-11 | Updated: 2026-06-11 -->

# mipmap-xhdpi

## Purpose
This directory holds the extra-high-density (xhdpi, ~320 dpi) Android launcher icon assets for the Tauri app. These PNGs are bundled into the Android package and referenced by the Android manifest to provide correctly-sized launcher icons on xhdpi devices.

## Key Files

| File | Description |
|------|-------------|
| `ic_launcher.png` | Square launcher icon for xhdpi screens |
| `ic_launcher_foreground.png` | Foreground layer used for adaptive launcher icons on xhdpi screens |
| `ic_launcher_round.png` | Circular launcher icon variant for xhdpi screens |

## Subdirectories

| Directory | Purpose |
|-----------|---------|
| *(none)* | |

## For AI Agents

### Working In This Directory
- Only replace PNG files; keep file names exactly as expected by the Android build.
- Use Android xhdpi launcher icon dimensions: 96x96 dp (96x96 px at 1:1 xhdpi scale) for `ic_launcher.png` and `ic_launcher_round.png`; the foreground asset should fit within the safe zone of the adaptive icon grid.
- Preserve transparency where the original PNGs have transparent regions.

### Testing Requirements
- After changing assets, run `cargo tauri build` (or `cargo tauri android build`) and verify the Android package installs and shows the updated icon on an xhdpi device or emulator.
- No automated unit tests apply to image assets.

### Common Patterns
- Icon assets are generated once during project setup and updated only when the app icon or branding changes.
- Adaptive icons combine `ic_launcher_foreground.png` with a background color defined in `mipmap-anydpi-v26/values/` or the Android manifest.

## Dependencies

### Internal
- `src-tauri/icons/` — parent directory containing source icon assets and platform-specific icon sets (see `src-tauri/icons/AGENTS.md`)
- `src-tauri/` — Tauri backend build configuration that packages these icons into the Android app

### External
- Android adaptive/legacy launcher icon conventions
- Tauri 2 icon asset pipeline

<!-- MANUAL: -->
