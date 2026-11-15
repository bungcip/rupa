# DECISION-018: Layer-local coordinate mapping

**Date**: 2026-11-10
**Task**: TASK-021 (Local coordinate system handling)

## Context

Specification §6.1 states that layer children use a local coordinate system
with `(0,0)` at the layer's top-left. §6.2 places instances in a parent
coordinate space and §11 recommends the layer bounds center as transform
origin. TASK-020 rendered local definitions correctly but kept the mapping
private to the renderer.

## Decision

- **Local space**: each `LayerDefinition` has a local coordinate system with
  origin at its top-left. Child geometry is stored unchanged in local
  coordinates.
- **Mapping**: `LayerInstance::local_to_parent_transform(layer)` returns:
  `translate(x, y) * translate(cx, cy) * instance.transform *
  translate(-cx, -cy)`, where `(cx, cy)` is the layer bounds center. Local
  `(0,0)` maps to the instance's `(x,y)` when the transform is identity.
- **Shared implementation**: the mapping lives in `rupa-ir` and the renderer
  consumes it; there is no duplicate matrix formula in the backend. For root
  instances the parent space is the document. The same API will be reused for
  nested placements in TASK-023.
- **Composition**: each child shape's own transform is center-pivoted in local
  space, then composed with the instance parent matrix
  (`parent * local`). Definitions and other instances are never mutated.
- **Coordinates**: negative local coordinates are valid. Clipping to layer
  bounds is TASK-025; it is not implied by coordinate mapping.
- **Opacity**: instance opacity remains a separate multiplier; it does not
  affect coordinate mapping.
- **Root content**: document-level shapes use document coordinates directly.

## Consequences

- Local origin, center mapping, scale pivot, and child-transform composition
  are covered by IR and renderer tests.
- Nested layers can compose parent transforms without changing the local
  mapping formula.
- Clipping remains a separate rendering concern.
