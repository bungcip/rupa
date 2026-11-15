# DECISION-010: Polygon geometry, validation, and fill rule

**Date**: 2026-11-10
**Task**: TASK-013 (Polygon with points list)

## Context

Specification §7.2 defines the `polygon` points block and requires at least
three points for a filled polygon. It states that polygon points are local
coordinates but does not define closure, fill rule, or transform origin.

## Decision

- **Syntax**: `polygon` contains an optional nested `points` block and an
  optional `fill`. The `points` block is indented one level under `points`;
  each entry is a parenthesized `(x, y)` position. `points` and `fill` may
  appear in either order.
- **Closure**: polygons are always closed. The renderer connects the last
  vertex to the first; no explicit `close` property is required.
- **Validation**:
  - missing `points` block → `S008`
  - fewer than three points → `S009`
  - missing `fill` → `S006`
  - non-pixel coordinates → `S007`
  - negative coordinates are valid; only dimensions/radii must be positive
  - self-intersection is not rejected in this task
- **Coordinates**: points are in the current coordinate system, which at this
  milestone is the document coordinate system. Layer-local coordinates arrive
  with TASK-021. The IR stores absolute `Point { x, y }` values in pixels.
- **Fill rule**: the renderer uses `tiny-skia`'s nonzero winding rule. No
  alternate fill rule is exposed yet.
- **Transform origin**: the center of the polygon's bounding box, consistent
  with the primitive-origin convention in DECISION-005.
- **Fill**: required for this milestone; `fill none` is allowed and lowers to
  no paint. Stroke support is TASK-016.

## Consequences

- Polygons with fewer than three points fail validation rather than rendering
  an invisible or degenerate shape.
- Self-intersecting polygons render according to nonzero winding; a future
  `fill-rule` feature must be explicit.
- Golden tests should use simple non-self-intersecting polygons for clarity.
