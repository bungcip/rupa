# TASK-044: Radial gradient paint

**Milestone**: 7 — Advanced Static Graphics
**Dependencies**: TASK-043 (Linear gradient paint)
**Status**: Complete

## Goal

Implement radial gradient fills for shapes.

## Scope

- Parse `fill radial` / `fill radial gradient` / `fill gradient radial`.
- Lower radial gradients into IR `Paint::RadialGradient`.
- Render radial gradients with tiny-skia.

## Acceptance Criteria

- [x] Radial gradient syntax parses and validates
- [x] Radial gradient paints lower to IR
- [x] Radial gradients render to PNG
- [x] `cargo fmt`, `cargo clippy`, `cargo build`, `cargo test` pass