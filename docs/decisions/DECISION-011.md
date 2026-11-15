# DECISION-011: Line, polyline, and stroke conventions

**Date**: 2026-11-10
**Task**: TASK-014 (Line and polyline primitives)

## Context

Specification §7.3 defines `line` and `polyline`; §7.5 requires geometry and
appearance to be separate and allows stroke-only shapes. Full fill/stroke IR
separation is TASK-016 and cap/join/dash properties are TASK-018, but lines
cannot be visible without a minimal stroke path.

## Decision

- **Line syntax**: `line from (x1, y1) to (x2, y2)` with a block containing
  `stroke <color>` and optional `width <dimension>`.
- **Polyline syntax**: `polyline` with a nested `points` block, `stroke`, and
  optional `width`.
- **Open polylines**: the last point is not connected to the first. Paths are
  not closed in this task; explicit path closure is TASK-015.
- **No fill**: lines and polylines have `fill: None`. They render only through
  their stroke.
- **Stroke required**: a missing `stroke` declaration is `S010`. `stroke none`
  is accepted and lowers to no stroke (an invisible but valid line).
- **Width**: optional, default `1px`. Explicit widths must be positive pixels;
  invalid widths are `S011`, non-pixel widths `S007`.
- **Counts/endpoints**: line requires both endpoints (`S003` if missing);
  polyline requires `points` (`S008`) with at least two points (`S012`).
- **IR stroke**: `ShapeNode.stroke` is now `Option<Stroke>` where
  `Stroke { paint: PaintId, style: StrokeStyle }`. This adds the paint
  reference needed for stroking. `StrokeStyle::new(width)` defaults to
  butt caps, miter joins, no dash, and miter limit 4.0.
- **Renderer**: open paths are stroked through tiny-skia with the declared
  paint, width, opacity, and transform. Fill and stroke are drawn
  independently when both are present; this is a first step toward TASK-016.
- Geometry transform origins: line midpoint and polyline bounding-box center.

## Consequences

- The AST `Stroke` declaration is separate from the IR `Stroke` paint/style
  pair; TASK-016 may refine the separation further.
- Cap/join/dash syntax is not parsed yet; only defaults are rendered.
- `stroke none` is valid but produces no visible path.
