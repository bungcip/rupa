# DECISION-009: Ellipse, square, and triangle geometry conventions

**Date**: 2026-11-10
**Task**: TASK-012 (Ellipse, square, triangle primitives)

## Context

Specification §7.1 lists `ellipse`, `square`, and `triangle` and gives examples,
but only partially specifies geometry semantics. It explicitly states:

- rectangle and square positions refer to the top-left;
- circle and ellipse positions refer to the center;
- triangle position refers to its bounding-box top-left;
- negative dimensions are invalid.

It does not define the triangle's exact vertex placement inside its bounding
box, whether `square size` takes one or two dimensions, or whether ellipse
`size` is a full size or a radius pair.

## Decision

- **Ellipse**: `center at (cx, cy)` plus `size <width> x <height>` where width
  and height are the full axes. The IR stores `Ellipse { cx, cy, width,
  height }`. It renders into the axis-aligned rectangle
  `(cx - width/2, cy - height/2, width, height)`.
- **Square**: `at (x, y)` is the top-left and `size <side>` is a single
  dimension, matching the specification example. It lowers to rectangle IR
  with `width = height = side`.
- **Triangle**: `at (x, y)` is the bounding-box top-left and `size <w> x <h>`
  is the bounding box. The triangle is **isosceles, apex-up**:
  - apex `(x + w/2, y)`;
  - base-left `(x, y + h)`;
  - base-right `(x + w, y + h)`.
  The transform origin is the bounding-box center.
- All three required properties (`position`/`center`, `size`, `fill`) are
  retained as `Option` in the AST. Missing properties use the existing stable
  codes `S003` (position), `S004` (size), `S006` (fill); non-positive sizes use
  `S001`; non-pixel dimensions use `S007`.
- Shape dimensions are pixels, consistent with M1. Fill is required in this
  task because stroke support is not implemented until TASK-016.
- `fill none` remains valid and lowers to no paint.

## Consequences

- A future task may add an explicit triangle vertex form; the current
  bounding-box convention should remain the canonical shorthand.
- Golden tests for these primitives should assert the apex-up triangle
  convention.
- If the specification later defines a different triangle orientation, this
  decision must be revised and golden images regenerated.
