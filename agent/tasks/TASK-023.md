# TASK-023: Nested layer composition

## Task Header
- **Task ID**: TASK-023
- **Title**: Nested layer composition
- **Milestone**: M3 — Layers and Composition
- **Priority**: Critical
- **Estimated Effort**: 6-8 hours
- **Dependencies**: TASK-022

## Description
Allow a layer definition to contain `place` instances of other layer
definitions (specification §6.5). Nested placements use the containing layer's
local coordinate system, compose their transforms through the hierarchy, and
order their children by z-index within each composition parent.

Definitions resolve independently of declaration order, so a layer may place a
definition declared later in the document. Static recursive-cycle rejection is
TASK-024; this task adds a render-time guard so a cyclic IR returns an error
instead of recursing without bound.

## Files to Create/Modify

| File | Action | Notes |
|------|--------|-------|
| `crates/rupa-syntax/src/parser.rs` | Modify | Accept `place` inside layer blocks |
| `crates/rupa-syntax/src/ast.rs` | Modify | Document nested children |
| `crates/rupa-compiler/src/layer.rs` | Modify | Validate nested instances |
| `crates/rupa-compiler/src/lower.rs` | Modify | Lower nested shapes and instances in order |
| `crates/rupa-ir/src/document.rs` | Modify | Shared `CompositionChild`; layer children list |
| `crates/rupa-ir/src/lib.rs` | Modify | Re-export `CompositionChild` |
| `crates/rupa-renderer/src/vector.rs` | Modify | Recursive composition rendering and cycle guard |
| `crates/rupa-cli/tests/cli.rs` | Modify | End-to-end nested composition render |
| `docs/decisions/DECISION-020.md` | Create | Nested composition conventions |
| `agent/tasks/TASK-023.md` | Create | This document |

## Semantics

- A `layer` block accepts `size`, drawing primitives, and `place` instances.
- A nested `place "Name" at (x, y)` positions the referenced definition's local
  origin in the containing layer's local space.
- Nested instance transforms pivot on the referenced layer's bounds center and
  compose with all ancestor transforms: `parent * local_to_parent`.
- Instance opacity multiplies down the hierarchy; effective child opacity is
  the product of every enclosing instance opacity and the shape opacity.
- z-index orders siblings within one composition parent. Root children and each
  layer's children are sorted independently; equal z-index keeps source order.
- Direct shapes inside a layer participate at z-index `0`, like root shapes.
- References resolve globally by name regardless of declaration order.
- Definitions are immutable; a definition placed many times renders
  independently at each position.
- Clipping to layer bounds remains TASK-025.
- Static cycle rejection remains TASK-024. The renderer returns
  `RenderError::CyclicLayer` when a definition is re-entered in its own active
  ancestry, so cyclic IR cannot overflow the stack.

## Acceptance Criteria

- [x] Parser accepts `place` inside layer blocks and retains its properties
- [x] Invalid layer child help mentions `place`
- [x] Nested instances are validated: unknown layer `S019`, missing position
      `S003`, fractional z-index `S020`, transform `S014`
- [x] IR stores root and layer composition children as ordered
      `CompositionChild` lists
- [x] `LayerDefinition` exposes stable z-index render order for its children
- [x] Lowering preserves source interleaving of nested shapes and instances
- [x] Renderer draws nested instances with composed transforms
- [x] Nested instance opacity multiplies ancestor opacity
- [x] Nested z-index ordering is independent per composition parent
- [x] Forward references to later definitions work when nested
- [x] Cyclic IR returns `RenderError::CyclicLayer` without stack overflow
- [x] Multiple instances of nested definitions render independently
- [x] Unit tests for parsing, validation, lowering, IR, and renderer
- [x] End-to-end CLI test for a nested composition
- [x] `cargo fmt -- --check` passes
- [x] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [x] `cargo build --workspace` passes
- [x] `cargo test --workspace` passes

## Implementation Steps

1. Parse `place` blocks inside `layer` definitions.
2. Generalize IR `RootChild` to `CompositionChild`; store ordered children on
   `LayerDefinition`.
3. Validate instances found in root and nested children against all names.
4. Lower layer children through the same ordered lowering as root children.
5. Render compositions recursively with composed transforms, opacity, and
   per-parent stable z-index ordering; guard re-entrant definitions.
6. Add parser, validation, lowering, IR, renderer, and CLI tests.
7. Record conventions in `docs/decisions/DECISION-020.md`.
8. Run validation commands and update project state.

## Testing Requirements

- Unit tests: nested parse, nested validation codes, lowering order, IR layer
  ordering, renderer transform/opacity/z-index/forward-reference/cycle guard.
- Integration tests: nested layer render with pixel probes.
- Golden images: not required; deterministic pixel probes are sufficient.

## Specification References

- §6.1: Layer definition
- §6.2: Layer instances
- §6.3: Z-index and stable ordering
- §6.5: Nested composition
- §6.6: Forward references
- §11: Transformations and opacity

## Handoff Notes

**Completed Work**:
- Parser accepts `place` blocks inside `layer` definitions; help text now
  mentions `place`.
- IR `RootChild` generalized to `CompositionChild`; `LayerDefinition` stores
  ordered `children` and exposes `children_in_render_order`.
- Root and nested instances validated against all definition names, including
  forward references.
- Shared `lower_composition_children` preserves source interleaving for both
  root and nested children.
- Renderer recurses through nested instances with composed transforms and
  multiplied opacity; each composition sorts its own children by z-index.
- Renderer guards re-entrant definitions with `RenderError::CyclicLayer` so
  cyclic IR cannot overflow the stack.
- Added parser, validation, lowering, IR, renderer, and CLI tests.
- Recorded conventions in `docs/decisions/DECISION-020.md`.

**Remaining Work**:
- None for TASK-023.

**Test Results**:
- `cargo test --workspace`: passes (rupa-syntax 102, rupa-compiler 164,
  rupa-ir 28, rupa-geometry 9, rupa-image 7, rupa-renderer 55,
  rupa-cli 29 CLI + 5 integration).
- `cargo clippy --all-targets --all-features -- -D warnings`: passes.
- `cargo build --workspace`: passes.
- `cargo fmt -- --check`: passes.

**Known Issues**:
- Static cycle rejection is TASK-024; the renderer guard only prevents a
  stack overflow for cyclic IR.
- Clipping to layer bounds remains TASK-025.
- `place` still requires an indented property block even when no properties are
  given (existing TASK-020 parser behavior).
- Nesting depth is unbounded and uses call-stack recursion.

**Exact Next Action**:
- Create `agent/tasks/TASK-024.md` and implement static cycle detection for
  layer references, reporting a semantic diagnostic.
