# TASK-048: Filter pipeline

**Milestone**: 7 — Advanced Static Graphics
**Dependencies**: TASK-025, TASK-047
**Status**: Complete

## Goal

Implement a filter pipeline for rendered layers.

## Scope

- Parse `effect <filter>` on layer definitions.
- Lower filters into IR layer definitions.
- Renderer applies filters to a layer's rendered content.

## Acceptance Criteria

- [x] Filter syntax parses and validates
- [x] Filters lower to IR
- [x] Renderer applies filters
- [x] `cargo fmt`, `cargo clippy`, `cargo build`, `cargo test` pass
