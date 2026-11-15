# DECISION-005: Transform semantics and vector renderer boundary

**Date**: 2026-11-10
**Task**: TASK-008 (Basic renderer with tiny-skia)

## Context

Specification §11 requires an explicit transform origin and a documented
composition order. §18.1 requires a documented internal color representation
with premultiplied alpha for compositing. TASK-008 requires rendering IR
geometry with fills, transforms, opacity, and a pixel-buffer output.

## Decision

### Transform matrix

- `Transform` is a 3x2 affine matrix in column-vector form:
  `x' = a*x + c*y + e`, `y' = b*x + d*y + f`.
- `Transform::multiply(other)` computes `self * other`: `other` is applied
  first, then `self`. `TransformBuilder` keeps appending to the right, so the
  first builder call is the outermost transform.
- Rotation angles are radians. In Rupa's Y-down system a positive rotation is
  clockwise on screen; a quarter turn maps `(1, 0)` to `(0, 1)`. This fixes the
  previous rotated matrix, which rotated counter-clockwise.
- `rotate_about` and `scale_about` compose
  `translate(pivot) * operation * translate(-pivot)`.

### Transform origin

Primitive transforms pivot on the primitive's geometric center:

- rectangle: center of its bounding box;
- circle: its center.

The renderer composes `translate(origin) * shape.transform * translate(-origin)`
before rasterizing.

### Renderer boundary and color

- The renderer consumes `RenderDocument`: canvas dimensions, optional
  background `Color`, a `PaintStore`, and ordered `ShapeNode`s. It returns
  `RenderOutput { width, height, pixels }`.
- Output pixels are premultiplied RGBA8 (tiny-skia's native format), row-major.
  `RenderOutput::pixel` exposes one premultiplied pixel.
- IR colors remain straight-alpha; the renderer multiplies shape opacity into
  the alpha channel and tiny-skia performs premultiplication during
  rasterization/compositing.
- Background omission means transparent. The canvas starts zeroed.
- Shape `fill: None` draws nothing. A `PaintId` missing from the store is
  `RenderError::MissingPaint`; degenerate geometry is
  `RenderError::InvalidGeometry`; zero canvas dimensions are
  `RenderError::InvalidDimensions`.
- Rendering order is source order. Opacity is clamped defensively to
  `0.0..=1.0` in the renderer; semantic validation of out-of-range opacity is
  a future compiler task.

### Dependency

`tiny-skia = "0.12"` is used for CPU path rasterization and compositing. No
other geometry dependency (`kurbo`/`glam`) was added; the existing matrix type
is sufficient.

## Consequences

- Rotation tests now expect clockwise screen rotation.
- TASK-009 can encode `RenderOutput.pixels` directly as PNG; the buffer is
  already premultiplied, so encoding must unpremultiply or define the PNG
  contract explicitly.
- TASK-010 needs AST→`RenderDocument` lowering before it can render source.
