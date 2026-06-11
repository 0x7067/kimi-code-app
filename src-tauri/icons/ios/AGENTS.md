<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-06-11 | Updated: 2026-06-11 -->

# ios

## Purpose
iOS app icon assets in the sizes and scales required by Xcode and the App Store. These PNGs are referenced by the iOS bundle target and provide launcher, spotlight, settings, and App Store icons for iPhones and iPads.

## Key Files

| File | Description |
|------|-------------|
| `AppIcon-20x20@1x.png` | 20 pt icon at 1x (20 px) |
| `AppIcon-20x20@2x.png` | 20 pt icon at 2x (40 px) |
| `AppIcon-20x20@2x-1.png` | Alternate 20 pt 2x variant (40 px) |
| `AppIcon-20x20@3x.png` | 20 pt icon at 3x (60 px) |
| `AppIcon-29x29@1x.png` | 29 pt icon at 1x (29 px) |
| `AppIcon-29x29@2x.png` | 29 pt icon at 2x (58 px) |
| `AppIcon-29x29@2x-1.png` | Alternate 29 pt 2x variant (58 px) |
| `AppIcon-29x29@3x.png` | 29 pt icon at 3x (87 px) |
| `AppIcon-40x40@1x.png` | 40 pt icon at 1x (40 px) |
| `AppIcon-40x40@2x.png` | 40 pt icon at 2x (80 px) |
| `AppIcon-40x40@2x-1.png` | Alternate 40 pt 2x variant (80 px) |
| `AppIcon-40x40@3x.png` | 40 pt icon at 3x (120 px) |
| `AppIcon-60x60@2x.png` | 60 pt icon at 2x (120 px) |
| `AppIcon-60x60@3x.png` | 60 pt icon at 3x (180 px) |
| `AppIcon-76x76@1x.png` | 76 pt icon at 1x (76 px) |
| `AppIcon-76x76@2x.png` | 76 pt icon at 2x (152 px) |
| `AppIcon-83.5x83.5@2x.png` | 83.5 pt icon at 2x (167 px) |
| `AppIcon-512@2x.png` | App Store icon at 1024 px |

## Subdirectories

| Directory | Purpose |
|-----------|---------|
| *None* | |

## For AI Agents

### Working In This Directory
- Treat these PNGs as generated assets; do not edit them by hand.
- When the app icon changes, regenerate the entire `src-tauri/icons/` tree from a single source image (e.g., `tauri icon`) so the iOS set stays consistent with desktop and Android assets.
- Keep the naming pattern `AppIcon-{size}@{scale}.png` so the iOS bundle target can locate each icon.

### Testing Requirements
- Verify the icon renders correctly on an iOS device or simulator after `cargo tauri ios dev` / `cargo tauri ios build`.
- Confirm all expected sizes are present; missing icons will cause Xcode warnings or App Store validation failures.

### Common Patterns
- `AppIcon-...-1.png` duplicates are used for iPad vs. iPhone idioms in the same asset catalog.
- The `AppIcon-512@2x.png` file is the 1024 x 1024 px App Store listing icon.

## Dependencies

### Internal
- `src-tauri/icons/icon.png` — source/master raster used to generate these iOS icons
- `src-tauri/icons/AGENTS.md` — parent icon assets directory
- `src-tauri/tauri.conf.json` — references the icon set for iOS bundling

### External
*None.*

<!-- MANUAL: -->
