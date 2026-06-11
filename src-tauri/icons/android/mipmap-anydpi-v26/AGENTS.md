<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-06-11 | Updated: 2026-06-11 -->

# mipmap-anydpi-v26

## Purpose
Contains Android adaptive icon definitions for API 26+ (Android 8.0 Oreo and later). This directory lets Android pick a single density-independent icon resource that references separate foreground and background layers.

## Key Files

| File | Description |
|------|-------------|
| `ic_launcher.xml` | Adaptive icon manifest referencing `@mipmap/ic_launcher_foreground` and `@color/ic_launcher_background` |

## Subdirectories

None.

## For AI Agents

### Working In This Directory
- Do not edit `ic_launcher.xml` unless the app's launcher icon assets or colors have changed.
- Keep the `adaptive-icon` XML valid and ensure the referenced `@mipmap` and `@color` resources exist in the Android project.

### Testing Requirements
- Verify the Android app builds and the launcher icon displays correctly on an API 26+ device or emulator.
- If the foreground/background resources change, confirm the references in `ic_launcher.xml` still resolve.

### Common Patterns
- Tauri generates adaptive icons here automatically from source icon assets during `cargo tauri android build`.

## Dependencies

### Internal
- `src-tauri/icons/android/` — sibling density-specific and color/icon resources referenced by this adaptive icon.
- `src-tauri/icons/` — source icon assets used by Tauri to generate Android launcher icons.

### External
- Android SDK adaptive-icon format (`http://schemas.android.com/apk/res/android`).

<!-- MANUAL: -->
