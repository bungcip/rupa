# TASK-034: Basic brush stroke

**Milestone**: 5 — Raster Drawing
**Dependencies**: TASK-031 (raster surface IR and allocation)
**Status**: Complete

## Goal

Implement basic brush stroke raster operations on raster surfaces.

## Scope

- Parse `brush size <n>px` with `fill` and a `stroke` point block.
- Add AST and IR representations for brush strokes.
- Lower brush strokes into IR.
- Render brush strokes on raster surfaces.

## Acceptance Criteria

- [x] `brush` parses inside a layer block
- [x] AST and IR carry brush strokes
- [x] Lowering preserves brush strokes
- [x] Renderer applies brush strokes
- [x] `cargo fmt`, `cargo clippy`, `cargo build`, `cargo test` pass