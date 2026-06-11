<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-06-11 | Updated: 2026-06-11 -->

# scripts

## Purpose
Development automation scripts. Currently holds the pre-commit / CI quality gate that runs Rust formatting and linting checks across the workspace.

## Key Files

| File | Description |
|------|-------------|
| `check.sh` | Quality gate — runs `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets -- -D warnings` |

## Subdirectories

*None.*

## For AI Agents

### Working In This Directory
- Scripts are standard POSIX shell scripts. Keep them `#!/bin/sh` compatible when possible.
- `check.sh` is referenced by the pre-commit hook and CI pipelines.

### Testing Requirements
- Run `./scripts/check.sh` locally before committing to catch formatting or lint errors early.

### Common Patterns
- `cd "$(dirname "$0")/.."` ensures the script runs from the project root regardless of invocation path.
- `set -e` aborts on first failure so fmt errors prevent clippy from running on dirty code.

## Dependencies

### Internal
- Invoked against the workspace root, so it implicitly exercises the root `Cargo.toml` and all workspace members (`src-tauri/`).

### External
- `cargo` (Rust toolchain) — `cargo fmt` and `cargo clippy`.

<!-- MANUAL: -->
