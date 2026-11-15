# TASK-012: Ellipse, square, and triangle primitives

## Task Header
- **Task ID**: TASK-012
- **Title**: Ellipse, square, and triangle primitives
- **Milestone**: M2 — Core Vector Geometry
- **Priority**: Critical
- **Estimated Effort**: 6-8 hours
- **Dependencies**: TASK-011

## Description
Extend the language, semantic validation, IR lowering, and renderer with the
`ellipse`, `square`, and `triangle` primitives per specification §7.1. This is
the first task of Milestone 2.

## Files to Create/Modify

| File | Action | Notes |
|------|--------|-------|
| `crates/rupa-syntax/src/ast.rs` | Modify | Add ellipse/square/triangle nodes |
| `crates/rupa-syntax/src/parser.rs` | Modify | Parse the three primitives |
| `crates/rupa-syntax/src/lib.rs` | Modify | Re-export new AST types |
| `crates/rupa-compiler/src/shapes.rs` | Modify | Validate new primitives |
| `crates/rupa-compiler/src/validation.rs` | Modify | Dispatch validation |
| `crates/rupa-compiler/src/lower.rs` | Modify | Lower new primitives to IR |
| `crates/rupa-ir/src/geometry.rs` | Modify | Add ellipse/triangle IR geometry |
| `crates/rupa-ir/src/lib.rs` | Modify | Re-export new geometry |
| `crates/rupa-renderer/src/vector.rs` | Modify | Rasterize new geometry |
| `docs/decisions/DECISION-009.md` | Create | Geometry conventions |

## Syntax

```rupa
ellipse center at (200px, 350px)
    size 180px x 100px
    fill orange

square at (300px, 50px)
    size 100px
    fill red

triangle at (400px, 300px)
    size 160px x 140px
    fill purple
```

## Geometry Conventions

- `ellipse center at` is the center; `size <w> x <h>` is full width/height.
- `square at` is the top-left corner; `size <side>` is a single dimension.
- `triangle at` is the bounding-box top-left; `size <w> x <h>` is the bounding
  box. The triangle is isosceles with its apex at top-center and base along the
  bottom edge (DECISION-009).
- Negative/zero dimensions are invalid; shape dimensions use pixels in M2.
- Fill is required for these primitives (stroke support is TASK-016).

## Acceptance Criteria

- [x] `ellipse`, `square`, and `triangle` parse into AST nodes
- [x] Each node retains missing required properties for semantic errors
- [x] Square accepts a single `size <side>` dimension
- [x] Position conventions match the specification
- [x] Missing position/size/fill produce `S003`/`S004`/`S006`
- [x] Negative/zero dimensions produce `S001`
- [x] Non-pixel shape dimensions produce `S007`
- [x] AST lowers to IR geometry (`Ellipse`, `Rectangle` for square, `Triangle`)
- [x] All three primitives render with solid fills at expected pixels
- [x] Unit tests for parsing, validation, lowering, and rendering
- [x] `cargo test --workspace` passes
- [x] `cargo fmt -- --check` passes
- [x] `cargo clippy --all-targets --all-features -- -D warnings` passes

## Specification References

- §7.1: Basic shapes
- §7.5: Fill (fill only for this task)
- §2.2: Explicit position semantics
- §17: Graphics IR

## Handoff Notes

**Completed Work**:
- Added AST nodes `Ellipse`, `Square`, `Triangle` and `DocumentChild` variants.
- Parser support for `ellipse center at`, `square at`, `triangle at`; square
  accepts single `size <side>`, the others use `size <w> x <h>`.
- Semantic validation with existing stable codes (`S003`, `S004`, `S006`,
  `S001`, `S007`) plus shape-specific help text.
- IR geometry: `Ellipse { cx, cy, width, height }`, `Triangle { x, y, width,
  height }`; squares lower to rectangle IR.
- AST→IR lowering for all three primitives and source-order preservation.
- Renderer: ellipse via `tiny-skia` oval; triangle apex-up path; transform
  origins at ellipse center and triangle bounding-box center.
- Recorded conventions in `docs/decisions/DECISION-009.md`.
- Tests: 4 parser, 16 shape-validation, 4 lowering, 3 IR, 3 renderer, and
  1 CLI end-to-end test for the new primitives.

**Remaining Work**:
- None for TASK-012.

**Test Results**:
- `cargo test --workspace`: passes (rupa-syntax 81, rupa-compiler 94,
  rupa-ir 12, rupa-geometry 9, rupa-image 7, rupa-renderer 26,
  rupa-cli 13 CLI + 5 integration).
- `cargo clippy --all-targets --all-features -- -D warnings`: passes.
- `cargo fmt -- --check`: passes.

**Known Issues**:
- Triangle orientation is an explicit interpretation (apex-up isosceles) and
  is recorded in DECISION-009.
- Stroke support is not part of this task (TASK-016).

**Exact Next Action**:
- Create `agent/tasks/TASK-013.md` from the backlog and implement polygon
  with points list.
