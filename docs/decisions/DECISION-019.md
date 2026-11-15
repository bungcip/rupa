# DECISION-019: Composition ordering and stable z-index

**Date**: 2026-11-10
**Task**: TASK-022 (Z-index stable sorting)

## Context

Specification §6.3 requires that lower z-index values render behind higher
values, that equal values keep source declaration order, and that the default
z-index is source declaration order. TASK-020 stored root shapes and root layer
instances in separate IR vectors and rendered every shape before every
instance, an explicit temporary limitation recorded in DECISION-017.

## Options considered

1. **Order only instances among themselves.** Smallest change, but root shapes
   would remain a fixed base layer and the documented intent in DECISION-017
   to merge root shapes and instances would not be met.
2. **Keep separate storage plus a source-order index list.** Avoids touching
   existing shape access sites, but introduces two representations of the same
   composition that can silently disagree.
3. **One ordered child list owned by the composition.** Shapes and instances
   are stored in source declaration order; ordering is computed at render
   time. Requires updating IR consumers once, and gives TASK-023's nested
   compositions the same representation.

## Decision

Option 3.

- **IR**: `GraphicsDocument.children: Vec<RootChild>` replaces separate root
  `shapes` and `instances` vectors. `RootChild` is `Shape(ShapeNode)` or
  `Instance(LayerInstance)`, stored in source declaration order.
  `push_shape`/`push_instance` remain conveniences that append in source order.
- **Ordering key**: an instance's integer `z_index`; a direct shape has no
  `z-index` property and participates at `0`.
- **Default**: an instance without `z-index` lowers to `0`. Documents that
  never declare `z-index` therefore render in source declaration order, which
  satisfies the §2.3 default.
- **Sort**: ascending `z_index` using a stable sort, so equal values keep
  source order. The implementation is
  `stable_z_index_order(&[RootChild]) -> Vec<&RootChild>` in `rupa-ir`; the
  renderer calls it through `GraphicsDocument::children_in_render_order`.
- **Scope**: z-index orders siblings within one composition parent. This task
  applies it to the root composition; TASK-023 reuses the same helper for
  nested compositions.
- **Unchanged behavior**: instance transform pivots, opacity multiplication,
  local-coordinate mapping, paint storage, and layer definition immutability
  are not affected by ordering.

## Consequences

- A direct shape and a default instance interleave by source order instead of
  all shapes rendering first.
- Explicit positive z-index lifts an instance in front of direct shapes;
  negative z-index places it behind them.
- Root composition storage now matches the ordered children needed by nested
  layer composition.
- IR consumers read root shapes/instances through `shapes()`/`instances()`
  accessors, which preserve source order.
