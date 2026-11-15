# TASK-035: Vector/raster composition

**Milestone**: 5 — Raster Drawing
**Dependencies**: TASK-031 (raster surface IR and allocation), TASK-032, TASK-033, TASK-034
**Status**: Complete

## Goal

Ensure raster layers compose correctly with vector layers and other raster
layers: independent pixel buffers, transforms, clipping, opacity, and source
order.

## Scope

- Raster layers allocate an independent pixel buffer.
- Raster layer instances composite their buffer into the parent with the
  instance transform and clip.
- Instance (and enclosing) opacity modulates the whole raster surface,
  including clear, pixel, rectangle, and brush operations.
- Vector and raster children may coexist in the same composition and are
  ordered by z-index.
- Nested raster and vector layers compose recursively.

## Acceptance Criteria

- [x] Raster layers create independent pixel buffers
- [x] Raster layers compose with vector layers correctly
- [x] Source order execution within raster layer
- [x] `cargo fmt`, `cargo clippy`, `cargo build`, `cargo test` pass