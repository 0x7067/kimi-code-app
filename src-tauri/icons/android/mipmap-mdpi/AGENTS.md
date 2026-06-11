<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-06-11 | Updated: 2026-06-11 -->

# mipmap-mdpi

## Purpose
Contains Android launcher icon assets at **medium density (mdpi, ~160 dpi)** for the Tauri app. These PNGs are bundled as Android `mipmap-mdpi` resources and used by the Android system to display the app icon on home screens and launchers.

## Key Files

| File | Description |
|------|-------------|
| `ic_launcher.png` | Square launcher icon at mdpi resolution (48x48 dp) |
| `ic_launcher_round.png` | Circular launcher icon variant at mdpi resolution |
| `ic_launcher_foreground.png` | Foreground layer for Android adaptive icons at mdpi |

## Subdirectories

None.

## For AI Agents

### Working In This Directory
- Do not edit these PNGs by hand. They are generated assets derived from the source app icon.
- To change the launcher icon, regenerate the full icon set from the source image using Tauri's icon tooling (e.g., `cargo tauri icon`) and replace the entire `src-tauri/icons/` directory contents.
- Keep filenames and the directory structure exactly as Android expects; the Tauri build relies on standard `mipmap-*` resource paths.

### Testing Requirements
- Verify the app icon renders correctly on an Android device or emulator after changing assets.
- Check both square and round launcher icon shapes, as well as adaptive icon behavior, if the target Android version supports it.

### Common Patterns
- PNG dimensions follow Android's mdpi baseline of 48x48 dp.
- `ic_launcher_foreground.png` pairs with a background color defined under `mipmap-anydpi-v26`/`values/` for adaptive icons.

## Dependencies

### Internal
- Generated from the source icon set in `src-tauri/icons/`.
- Referenced by the Tauri Android build configuration and packaged as Android resources.

### External
- Android **mipmap** resource system and launcher icon conventions.
- Tauri 2 icon generation tooling.

<!-- MANUAL: -->
