<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-06-11 | Updated: 2026-06-11 -->

# icons

## Purpose
Application icon assets in multiple resolutions and formats for cross-platform bundling. Referenced by `src-tauri/tauri.conf.json` under the `bundle.icon` array.

## Key Files

| File | Description |
|------|-------------|
| `icon.png` | Source/master icon |
| `icon.icns` | macOS icon bundle |
| `icon.ico` | Windows icon bundle |
| `32x32.png` | Small raster icon |
| `128x128.png` | Standard raster icon |
| `128x128@2x.png` | Retina/high-DPI raster icon |
| `StoreLogo.png` | Microsoft Store logo |
| `Square*.png` | Various Windows tile/square logo sizes |

## Subdirectories

*None.*

## For AI Agents

### Working In This Directory
- When updating the app icon, regenerate all sizes/formats to maintain consistency across platforms.
- `tauri.conf.json` explicitly lists `icons/32x32.png`, `icons/128x128.png`, `icons/128x128@2x.png`, `icons/icon.icns`, and `icons/icon.ico` for bundling.

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
