# DECISION-014: Transform property syntax and composition

**Date**: 2026-11-10
**Task**: TASK-017 (Transform matrix composition)

## Context

Specification §11 requires translation/position, rotation, scale, opacity, and
an explicit transform origin with documented composition order. The renderer
already had a 3x2 matrix and center pivots (DECISION-005), but source syntax
for `rotate`, `scale`, and `opacity` was not parsed.

## Decision

- **Properties** available on every primitive (closed shapes, lines,
  polylines, paths):
  - `rotate <angle>deg` — rotation around the primitive's geometric center.
  - `scale <factor>` — uniform scale around the center.
  - `scale <sx> x <sy>` — non-uniform scale around the center.
  - `opacity <value>` — dimensionless value in `0.0..=1.0`.
- **Defaults**: rotation 0rad, scale `(1, 1)`, opacity `1.0`.
- **Composition order**: `rotate * scale` in the documented column-vector
  matrix convention, so scale is applied first and rotation second. The
  renderer then pivots the matrix around the primitive center:
  `translate(center) * (rotate * scale) * translate(-center)`.
- **Position**: `at`/`center at` remains geometry, not part of the transform
  matrix.
- **Units**: rotation must use `deg`; scale and opacity are dimensionless.
  Rotation with other units is invalid.
- **Validation**: scale factors must be finite and strictly positive; opacity
  must be finite and within `0.0..=1.0`. Violations produce `S014`.
  Negative/zero scale is rejected rather than treated as a mirror.
- **Opacity**: applied to both fill and stroke colors before rasterization.
- **Matrix origin**: existing renderer pivots are used (rectangle/ellipse/
  polygon/path bounding-box center, circle/ellipse center, line midpoint).

## Consequences

- Transform semantics are now testable from `.rupa` source through the CLI.
- Non-uniform scale is available even though the specification only shows a
  uniform example.
- A future task may allow mirroring via negative scale if explicitly
  specified; `S014` currently rejects it.
- Transform ordering must not change without updating this decision and
  renderer/golden tests.
