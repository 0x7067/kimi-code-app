<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-06-11 | Updated: 2026-06-11 -->

# values

## Purpose
Android resource values for the Tauri app icon. This directory contains XML resource definitions used by the Android adaptive launcher icon system, specifically the background color for the generated `ic_launcher` icon.

## Key Files

| File | Description |
|------|-------------|
| `ic_launcher_background.xml` | Android `<color>` resource defining the adaptive launcher icon background (`#fff`) |

## Subdirectories

| Directory | Purpose |
|-----------|---------|
| *(none)* | |

## For AI Agents

### Working In This Directory
- This directory holds generated Android asset resources. Manual edits are usually overwritten when Tauri icon assets are regenerated via `cargo tauri icon`.
- If changing the launcher icon background color, prefer updating the source icon and regenerating assets, or update this file and verify it is preserved across rebuilds.

### Testing Requirements
- Changes take effect after rebuilding/reinstalling the Android app (`cargo tauri android dev` / `cargo tauri android build`).
- No automated test suite covers this file; validate visually on an Android device or emulator.

### Common Patterns
- Files here follow the Android Resources directory naming convention (`res/values/`).
- Color resources use the `<resources><color name="...">#rrggbb</color></resources>` format.

## Dependencies

### Internal
- `src-tauri/icons/android/` — parent directory containing Android platform-specific icon assets and other density/resource folders
- `src-tauri/icons/` — source icons used by Tauri to generate Android launcher icons

### External
- Android Gradle build system / Android Asset Studio conventions
- Tauri 2 icon generation (`cargo tauri icon`)

<!-- MANUAL: -->
