# DECISION-012: Path command and Bézier conventions

**Date**: 2026-11-10
**Task**: TASK-015 (Path geometry)

## Context

Specification §7.4 defines path commands `start at`, `line to`, `curve to`,
`quad to`, and `close`. It shows cubic controls as an indented
`control (c1) and (c2)` line but does not show the quadratic control syntax.
It also does not state whether fill/stroke are required.

## Decision

- **Commands**:
  - `start at (x, y)` begins a subpath.
  - `line to (x, y)` adds a straight segment.
  - `curve to (x, y)` requires an indented `control (c1) and (c2)` block and
    emits a cubic Bézier.
  - `quad to (x, y)` requires an indented `control (c)` block and emits a
    quadratic Bézier. This is the chosen interpretation of the unspecified
    quadratic control syntax; it mirrors the cubic control block.
  - `close` closes the current subpath.
- **Multiple subpaths**: additional `start at` commands are allowed. Each
  begins a new subpath. The path must start with `start at` (`S013`).
- **Fill/stroke**: both are optional. A path may have fill only, stroke only,
  both, or neither. Width is optional and defaults to `1px` when a stroke is
  present; explicit widths must be positive pixels.
- **Open fills**: a path that is filled without an explicit `close` is
  implicitly closed by the fill rule, matching standard vector semantics.
  Explicit `close` is recommended for stroked outlines.
- **IR**: `Geometry::Path { commands }` with `MoveTo`, `LineTo`, `CubicTo`,
  `QuadTo`, and `Close` command variants.
- **Transform origin**: the center of the bounding box containing all explicit
  endpoints and Bézier control points. This is conservative and deterministic.
- **Coordinates**: document coordinates in pixels for this milestone;
  layer-local coordinates arrive in TASK-021.

## Consequences

- Quadratic path syntax is now defined as a nested single-control block; a
  future spec change requires updating parser tests and this decision.
- Fill-only open paths follow the renderer fill rule rather than introducing a
  separate implicit-close flag.
- Stroke cap/join/dash still use defaults until TASK-018.
