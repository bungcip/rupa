# TASK-006: Rectangle and circle primitives

## Task Header
- **Task ID**: TASK-006
- **Title**: Rectangle and circle primitives
- **Milestone**: M1 — Minimal Vertical Slice
- **Priority**: Critical
- **Estimated Effort**: 4-5 hours
- **Dependencies**: TASK-005

## Description
Implement AST extensions and semantic validation for rectangle and circle primitives per specification §7.1. Define geometry conventions and validation rules.

## Files to Create/Modify

| File | Action | Notes |
|------|--------|-------|
| `crates/rupa-syntax/src/ast.rs` | Modify | Add shape nodes |
| `crates/rupa-compiler/src/shapes.rs` | Create | Shape validation |
| `crates/rupa-ir/src/geometry.rs` | Create | Geometry IR types |
| `crates/rupa-compiler/src/error.rs` | Modify | Add shape error codes |
| `crates/rupa-compiler/src/lib.rs` | Modify | Export modules |

## Acceptance Criteria

**All must pass to mark task complete:**

- [x] Rectangle AST node with position, size, fill (required fields are `Option` so missing properties are semantic errors)
- [x] Circle AST node with center, radius, fill (required fields are `Option`)
- [x] Rectangle position = top-left via `at (x, y)`
- [x] Circle position = center via `center at (x, y)`
- [x] Size required, positive (`size <w> x <h>`)
- [x] Radius required, positive (`radius <r>`)
- [x] Fill required (solid color); color validity is TASK-007
- [x] Negative dimensions/radius → error `S001`/`S002` (task-doc `R200`/`R201`; DECISION-003)
- [x] Missing position/size/radius/fill → `S003`–`S006` (task-doc `R202`–`R205`; DECISION-003)
- [x] Geometry IR separates shape from paint (per §17); single shared `PaintId`
- [x] Unit tests for all validation rules
- [x] `cargo test -p rupa-compiler -p rupa-ir` passes
- [x] `cargo fmt -- --check` passes
- [x] `cargo clippy -- -D warnings` passes

## Implementation Steps

1. Extend AST with `Rectangle` and `Circle` nodes
2. Define `Geometry` enum in IR (`Rectangle`, `Circle`)
3. Define `ShapeNode` with geometry, fill, transform, opacity
4. Implement `ShapeValidator` for semantic checks
5. Define shape error codes (`R200`–`R299`)
6. Write tests

## Testing Requirements

- Valid rectangle → passes
- Valid circle → passes
- Rectangle missing `at` → error
- Rectangle missing `size` → error
- Rectangle missing `fill` → error
- Rectangle negative width/height → error
- Circle missing `center at` → error
- Circle missing `radius` → error
- Circle missing `fill` → error
- Circle negative radius → error
- Position semantics: rectangle top-left, circle center

## Specification References

- §7.1: Basic shapes (rectangle, circle)
- §7.5: Fill and stroke (fill only for M1)
- §17: Graphics IR (geometry/paint separation)
- §2.2: Explicit semantics (position conventions)

## Handoff Notes

**Completed Work**:
- Changed `Rectangle.position`, `Rectangle.size`, `Circle.center`, and
  `Circle.radius` to `Option`; parser retains shapes with missing properties.
- Parser accepts leading `-` in dimension literals and top-level negative
  coordinates, so negative values reach semantic validation.
- `ShapeValidator` now reports:
  - `S001` negative/non-positive rectangle width/height
  - `S002` negative/non-positive circle radius
  - `S003` missing `at` / `center at`
  - `S004` missing rectangle size
  - `S005` missing circle radius
  - `S006` missing fill
  - `S007` non-pixel coordinate/dimension
- Documented rectangle top-left and circle-center semantics.
- IR: removed duplicate `PaintId` from `geometry.rs`; `ShapeNode` uses the
  single paint-module `PaintId`; added geometry/paint separation tests.
- Recorded conventions in `docs/decisions/DECISION-003.md`.
- Added 19 shape tests, 1 parser test, and 3 IR tests.

**Remaining Work**:
- None for TASK-006.

**Test Results**:
- `cargo test --workspace`: passes (rupa-syntax 76, rupa-compiler 44, rupa-ir 3).
- `cargo clippy --all-targets --all-features -- -D warnings`: passes.
- `cargo fmt -- --check`: passes.

**Known Issues**:
- Fill color parsing/validation remains TASK-007; shape validation only checks
  that `fill` is present.
- AST is not yet lowered to IR (TASK-008).

**Exact Next Action**:
- Begin TASK-007: Solid fill paint representation.
