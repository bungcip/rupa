# DECISION-016: Layer definition storage and validation

**Date**: 2026-11-10
**Task**: TASK-019 (Layer definition AST and validation)

## Context

Specification §6.1 defines reusable layer definitions with their own local
coordinate system and declared size. §6.2–§6.6 describe instances, ordering,
nested composition, and clipping, which are later Milestone 3 tasks. The AST
needed a representation that does not conflate definitions with drawn content.

## Decision

- **Separate storage**: `Document` has `children: Vec<DocumentChild>` for
  root drawing content and `layers: Vec<LayerDefinition>` for reusable
  definitions. Definitions are not `DocumentChild` variants, so they cannot be
  accidentally treated as instances.
- **LayerDefinition**: `{ name, size: Option<Size>, children, span }`.
  Children are existing primitive `DocumentChild` nodes and use layer-local
  coordinates (origin at the layer's top-left).
- **Parsing**: `layer "<name>"` accepts a `size` declaration and shape
  children. Nested layer definitions are not supported in this task.
- **Definitions do not draw**: lowering currently ignores definitions; a
  layer-only document produces an empty root scene. TASK-020 adds `place`
  instances and rendering.
- **Validation**:
  - missing layer size → `S015`
  - duplicate name (case-sensitive) → `S016`
  - non-positive/non-finite size → `S017`
  - empty name → `S018`
  - non-pixel size → `S007`
  - layer child shapes run through the same shape/paint validation as root
    children.
- **Names**: case-sensitive, matching §4.1. `"Cloud"` and `"cloud"` are
  distinct.
- **Forward references and cycles**: not applicable until instances exist
  (TASK-020/TASK-024).

## Consequences

- Adding instances can resolve names from `document.layers` without flattening
  definitions into root content.
- Layer IR storage is deferred until TASK-020; definitions are currently
  compile-time metadata.
- Duplicate detection is O(n) with a `HashSet`.
- Nested definitions, clipping, and z-index remain future decisions.
