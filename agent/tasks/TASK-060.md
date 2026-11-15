# TASK-060: Unify shape validation and lowering

**Milestone**: 9 — Document Model & Root Layer Refactor
**Dependencies**: TASK-058, TASK-059
**Status**: Complete

## Goal

Reduce duplicated shape validation and lowering for fill, stroke, transform,
opacity, paint, and common diagnostics while preserving IR and rendered output.

## Scope

- Extract shared validation for common shape fields and paint/stroke/transform
  checks.
- Extract shared lowering of common fields into IR shape nodes.
- Keep geometry-specific validation explicit (radius, size, points, path
  commands, text content, picture resources).
- Unify diagnostic construction so equivalent mistakes produce the same code and
  help across primitives.
- Preserve source order, paint IDs, transform composition, and layer ownership.

## Files to Modify

- `crates/rupa-compiler/src/shapes.rs`
- `crates/rupa-compiler/src/lower.rs`
- `crates/rupa-compiler/src/validation.rs`
- `crates/rupa-compiler/src/paint.rs` (only shared helpers as needed)

## Acceptance Criteria

- [x] Common shape validation is implemented once, not per primitive
- [x] Common shape lowering is implemented once, not per primitive
- [x] IR output and goldens are unchanged
- [x] Diagnostics retain stable codes and helpful messages
- [x] `cargo test -p rupa-compiler -p rupa-renderer` passes

## Testing

- Validation tests for every primitive
- Lowering tests for fill/stroke/transform combinations
- Golden image regression suite

## Implementation Notes

- `ShapeValidator` now shares `validate_required_position`; rectangle/ellipse/
  square/triangle/picture reuse `validate_size_property`.
- `validation.rs` validates fill/stroke paint through shared
  `validate_fill_color` and `validate_paint_fields` helpers.
- Lowering uses `ShapeAppearance` + `lower_shape_appearance`, which handles
  fill, optional/required stroke, transform, and opacity once. Each primitive
  now supplies only its geometry plus whether a stroke is required.
- `lower_fill` accepts `Option<&Fill>`, removing per-shape conditional glue.
- Geometry-specific missing-property diagnostics remain explicit.
- Added table tests for invalid fill and stroke colors across every painted/
  stroked primitive.

## Verification

- `cargo fmt -- --check` — pass
- `cargo clippy --all-targets --all-features -- -D warnings` — pass
- `cargo build --workspace` — pass
- `cargo test --workspace` — pass
- Golden image tests remain pixel-identical
