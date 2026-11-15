# TASK-021: Local coordinate system handling

## Task Header
- **Task ID**: TASK-021
- **Title**: Local coordinate system handling
- **Milestone**: M3 — Layers and Composition
- **Priority**: Critical
- **Estimated Effort**: 4-6 hours
- **Dependencies**: TASK-020

## Description
Make layer-local coordinate handling explicit and shared between IR and
renderer. TASK-020 already renders definition children in local coordinates;
this task formalizes the mapping from layer-local space to parent/document
space, documents the origin and pivot, and adds tests for local origin,
instance transforms, and child transforms.

## Files to Create/Modify

| File | Action | Notes |
|------|--------|-------|
| `crates/rupa-ir/src/document.rs` | Modify | Add local-to-parent transform API |
| `crates/rupa-renderer/src/vector.rs` | Modify | Use shared coordinate mapping |
| `crates/rupa-cli/tests/cli.rs` | Modify | End-to-end local-origin test |
| `docs/decisions/DECISION-018.md` | Create | Local coordinate conventions |

## Semantics

- A layer definition has a local coordinate system whose origin `(0,0)` is
  its top-left corner.
- Definition child geometry remains in layer-local coordinates; instances do
  not mutate definitions.
- `place "Layer" at (x, y)` maps the layer's local origin to document position
  `(x, y)` (or to the containing layer's local parent space for nested use in
  TASK-023).
- Instance scale/rotation pivot on the layer bounds center in local space:
  `translate(x,y) * translate(cx,cy) * transform * translate(-cx,-cy)`.
- Child shape transforms are applied in local space and then composed with the
  instance parent transform.
- Negative local coordinates are allowed; clipping is TASK-025.
- The same coordinate mapping will be reusable for nested placements.

## Acceptance Criteria

- [x] IR exposes a layer-local → parent-space transform method
- [x] Mapping local origin with identity transform yields the instance position
- [x] Layer center maps to `(x + width/2, y + height/2)` under identity
- [x] Scale/rotation pivot on layer bounds center
- [x] Child shape transforms compose with the instance transform
- [x] Renderer uses the shared IR mapping (no duplicate matrix logic)
- [x] Multiple instances share local geometry without mutation
- [x] Unit tests for mapping and rendering
- [x] End-to-end CLI test for a non-origin local placement
- [x] `cargo test --workspace` passes
- [x] `cargo fmt -- --check` passes
- [x] `cargo clippy --all-targets --all-features -- -D warnings` passes

## Specification References

- §6.1: Layer definition local coordinates
- §6.2: Layer instance placement
- §11: Transform origin
- §17: Graphics IR

## Handoff Notes

**Completed Work**:
- Added `LayerDefinition::center()` and
  `LayerInstance::local_to_parent_transform()` to `rupa-ir`, documenting the
  local-origin and bounds-center pivot mapping.
- Renderer now consumes the shared IR mapping; duplicate private matrix logic
  removed.
- Added IR tests for local origin, center mapping, scale pivot, and definition
  immutability.
- Added renderer test for child transform composed with instance transform.
- Added CLI end-to-end test placing a non-origin local child.
- Recorded conventions in `docs/decisions/DECISION-018.md`.

**Remaining Work**:
- None for TASK-021.

**Test Results**:
- `cargo test --workspace`: passes (rupa-syntax 101, rupa-compiler 157,
  rupa-ir 22, rupa-geometry 9, rupa-image 7, rupa-renderer 45,
  rupa-cli 27 CLI + 5 integration).
- `cargo clippy --all-targets --all-features -- -D warnings`: passes.
- `cargo fmt -- --check`: passes.

**Known Issues**:
- Clipping to layer bounds remains TASK-025.
- Nested placement coordinate composition is prepared for TASK-023.
- Root shapes still render before instances; z-index is TASK-022.

**Exact Next Action**:
- Create `agent/tasks/TASK-022.md` from the backlog and implement z-index
  stable sorting.
