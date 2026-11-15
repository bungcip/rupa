# DECISION-020: Nested layer composition

**Date**: 2026-11-10
**Task**: TASK-023 (Nested layer composition)

## Context

Specification §6.5 allows a layer definition to contain `place` instances of
other definitions. Nested placements use the containing layer's local
coordinate system, compose through the hierarchy, and must order siblings by
z-index within each composition. TASK-022 introduced an ordered root child list
(`RootChild`) to merge direct shapes and instances for stable z-index ordering.

## Decision

- **AST and syntax**: a `layer` block accepts `size`, drawing primitives, and
  `place` blocks. `LayerDefinition::children` already held `DocumentChild`, so
  the AST stores nested placements without a new node type. `place` retains its
  existing requirement of an indented property block.
- **IR representation**: `RootChild` is renamed to `CompositionChild`, and
  `LayerDefinition.children: Vec<CompositionChild>` uses the same ordered
  child representation as the document root. `CompositionChild` holds a
  `ShapeNode` or `LayerInstance`; z-index ordering is per composition.
- **Ordering**: `LayerDefinition::children_in_render_order()` reuses
  `stable_z_index_order`, so nested children sort by ascending z-index with
  source order as the stable tiebreaker, independently of the root and of any
  other layer.
- **Lowering**: one `lower_composition_children` path lowers root and layer
  children, preserving source interleaving of shapes and nested instances.
  Paints for all definitions still go to the document paint arena.
- **Validation**: all definition names are collected before instances are
  checked. Root and nested instances share the same validation, so unknown
  references are `S019`, missing positions `S003`, fractional z-index `S020`,
  and invalid transforms `S014`. References resolve independent of declaration
  order (§6.6).
- **Rendering**: `render_composition` recurses through nested instances.
  A nested instance's parent transform is
  `enclosing_parent * instance.local_to_parent_transform(layer)`, and
  opacity multiplies down the hierarchy. Each composition applies its own
  stable z-index ordering. Definitions remain immutable.
- **Cycles**: static rejection of recursive definitions is TASK-024. As a
  safety measure for the new recursion, the renderer tracks definition names on
  the active ancestry and returns `RenderError::CyclicLayer` on re-entry, so
  cyclic IR fails instead of overflowing the stack. This is a renderer guard,
  not a language diagnostic.
- **Clipping and nesting depth**: clipping to layer bounds remains TASK-025.
  No explicit nesting-depth limit is imposed; finite documents are unbounded in
  depth and use call-stack recursion.

## Consequences

- Nested placements render in the parent's local coordinate space with
  composed scale/rotation and multiplied opacity.
- z-index remains scoped to one composition parent, as §6.3 requires.
- Root and layer content share one IR child representation, simplifying later
  raster and clipping work.
- A cyclic definition now produces a render error until TASK-024 rejects it
  during validation.
