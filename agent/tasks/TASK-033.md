# TASK-033: Raster rectangle fill

**Milestone**: 5 — Raster Drawing
**Dependencies**: TASK-031 (raster surface IR and allocation)
**Status**: Complete

## Goal

Support filled rectangle operations on raster surfaces.

## Scope

- Reuse the existing rectangle primitive (`rectangle at (x, y)` with `size` and `fill`)
  inside raster layers.
- Render rectangle fills in source order on raster surfaces.

## Acceptance Criteria

- [x] Rectangle fill parses inside a raster layer
- [x] Rectangle fill renders on raster surfaces
- [x] `cargo fmt`, `cargo clippy`, `cargo build`, `cargo test` pass