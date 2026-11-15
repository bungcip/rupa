# TASK-022: Z-index stable sorting

## Task Header
- **Task ID**: TASK-022
- **Title**: Z-index stable sorting
- **Milestone**: M3 — Layers and Composition
- **Priority**: Critical
- **Estimated Effort**: 4-6 hours
- **Dependencies**: TASK-020, TASK-021

## Description
Implement stable z-index ordering for a composition's children per
specification §6.3. Root composition children are direct shapes and layer
instances; TASK-020 rendered all root shapes first and then instances in source
order, an explicit temporary limitation. This task merges root shapes and
instances into one source-ordered child list and renders them by ascending
z-index with source declaration order as the stable tiebreaker.

## Files to Create/Modify

| File | Action | Notes |
|------|--------|-------|
| `crates/rupa-ir/src/document.rs` | Modify | Ordered root children and stable z-index order API |
| `crates/rupa-ir/src/lib.rs` | Modify | Re-export `RootChild` |
| `crates/rupa-compiler/src/lower.rs` | Modify | Preserve interleaved root order when lowering |
| `crates/rupa-renderer/src/vector.rs` | Modify | Render root children in stable z-index order |
| `crates/rupa-cli/tests/cli.rs` | Modify | End-to-end z-index ordering render |
| `docs/decisions/DECISION-019.md` | Create | Z-index ordering conventions |
| `agent/tasks/TASK-022.md` | Create | This document |

## Semantics

- Lower z-index values render behind higher values; ascending order.
- Children with equal z-index retain source declaration order (stable sort).
- A direct shape has no `z-index` property and participates at z-index `0`.
- An instance without `z-index` defaults to `0`, so documents that never use
  `z-index` still render in source declaration order (§2.3).
- `z-index` is an integer; non-integer values are already rejected with `S020`.
- Z-index is scoped to one composition parent. Only the root composition is
  sorted here; the same IR order API is reused for nested compositions in
  TASK-023.
- Instance transforms, opacity, and local-coordinate mapping are unchanged.
- Layer definitions remain immutable; sorting only orders instances.

## Acceptance Criteria

- [x] IR stores root shapes and instances in one source-ordered child list
- [x] IR exposes a stable render-order API with source order as tiebreaker
- [x] Lowering preserves source interleaving of shapes and instances
- [x] Renderer draws root children in stable ascending z-index order
- [x] Lower z-index renders behind higher regardless of source order
- [x] Equal z-index retains source declaration order
- [x] Direct root shapes participate at z-index `0`
- [x] Negative z-index values order behind zero and positive values
- [x] Instance transform/opacity behavior is unchanged
- [x] Unit tests for IR ordering and renderer compositing
- [x] End-to-end CLI test for out-of-source-order z-index rendering
- [x] `cargo fmt -- --check` passes
- [x] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [x] `cargo build --workspace` passes
- [x] `cargo test --workspace` passes

## Implementation Steps

1. Replace separate root `shapes`/`instances` IR vectors with ordered
   `RootChild` children; keep `push_shape`/`push_instance` conveniences.
2. Add stable z-index ordering over root children in `rupa-ir`.
3. Preserve interleaved order while lowering `Document::children`.
4. Render root children through the ordered API.
5. Add IR, renderer, and CLI tests.
6. Record conventions in `docs/decisions/DECISION-019.md`.
7. Run validation commands and update project state.

## Testing Requirements

- Unit tests: stable ordering for lower/equal/higher and negative z-index;
  root shape participation at zero; definition immutability.
- Integration tests: rendered overlap proves z-index wins over source order.
- Golden images: not required; pixel probes are deterministic and sufficient.

## Specification References

- §6.2: Layer instances
- §6.3: Z-index and stable ordering
- §2.3: Predictable defaults
- §23.3: Renderer tests (stable z-index ordering)

## Handoff Notes

**Completed Work**:
- Replaced separate root `shapes`/`instances` IR vectors with an ordered
  `children: Vec<RootChild>` list; `push_shape`/`push_instance` append in
  source order and `shapes()`/`instances()` read source-ordered subsets.
- Added `RootChild::z_index()` (shapes are `0`) and `stable_z_index_order`,
  exposed as `GraphicsDocument::children_in_render_order()`.
- Lowering preserves source interleaving; renderer draws root children in
  stable ascending z-index order.
- Added IR ordering tests, four renderer compositing tests, a lowering order
  test, and an end-to-end CLI z-index render test.
- Recorded conventions in `docs/decisions/DECISION-019.md`.

**Remaining Work**:
- None for TASK-022.

**Test Results**:
- `cargo test --workspace`: passes (rupa-syntax 101, rupa-compiler 158,
  rupa-ir 27, rupa-geometry 9, rupa-image 7, rupa-renderer 49,
  rupa-cli 28 CLI + 5 integration).
- `cargo clippy --all-targets --all-features -- -D warnings`: passes.
- `cargo build --workspace`: passes.
- `cargo fmt -- --check`: passes.

**Known Issues**:
- z-index ordering applies to the root composition only; nested composition
  is TASK-023.
- Layer bounds clipping remains TASK-025.
- Cycle detection remains TASK-024.

**Exact Next Action**:
- Create `agent/tasks/TASK-023.md` from the backlog and implement nested layer
  composition, reusing `stable_z_index_order` for nested children.
