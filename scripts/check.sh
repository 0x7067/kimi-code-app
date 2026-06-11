#!/bin/sh
# Full quality gate: formatting + lints. Used by the pre-commit hook and CI.
set -e
cd "$(dirname "$0")/.."
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
