# DECISION-015: Stroke cap and join properties

**Date**: 2026-11-10
**Task**: TASK-018 (Stroke style properties)

## Context

Specification §7.6 lists width, cap, join, dash, and miter-limit, and says to
implement core width, cap, and join first. Width already existed; cap/join
were stored in IR defaults but not expressible in source.

## Decision

- **Syntax**: `cap butt|round|square` and `join miter|round|bevel` are
  primitive properties accepted by every shape that supports a stroke:
  closed shapes, lines, polylines, and paths.
- **Canonical values**: only the specified spellings are accepted. Invalid
  values are parser errors (`P003`) with a list of valid options; no synonyms
  are added.
- **Ordering**: `cap`/`join` may appear before or after `stroke`; they
  configure a shared stroke declaration. This required merging a later
  `stroke` color into an existing style declaration instead of replacing it.
- **Defaults**: `butt` cap and `miter` join, matching IR defaults.
- **Without stroke**: cap/join are retained but have no visible effect.
- **Width**: remains a positive pixel dimension (`S011` for non-positive,
  `S007` for non-pixel units).
- **IR/renderer**: AST `StrokeCap`/`StrokeJoin` map to IR `LineCap`/`LineJoin`,
  then to tiny-skia. Rendering uses tiny-skia's native cap/join behavior.
- **Deferred**: `dash` and `miter-limit` are not parsed in this task. IR fields
  retain defaults (`dash: None`, `miter_limit: 4.0`).

## Consequences

- Cap/join are deterministic and shared across all primitives.
- Adding dash later requires a new value syntax and tiny-skia `StrokeDash`
  mapping; `miter-limit` requires a dimensionless validation rule.
- The parser's stroke merge preserves style properties regardless of property
  order.
