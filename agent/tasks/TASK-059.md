# TASK-059: Unify primitive parsing

**Milestone**: 9 — Document Model & Root Layer Refactor
**Dependencies**: TASK-057
**Status**: Complete

## Goal

Reduce duplicated primitive parsing while preserving every primitive's behavior
and diagnostics.

## Scope

- Replace repeated per-shape `else if` property loops with shared helpers or
  property descriptors for:
  - position and size
  - fill and stroke
  - width, cap, join, and other stroke style
  - rotate, scale, and opacity
  - unexpected-property diagnostics and suggestions
- Keep typed shape fields where they encode real requirements (radius, points,
  path commands, text content, picture path, fit mode).
- Ensure rectangle, circle, ellipse, square, triangle, polygon, line, polyline,
  path, picture, and text share common parsing machinery.
- Do not change the source grammar or AST shape semantics.

## Files to Modify

- `crates/rupa-syntax/src/parser.rs`
- Possibly `crates/rupa-syntax/src/ast.rs` for shared field structs
- Parser regression tests

## Acceptance Criteria

- [x] No per-shape copy-paste loops remain for common properties
- [x] Every primitive retains its existing required/optional property behavior
- [x] Diagnostics, codes, suggestions, and spans are unchanged or improved
- [x] Parser test coverage covers every primitive before and after refactor
- [x] `cargo test -p rupa-syntax` passes

## Testing

- Existing parser tests must pass unchanged except for intentional improvements
- Add table-driven tests for common properties across all primitives

## Implementation Notes

- Added parser-local `ShapeProperties` and
  `parse_common_shape_property(allow_fill, allow_stroke)`.
- Rectangle, circle, ellipse, square, triangle, polygon, line, polyline, path,
  picture, and text now share fill/stroke/width/cap/join/rotate/scale/opacity
  parsing.
- Shape-specific properties remain explicit (size, radius, points, path
  commands, fit, text fields, etc.).
- Added a table-style parser test covering common properties across all
  primitives and a negative test proving line still rejects `fill`.
- Parser source size decreased while preserving 100% of the existing parser test
  behavior; new tests bring the total to 117.

## Verification

- `cargo fmt -- --check` — pass
- `cargo clippy --all-targets --all-features -- -D warnings` — pass
- `cargo build --workspace` — pass
- `cargo test --workspace` — pass
