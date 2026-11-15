# TASK-047: Blend mode implementations

**Milestone**: 7 — Advanced Static Graphics
**Dependencies**: TASK-025
**Status**: Complete

## Goal

Implement blend modes for layer instances.

## Scope

- Parse `blend <mode>` on layer instances.
- Lower blend modes into IR layer instances.
- Renderer composites instances using the selected blend mode.

## Acceptance Criteria

- [x] Blend mode syntax parses and validates
- [x] Blend modes lower to IR
- [x] Renderer applies blend modes
- [x] `cargo fmt`, `cargo clippy`, `cargo build`, `cargo test` pass