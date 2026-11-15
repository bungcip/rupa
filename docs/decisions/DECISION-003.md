# DECISION-003: Shape validation representation and conventions

**Date**: 2026-11-10
**Task**: TASK-006 (Rectangle and circle primitives)

## Context

Specification §7.1 requires positive dimensions and radii, and defines
position conventions: rectangle `at` is the top-left corner and circle
`center at` is the center. TASK-006 also requires specific errors for missing
position, size, radius, and fill.

The parser originally made `at`/`center at` hard syntax errors and filled
missing sizes/radii with `0.0`, so the compiler could not distinguish a
missing required property from an explicit zero value. Negative literals were
also rejected as syntax errors, so S001/S002 could never be emitted.

TASK-006's acceptance criteria name `R200`/`R201` for negative dimensions and
`R202`–`R205` for missing properties. Per DECISION-001, codes use category
prefixes; the compiler codes are:

| Task-doc code | Implemented code | Meaning |
|---|---|---|
| `R200` | `S001` | negative/non-positive rectangle dimension |
| `R201` | `S002` | negative/non-positive circle radius |
| `R202` | `S003` | missing rectangle `at` / circle `center at` |
| `R203` | `S004` | missing rectangle `size` |
| `R204` | `S005` | missing circle `radius` |
| `R205` | `S006` | missing `fill` |
| — | `S007` | shape coordinate/dimension not in pixels |

## Decision

- Required shape properties are represented as `Option` in the AST:
  `Rectangle.position`, `Rectangle.size`, `Circle.center`, and
  `Circle.radius`. The parser retains the shape when a required property is
  missing so the compiler reports a targeted semantic diagnostic with a
  suggestion.
- Missing properties are semantic errors (`S003`–`S006`), not parse errors.
  Malformed values (for example `at (10px 10px)`) remain syntax errors.
- A leading `-` before a dimension literal is parsed as part of the value, so
  negative dimensions/radii reach semantic validation and produce `S001`/`S002`.
- Shape coordinates and dimensions must use pixel units in the first
  implementation (`S007`). Negative *coordinates* remain valid; only
  dimensions and radii must be positive.
- Rectangle `at` is the top-left corner; circle `center at` is the center.
- In the IR, geometry and paint stay separate. `ShapeNode` references
  `PaintId` from the paint module; there is exactly one `PaintId` type.

## Consequences

- `DocumentChild::Rectangle`/`Circle` consumers must unwrap optional
  properties after validation. No lowering exists yet; TASK-008 will do this.
- Values `0px` are invalid dimensions; they are no longer used as a sentinel
  for "missing".
- Future units require a reference-resolution stage before S007 can be
  relaxed.
