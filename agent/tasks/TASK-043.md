# TASK-043: Linear gradient paint

**Milestone**: 7 — Advanced Static Graphics
**Dependencies**: TASK-007 (Solid fill paint representation)
**Status**: Complete

## Goal

Implement linear gradient fills for shapes.

## Scope

- Parse `fill gradient` with `from`/`to` shorthand or explicit `stop` list and
  optional `angle`.
- Lower gradient fills into IR `Paint::LinearGradient`.
- Render linear gradients with tiny-skia.

## Acceptance Criteria

- [x] Gradient syntax parses and validates
- [x] Gradient paints lower to IR
- [x] Gradients render to PNG
- [x] `cargo fmt`, `cargo clippy`, `cargo build`, `cargo test` pass