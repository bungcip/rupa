# TASK-092: Clean cargo clippy warnings

## Task Header

- **Task ID**: TASK-092
- **Title**: Inherit workspace lints from every member and remove unused dependencies
- **Milestone**: Maintenance
- **Priority**: High
- **Dependencies**: TASK-091
- **Status**: Complete

## Description

`cargo clippy --all-targets --all-features -- -D warnings` was green for Rust
code but Cargo still emitted manifest warnings:

- every member crate was missing `[lints] workspace = true`, so the workspace
  lint table was not inherited;
- `clap`/CI reported unused dependencies (`anyhow` in `rupa-cli`, `thiserror` in
  `rupa-ir`/`rupa-geometry`, `serde` in `rupa-image`);
- workspace dependencies `anyhow` and `pretty_assertions` were unused;
- the lint key `clippy.performance` is not a real clippy group.

## Changes

- Added `[lints]\nworkspace = true` to all eight member manifests.
- Removed the unused per-crate dependencies and workspace entries.
- Replaced `clippy.performance` with `clippy.perf`.
- Normalized manifest trailing newlines.

## Verification

- `cargo clippy --all-targets --all-features -- -D warnings` — exit 0, zero
  warning lines.
- `cargo build --workspace` — OK.
- `cargo test --workspace` — 565 passed, 0 failed.
