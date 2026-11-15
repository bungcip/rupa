# TASK-030: Resource diagnostics

**Milestone**: 4 — Resources and Text
**Dependencies**: TASK-026 (image decoding), TASK-027 (picture placement), TASK-029 (text rendering)
**Status**: Complete

## Goal

Make missing or undecodable resources produce clear diagnostics, including
at `check` time when the input document's directory is known.

## Scope

- Add stable diagnostic codes for missing and undecodable resources.
- Validate picture resources during `rupa check`, resolving paths relative to
  the input document's directory.
- Validate that requested text fonts can be resolved.
- Preserve existing render-time resource errors.

## Files

- crates/rupa-syntax/src/diagnostic.rs
- crates/rupa-cli/src/commands.rs
- crates/rupa-renderer/src/vector.rs
- agent tasks/docs

## Acceptance Criteria

- [ ] `check` reports missing picture resources with their path
- [ ] `check` reports undecodable picture resources with their path
- [ ] `check` reports unresolvable text fonts
- [ ] Render-time resource errors continue to name the path
- [ ] `cargo fmt`, `cargo clippy`, `cargo build`, `cargo test` pass
