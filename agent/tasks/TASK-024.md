# TASK-024: Cycle detection for layer references

## Task Header
- **Task ID**: TASK-024
- **Title**: Cycle detection for layer references
- **Milestone**: M3 — Layers and Composition
- **Priority**: Critical
- **Estimated Effort**: 3-4 hours
- **Dependencies**: TASK-023

## Description
Reject recursive layer-reference cycles during semantic validation
(specification §6.5). A layer definition may place other definitions, but a
definition must not reach itself through nested placements. TASK-023 added
nested composition and a renderer guard; this task adds the compiler
diagnostic so cyclic documents are rejected before lowering or rendering.

## Files to Create/Modify

| File | Action | Notes |
|------|--------|-------|
| `crates/rupa-syntax/src/diagnostic.rs` | Modify | Add stable `S021` code |
| `crates/rupa-compiler/src/layer.rs` | Modify | Depth-first cycle detection and tests |
| `crates/rupa-cli/tests/cli.rs` | Modify | End-to-end `check`/`render` rejection |
| `docs/decisions/DECISION-021.md` | Create | Cycle detection conventions |
| `agent/tasks/TASK-024.md` | Create | This document |

## Semantics

- The graph has one node per layer definition. An edge `A -> B` exists for
  every nested `place "B"` inside layer `A`.
- Root-level placements do not create graph edges; only definitions can be
  referenced, so only nested placements can form a cycle.
- Every definition is checked, whether or not it is instantiated.
- A cycle is reported as error `S021` at the `place` that closes the cycle,
  with the cycle path in the message (for example `` `A` -> `B` -> `A` ``).
- One diagnostic is reported per back edge found by depth-first search;
  independent cycles are each reported.
- Edges to unknown definitions are skipped during cycle detection because
  `S019` already reports them.
- Detection is deterministic: definitions and children are visited in source
  order.
- A definition that places the same target more than once without forming a
  cycle is valid.
- Compilation and rendering must not proceed when `S021` is present.
- The renderer's `CyclicLayer` guard from TASK-023 remains as defense in depth
  for hand-built IR.

## Acceptance Criteria

- [x] `S021` is a documented stable diagnostic code
- [x] Self-referential layer is rejected (`A` places `A`)
- [x] Two-layer and longer cycles are rejected
- [x] Cycle message includes the reference path
- [x] Diagnostic points at the placement that closes the cycle
- [x] Valid acyclic nests and repeated non-cyclic placements still pass
- [x] Unknown nested references produce `S019`, not a spurious `S021`
- [x] Root-level placements do not trigger cycle detection
- [x] Uninstantiated cyclic definitions are still rejected
- [x] CLI `check` exits non-zero with `error[S021]`
- [x] CLI `render` rejects the document and writes no PNG
- [x] Unit tests cover self, two-node, multi-node, and acyclic graphs
- [x] `cargo fmt -- --check` passes
- [x] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [x] `cargo build --workspace` passes
- [x] `cargo test --workspace` passes

## Implementation Steps

1. Add `RECURSIVE_LAYER_REFERENCE` = `S021` and its stability test.
2. Build the nested-placement graph from layer definitions.
3. Run deterministic depth-first search with visiting/done marks.
4. Report `S021` at each back edge with the cycle path.
5. Integrate detection into layer validation.
6. Add compiler and CLI tests.
7. Record conventions in `docs/decisions/DECISION-021.md`.
8. Run validation commands and update project state.

## Testing Requirements

- Unit tests: self cycle, two-node cycle, three-node cycle, acyclic nesting,
  repeated placement, unknown target, uninstantiated cycle, source-order
  determinism.
- Integration tests: `check` reports `S021`; `render` fails without output.
- Golden images: not applicable.

## Specification References

- §6.5: Nested composition (reject recursive layer-reference cycles)
- §6.6: Forward references
- §23.2: Semantic tests (recursive layer cycle)

## Handoff Notes

**Completed Work**:
- Added stable `S021` (`RECURSIVE_LAYER_REFERENCE`) to the diagnostic codes.
- Added deterministic depth-first cycle detection over nested `place` edges in
  `LayerValidator::validate_cycles`, integrated into `validate_layers`.
- Reports one error per back edge at the closing placement, with the cycle
  path in the message and actionable help text.
- Skips unknown targets so `S019` is not duplicated or turned into `S021`.
- Added compiler tests for self, two-node, three-node, uninstantiated,
  acyclic/repeated, unknown-target, and span cases, plus a CLI test proving
  `check` reports `S021` and `render` writes no PNG.
- Recorded conventions in `docs/decisions/DECISION-021.md`.

**Remaining Work**:
- None for TASK-024.

**Test Results**:
- `cargo test --workspace`: passes (rupa-syntax 102, rupa-compiler 171,
  rupa-ir 28, rupa-geometry 9, rupa-image 7, rupa-renderer 55,
  rupa-cli 30 CLI + 5 integration).
- `cargo clippy --all-targets --all-features -- -D warnings`: passes.
- `cargo build --workspace`: passes.
- `cargo fmt -- --check`: passes.

**Known Issues**:
- A strongly connected component can report one `S021` per back edge; this is
  deterministic and intentional.
- Layer bounds clipping remains TASK-025.

**Exact Next Action**:
- Create `agent/tasks/TASK-025.md` and implement clipping of layer content to
  each layer's declared bounds.
