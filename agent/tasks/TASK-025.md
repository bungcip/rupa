# TASK-025: Layer bounds clipping

## Task Header
- **Task ID**: TASK-025
- **Title**: Layer bounds clipping
- **Milestone**: M3 — Layers and Composition
- **Priority**: Critical
- **Estimated Effort**: 3-4 hours
- **Dependencies**: TASK-023, TASK-024

## Description
Clip rendered layer content to each layer definition's declared bounds
(specification §6.4). A layer has a finite local viewport equal to its declared
`size`; shapes and nested placements that extend beyond it must not paint
outside the layer's bounds in the parent composition.

Clipping is always enabled for this milestone. No configurable `clip` property
is introduced, per the specification's guidance for the first implementation.

## Files to Create/Modify

| File | Action | Notes |
|------|--------|-------|
| `crates/rupa-renderer/src/vector.rs` | Modify | Device-space clip masks per instance |
| `docs/decisions/DECISION-022.md` | Create | Clipping model and coordinate space |
| `agent/tasks/TASK-025.md` | Create | This document |

## Semantics

- Every layer instance clips its content to the layer's local rectangle
  `(0, 0)` to `(width, height)`.
- The clip region is transformed into device space by the composed
  local-to-parent transform, so scale and rotation clip the transformed
  rectangle, not its axis-aligned bounding box.
- A nested instance's clip region is intersected with the containing layer's
  clip region.
- Root-level direct shapes are not layer content and are bounded only by the
  document canvas, as before.
- Clipping is independent per instance: two instances of one definition do not
  share or leak clip state.
- Fills and strokes are both clipped, including stroke width that extends past
  the bounds.
- The clip edge is antialiased. Clipping does not change opacity, z-index,
  transforms, or cycle behavior.
- Clipping is deterministic: same IR and dimensions produce the same pixels.

## Acceptance Criteria

- [x] Layer content extending past the declared bounds is not painted outside them
- [x] Content inside the bounds still renders normally
- [x] Strokes are clipped at the bounds
- [x] Rotated instances clip the transformed rectangle, not its AABB
- [x] Nested instance content is clipped to the intersection with the parent
- [x] Sibling instances of one definition keep independent clip regions
- [x] Root direct shapes are unaffected
- [x] Clip edges are antialiased
- [x] Existing layer, z-index, nesting, opacity, and cycle tests still pass
- [x] `cargo fmt -- --check` passes
- [x] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [x] `cargo build --workspace` passes
- [x] `cargo test --workspace` passes

## Implementation Steps

1. Thread an optional device-space clip mask through composition rendering.
2. Build a per-instance mask from the layer's transformed local bounds.
3. Intersect child masks with the enclosing mask for nested placements.
4. Pass the mask to fills and strokes.
5. Add renderer tests for bounds, nesting, rotation, strokes, and siblings.
6. Record the model in `docs/decisions/DECISION-022.md`.
7. Run validation commands and update project state.

## Testing Requirements

- Unit tests: clipping of fills, nested intersection, rotated bounds, stroke
  clipping, independent sibling instances.
- Integration tests: covered by existing end-to-end rendering tests.
- Golden images: existing golden remains unchanged.

## Handoff Notes

**Completed Work**:
- Threaded an optional device-space clip mask through composition rendering.
- Built a per-instance mask from the layer's transformed declared bounds and
  intersected nested masks with the enclosing mask.
- Applied the mask to both fills and strokes; clip edges are antialiased.
- Added renderer tests for fill clipping, stroke clipping, rotated transformed
  bounds, nested intersection, and sibling independence.
- Recorded the model in `docs/decisions/DECISION-022.md`.

**Remaining Work**:
- None for TASK-025. Milestone 3 is complete.

**Test Results**:
- `cargo test --workspace`: passes (rupa-renderer 60).
- `cargo clippy --all-targets --all-features -- -D warnings`: passes.
- `cargo build --workspace`: passes.
- `cargo fmt -- --check`: passes.

**Known Issues**:
- A document-sized mask is allocated per layer instance; acceptable for the
  current milestone and simpler than bounding-box masks with arbitrary
  transforms.

**Exact Next Action**:
- Create `agent/tasks/TASK-026.md` and implement image resource loading and
  decoding.

## Specification References

- §6.4: Layer bounds and clipping
- §13: Compositing, clipping, and masks (coordinate spaces and order)
- §16: Independent layer coordinate systems
- §24: Milestone 3 acceptance (content clipped to layer bounds)
