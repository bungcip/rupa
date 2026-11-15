# DECISION-027: Implicit root layer

**Date**: 2026-11-12
**Task**: TASK-056 (Document model and root layer specification)

## Context

Drawing primitives could previously be children of the document or of a layer.
That gave the root composition different semantics from every other layer and
forced validation, lowering, and rendering to maintain two ownership paths. The
language requires all drawing primitives to belong to a layer.

## Options Considered

1. Reject primitives at file top level and require an explicit layer for every
   drawing.
2. Keep document-level primitives as a separate composition path.
3. Accept top-level primitives as shorthand and lower them into a compiler-created
   root layer.

## Decision

Option 3. Every document has exactly one implicit root layer whose local
coordinate system is the document coordinate system and whose bounds equal the
canvas. Top-level primitives and placements are lowered into this layer in source
order. The root layer adds no clipping beyond the output canvas. It is not a
reusable definition and cannot be referenced by `place`.

The layer name `root` is reserved exactly (case-sensitive, like all layer
names). An explicit `layer "root"` definition is rejected with a dedicated
diagnostic (`S029`).

## Consequences

- Compiler, IR, and renderer can use one ownership model for all drawing.
- Existing root output must remain pixel-stable after migration.
- `place` and `z-index` ordering at top level now operate within the root layer.
- Formatter and `inspect` output must understand the implicit layer even though
  it is not author-written.
- Explicit layer definitions and instances keep their current semantics.
