# TASK-032: Clear and pixel operations

**Milestone**: 5 — Raster Drawing
**Dependencies**: TASK-031 (raster surface IR and allocation)
**Status**: Complete

## Goal

Implement `clear` and `pixel` raster operations inside raster layers.

## Scope

- Parse `clear <color>` and `pixel at (x, y) ... fill <color>` in layer blocks.
- Add AST and IR representations for clear and pixel operations.
- Lower raster operations into IR.
- Render clear and pixel operations in source order on raster surfaces.

## Acceptance Criteria

- [ ] `clear` parses inside a layer block
- [ ] `pixel` parses inside a layer block
- [ ] AST and IR carry clear/pixel operations
- [ ] Lowering preserves clear/pixel operations
- [ ] Renderer applies clear and pixel operations
- [ ] `cargo fmt`, `cargo clippy`, `cargo build`, `cargo test` pass