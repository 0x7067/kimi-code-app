<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-06-11 | Updated: 2026-06-11 -->

# mipmap-hdpi

## Purpose
Android high-density (hdpi, ~240 dpi) launcher icon assets used by Tauri when bundling the Android app. These PNGs are consumed by the Android build system as `mipmap-hdpi` resources.

## Key Files

| File | Description |
|------|-------------|
| `ic_launcher.png` | Standard launcher icon at 72x72 dp (108x108 px) |
| `ic_launcher_round.png` | Circular launcher icon at 72x72 dp (108x108 px) |
| `ic_launcher_foreground.png` | Adaptive icon foreground layer used for API 26+ |

## Subdirectories

*None.*

## For AI Agents

### Working In This Directory
- Do not edit these files manually; regenerate Android icon variants from a single source image to keep densities consistent.
- Keep all three PNG variants in sync with the other `mipmap-*` density buckets.

### Testing Requirements
- Verify the Android app icon renders correctly on an hdpi device or emulator after `cargo tauri android build`.

### Common Patterns
- Use `tauri icon` (from `tauri-cli`) to regenerate the full `icons/android/` tree from the master source.

## Dependencies

### Internal
- `src-tauri/icons/android/mipmap-mdpi/`, `mipmap-xhdpi/`, `mipmap-xxhdpi/`, `mipmap-xxxhdpi/` — sibling density buckets for other screen densities
- `src-tauri/icons/android/mipmap-anydpi-v26/` — adaptive-icon resources that reference `ic_launcher_foreground.png`

### External
*None.*

<!-- MANUAL: -->
