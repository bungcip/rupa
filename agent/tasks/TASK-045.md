# TASK-045: Pattern fills

**Milestone**: 7 — Advanced Static Graphics
**Dependencies**: TASK-043, TASK-044
**Status**: Complete

## Goal

Implement pattern fills (dots, grid, stripes, checkerboard).

## Scope

- Parse `fill pattern <kind>` with indented properties.
- Lower pattern fills into IR `Paint::Pattern`.
- Render pattern fills with tiled pixmaps.

## Acceptance Criteria

- [x] Pattern fill syntax parses and validates
- [x] Pattern paints lower to IR
- [x] Pattern fills render to PNG
- [x] `cargo fmt`, `cargo clippy`, `cargo build`, `cargo test` pass