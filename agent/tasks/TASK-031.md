# TASK-031: Raster surface IR and allocation

**Milestone**: 5 — Raster Drawing
**Dependencies**: TASK-025 (layer bounds clipping)
**Status**: Complete

## Goal

Add raster layer mode to the layer IR and allocate an independent pixel buffer
when rendering a raster layer.

## Scope

- Parse `mode raster` inside layer definitions.
- Add a layer mode to the syntax AST and graphics IR.
- Lower the declared mode into IR.
- Allocate an independent raster surface when rendering a raster layer instance.

## Acceptance Criteria

- [ ] `mode raster` parses inside a layer block
- [ ] AST and IR carry the layer mode
- [ ] Lowering preserves raster mode
- [ ] Renderer allocates an independent buffer for raster layers
- [ ] `cargo fmt`, `cargo clippy`, `cargo build`, `cargo test` pass
