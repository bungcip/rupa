# DECISION-017: Layer instance placement and group transforms

**Date**: 2026-11-10
**Task**: TASK-020 (Layer instance placement and transforms)

## Context

Specification §6.2 defines `place` instances with independent position,
scale/rotation, opacity, and z-index. §6.6 requires forward references. The
renderer already applies a matrix around each primitive's center; layer
instances need a parent transform that applies to all definition children as a
group.

## Decision

- **AST**: `LayerInstance { name, position, transform, z_index, span }` and
  `DocumentChild::Place`. Instances live in document/root child order.
- **Syntax**: `place "<name>" at (x, y)` with an indented block containing
  `rotate`, `scale`, `opacity`, and `z-index`.
  - Missing position → `S003`
  - Empty name → `S018`
  - Unknown definition → `S019`
  - Non-integer/out-of-range z-index → `S020`
  - Transform values reuse `S014` validation.
- **Reference resolution**: all layer definitions are collected before
  validating instances, so declarations may appear before or after placements.
  Names remain case-sensitive.
- **IR**: `GraphicsDocument` gains `layers: Vec<LayerDefinition>` and
  `instances: Vec<LayerInstance>`; root `shapes` remain for direct drawings.
  Definitions store local shape geometry and reference the shared paint store.
- **Position**: instance `(x, y)` is the layer's local top-left origin.
- **Group transform**: instance transform is applied around the layer's
  declared bounds center:
  `translate(x, y) * translate(w/2, h/2) * transform * translate(-w/2, -h/2)`.
  The renderer prepends this parent matrix to each shape's own center-pivoted
  transform. Definitions are cloned logically at render time, never mutated.
- **Opacity**: instance opacity multiplies each child shape opacity.
- **Rendering order**: root shapes render first, then instances in source
  order. Stable z-index sorting and interleaving with root shapes are TASK-022;
  this is an explicit temporary limitation.
- **Clipping**: not yet applied; TASK-025 clips to layer bounds.
- **Nested instances**: definitions currently contain shape children only;
  nested placements are TASK-023.

## Consequences

- Two or more instances of one definition render independently.
- Instance transforms affect only that instance's rendered copy.
- Forward references work regardless of declaration order.
- TASK-022 must merge root shapes/instances and sort by z-index without
  changing this transform convention.
