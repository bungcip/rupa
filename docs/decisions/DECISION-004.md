# DECISION-004: Paint IR and color conventions

**Date**: 2026-11-10
**Task**: TASK-007 (Solid fill paint representation)

## Context

Specification §4.3 defines named colors and hex strings. §8.1 describes solid
fill and §17 requires geometry and paint to be separate. TASK-007's
acceptance criteria name `R300`/`R301` for invalid hex and unknown named
colors; DECISION-001 requires category-prefixed stable codes.

TASK-006 introduced geometry-only `ShapeNode` state and a duplicate `PaintId`
type in the geometry module, which was removed.

## Decision

- **IR color**: `rupa_ir::paint::Color { r, g, b, a }` with straight-alpha
  `f32` components in `0.0..=1.0`. Renderers convert with
  `to_premultiplied()` at the compositing boundary. 8-bit conversion helpers
  (`from_rgba8`, `from_rgb8`, `to_rgba8`) round to nearest and clamp.
- **IR paint**: `Paint::Solid(Color)`, extensible with future gradient and
  pattern variants. `Paint` is `Copy`.
- **Arena storage**: `PaintStore` owns paints and assigns stable
  insertion-order `PaintId`s. `ShapeNode.fill` is `Option<PaintId>`; `None`
  means no fill paint.
- **Named colors** are exactly the ten specification names, lowercase and
  case-sensitive. No `grey` alias. Unknown names produce `T002` (task-doc
  `R301`) with the valid-name list and a Levenshtein suggestion when close.
- **Hex colors** are strings that must start with `#`, followed by exactly 6
  or 8 hexadecimal digits (case-insensitive). Missing `#`, wrong length, and
  non-hex characters produce `T001` (task-doc `R300`) with format help.
- **`fill none`**: the AST `Fill.color` is `Option<Color>`; `None` represents
  explicit `fill none`. Lowering returns `Option<Paint>`/`Option<PaintId>` =
  `None` and allocates nothing. An absent `fill` declaration remains a shape
  error (`S006`); an explicit `fill none` satisfies the declaration and draws
  no fill.
- Named, hex, and (future) `rgb`/`rgba` syntax all lower to the same IR
  `Color`; no renderer-specific color representation is exposed.

## Consequences

- Existing IR consumers use `Color` and the single `PaintId`.
- `rupa-compiler` validates fill colors and document background colors through
  `PaintValidator`, reporting `T001`/`T002`.
- `#RRGGBBAA` alpha is straight alpha; premultiplication happens at render
  time.
- Adding gradient paint later extends `Paint` without changing geometry.
