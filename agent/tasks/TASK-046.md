# TASK-046: Clipping and mask system

**Milestone**: 7 — Advanced Static Graphics
**Dependencies**: TASK-025 (Layer bounds clipping)
**Status**: Complete

## Goal

Implement configurable clipping for layer content.

## Scope

- Parse `clip true` / `clip false` on layer definitions.
- Lower the clip flag into IR layer definitions.
- Renderer respects the flag, skipping layer bounds clipping when false.

## Acceptance Criteria

- [x] Layer clip property parses and validates
- [x] Clip flag lowers to IR
- [x] Renderer clips or skips clipping based on flag
- [x] `cargo fmt`, `cargo clippy`, `cargo build`, `cargo test` pass