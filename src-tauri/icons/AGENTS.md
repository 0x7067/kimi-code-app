<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-06-11 | Updated: 2026-06-11 -->

# icons

## Purpose
Application icon assets in multiple resolutions and formats for cross-platform bundling. These icons are referenced by `src-tauri/tauri.conf.json` under the `bundle.icon` array and supply the app icon on macOS, Windows, Linux, iOS, and Android.

## Key Files

| File | Description |
|------|-------------|
| `icon.png` | Source/master raster icon |
| `icon.icns` | macOS icon bundle |
| `icon.ico` | Windows icon bundle |
| `32x32.png` | Small raster icon used for bundling |
| `64x64.png` | Medium raster icon |
| `128x128.png` | Standard raster icon used for bundling |
| `128x128@2x.png` | Retina/high-DPI raster icon used for bundling |
| `StoreLogo.png` | Microsoft Store logo |
| `Square30x30Logo.png` | Windows small tile logo |
| `Square44x44Logo.png` | Windows tile logo |
| `Square71x71Logo.png` | Windows tile logo |
| `Square89x89Logo.png` | Windows tile logo |
| `Square107x107Logo.png` | Windows tile logo |
| `Square142x142Logo.png` | Windows tile logo |
| `Square150x150Logo.png` | Windows medium tile logo |
| `Square284x284Logo.png` | Windows large tile logo |
| `Square310x310Logo.png` | Windows wide/large tile logo |

## Subdirectories

| Directory | Purpose |
|-----------|---------|
| `android/` | Android adaptive and legacy launcher icon densities (see `android/AGENTS.md`) |
| `ios/` | iOS app icon sizes and the App Store icon (see `ios/AGENTS.md`) |

## For AI Agents

### Working In This Directory
- When updating the app icon, regenerate all sizes/formats to maintain consistency across platforms.
- `src-tauri/tauri.conf.json` explicitly lists `icons/32x32.png`, `icons/128x128.png`, `icons/128x128@2x.png`, `icons/icon.icns`, and `icons/icon.ico` for bundling.
- Update `android/` and `ios/` variants together with the desktop assets.

### Testing Requirements
- Verify icons render correctly in the built app bundle (`cargo tauri build`) on each target platform.

### Common Patterns
- Use a tool like `tauri icon` (from `tauri-cli`) to regenerate all variants from a single source image.

## Dependencies

### Internal
- `src-tauri/tauri.conf.json` — references these icons for app bundling

### External
*None.*

<!-- MANUAL: -->
