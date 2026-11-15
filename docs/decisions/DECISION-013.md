# DECISION-013: Shape appearance and fill/stroke separation

**Date**: 2026-11-10
**Task**: TASK-016 (Fill and stroke separation)

## Context

Specification §7.5 requires geometry and appearance to be separate and allows
any shape to have fill only, stroke only, both, or neither. Milestone 1
required fill on closed shapes and only lines/polylines/paths parsed strokes.
That was a deliberate M1 simplification and is now superseded.

## Decision

- **Closed shape appearance**: `rectangle`, `circle`, `ellipse`, `square`,
  `triangle`, and `polygon` each have optional `fill`, `stroke`, and `width`.
  `line`/`polyline` remain stroke-only; `path` already supports both.
- **Optional fill**: missing fill is valid. The stable code `S006` is retained
  in the registry for compatibility but is no longer emitted.
- **Combinations**: fill-only, stroke-only, both, and neither are all valid.
  Neither produces an invisible shape.
- **Explicit none**: `fill none` and `stroke none` disable the respective
  appearance component. Missing declarations mean the same absence for the
  renderer, but `stroke` remains required for line/polyline.
- **Width**: optional, default `1px` when a stroke exists. Explicit widths must
  be positive pixels (`S011`), non-pixel widths are `S007`. Width is validated
  even when no stroke is declared, but it has no visible effect.
- **IR separation**: `ShapeNode` keeps `geometry` separate from
  `fill: Option<PaintId>` and `stroke: Option<Stroke { paint, style }>`.
  Geometry contains no color or paint data. No structural refactor was needed;
  this task completes the behavior using the existing separation.
- **Rendering order**: fill first, then stroke, both subject to shape opacity
  and the shape transform. This is deterministic and matches common vector
  rendering expectations.
- **Stroke color validation**: closed-shape stroke paint uses the same
  `T001`/`T002` paint diagnostics as fill/background.

## Consequences

- Previous tests asserting `S006` for missing fill were replaced with
  fill-optional tests.
- `S006` remains a reserved stable code and must not be reused for another
  meaning without a new decision.
- Cap/join/dash configuration remains TASK-018.
- Future appearance features (gradients, patterns, blend modes) extend paint
  references without changing geometry.
