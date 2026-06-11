<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-06-11 | Updated: 2026-06-11 -->

# android

## Purpose
Android launcher icon assets for the Tauri-generated Android app. This directory provides adaptive-icon resources: density-specific PNG foregrounds and launcher icons, plus an XML definition that layers a foreground drawable over a background color for API 26+.

## Key Files

| File | Description |
|------|-------------|
| `mipmap-anydpi-v26/ic_launcher.xml` | Adaptive-icon manifest referencing the foreground PNG and background color |
| `values/ic_launcher_background.xml` | Color resource for the launcher icon background (`#fff`) |
| `mipmap-hdpi/ic_launcher.png` | 72×72 square launcher icon |
| `mipmap-hdpi/ic_launcher_round.png` | 72×72 round launcher icon |
| `mipmap-hdpi/ic_launcher_foreground.png` | 72×72 foreground layer for adaptive icons |
| `mipmap-mdpi/ic_launcher.png` | 48×48 square launcher icon |
| `mipmap-mdpi/ic_launcher_round.png` | 48×48 round launcher icon |
| `mipmap-mdpi/ic_launcher_foreground.png` | 48×48 foreground layer for adaptive icons |
| `mipmap-xhdpi/ic_launcher.png` | 96×96 square launcher icon |
| `mipmap-xhdpi/ic_launcher_round.png` | 96×96 round launcher icon |
| `mipmap-xhdpi/ic_launcher_foreground.png` | 96×96 foreground layer for adaptive icons |
| `mipmap-xxhdpi/ic_launcher.png` | 144×144 square launcher icon |
| `mipmap-xxhdpi/ic_launcher_round.png` | 144×144 round launcher icon |
| `mipmap-xxhdpi/ic_launcher_foreground.png` | 144×144 foreground layer for adaptive icons |
| `mipmap-xxxhdpi/ic_launcher.png` | 192×192 square launcher icon |
| `mipmap-xxxhdpi/ic_launcher_round.png` | 192×192 round launcher icon |
| `mipmap-xxxhdpi/ic_launcher_foreground.png` | 192×192 foreground layer for adaptive icons |

## Subdirectories

| Directory | Purpose |
|-----------|---------|
| `mipmap-anydpi-v26/` | Adaptive-icon XML descriptor |
| `mipmap-hdpi/` | High-density (240 dpi) launcher icons and foreground layer |
| `mipmap-mdpi/` | Medium-density (160 dpi) launcher icons and foreground layer |
| `mipmap-xhdpi/` | Extra-high-density (320 dpi) launcher icons and foreground layer |
| `mipmap-xxhdpi/` | Extra-extra-high-density (480 dpi) launcher icons and foreground layer |
| `mipmap-xxxhdpi/` | Extra-extra-extra-high-density (640 dpi) launcher icons and foreground layer |
| `values/` | Color resource for the adaptive-icon background |

## For AI Agents

### Working In This Directory
- These files are generated/updated by Tauri icon tooling; edit them only when replacing app icons or fixing packaging issues.
- If changing the icon, replace the PNGs in **all** density folders to keep Android's resource picker consistent across devices.
- Keep `ic_launcher.xml` drawable references in sync with the actual `ic_launcher_foreground.png` files.
- Update `values/ic_launcher_background.xml` if the desired background color changes.

### Testing Requirements
- Verify icon rendering with `cargo tauri android dev` or by building an APK/AAB with `cargo tauri android build`.
- Check both square and round variants on devices or emulators running different Android API levels.

### Common Patterns
- Density-specific `mipmap-<density>/` folders following Android resource conventions.
- `mipmap-anydpi-v26/` hosts the vector-style adaptive-icon XML used on API 26+.
- Foreground layers are PNGs; the background is a single shared color resource.

## Dependencies

### Internal
- Referenced from `src-tauri/icons/` (see `src-tauri/icons/AGENTS.md`)
- Copied into the Tauri Android project during Android build/packaging

### External
- Android adaptive-icon specification (API 26+)
- Tauri 2 Android build pipeline

<!-- MANUAL: -->
