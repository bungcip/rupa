# TASK-064: Reduce lowering duplication

**Milestone**: Maintenance (post-M9)
**Dependencies**: TASK-060
**Status**: Complete

## Goal

Simplify repeated required-property extraction and points lowering in
`rupa-compiler` without changing diagnostics, IR, rendering, or public behavior.

## Scope

- Add a generic `required` helper for lowering required AST properties with a
  stable diagnostic code, shape/property name, span, and help text.
- Add a shared `lower_points` helper for polygon/polyline point validation and
  conversion.
- Route rectangle, circle, ellipse, square, triangle, line, polygon, and
  polyline lowering through the helpers.
- Keep geometry-specific construction and messages explicit.

## Acceptance Criteria

- [x] Repeated missing position/size/radius/points blocks are removed
- [x] Polygon and polyline share point validation/conversion
- [x] Diagnostics and messages remain unchanged
- [x] IR and golden output remain unchanged
- [x] Full workspace validation passes

## Verification

- `cargo fmt -- --check` — pass
- `cargo clippy --all-targets --all-features -- -D warnings` — pass
- `cargo build --workspace` — pass
- `cargo test --workspace` — pass
- Golden image tests remain pixel-identical
