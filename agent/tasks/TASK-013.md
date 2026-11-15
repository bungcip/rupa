# TASK-013: Polygon with points list

## Task Header
- **Task ID**: TASK-013
- **Title**: Polygon with points list
- **Milestone**: M2 — Core Vector Geometry
- **Priority**: Critical
- **Estimated Effort**: 5-7 hours
- **Dependencies**: TASK-012

## Description
Implement the `polygon` primitive with a nested `points` list per
specification §7.2. A filled polygon requires at least three points.

## Files to Create/Modify

| File | Action | Notes |
|------|--------|-------|
| `crates/rupa-syntax/src/ast.rs` | Modify | Add polygon node |
| `crates/rupa-syntax/src/parser.rs` | Modify | Parse `points` block |
| `crates/rupa-syntax/src/lib.rs` | Modify | Re-export polygon |
| `crates/rupa-syntax/src/diagnostic.rs` | Modify | Add missing/too-few points codes |
| `crates/rupa-compiler/src/shapes.rs` | Modify | Validate polygon |
| `crates/rupa-compiler/src/validation.rs` | Modify | Dispatch validation |
| `crates/rupa-compiler/src/lower.rs` | Modify | Lower polygon to IR |
| `crates/rupa-ir/src/geometry.rs` | Modify | Add polygon geometry |
| `crates/rupa-ir/src/lib.rs` | Modify | Re-export polygon geometry |
| `crates/rupa-renderer/src/vector.rs` | Modify | Render closed polygon |
| `docs/decisions/DECISION-010.md` | Create | Polygon conventions |

## Syntax

```rupa
polygon
    points
        (100px, 100px)
        (200px, 50px)
        (300px, 100px)
        (250px, 200px)
        (150px, 200px)
    fill purple
```

## Semantics

- `points` is required and must contain at least three positions.
- A polygon is always closed; the renderer connects the last point back to the
  first.
- Points are in the current coordinate system (the document for now), expressed
  in pixels for this milestone.
- Fill is required for this milestone; stroke support arrives in TASK-016.
- `fill none` remains valid and lowers to no paint.
- Self-intersection behavior follows the renderer's nonzero winding fill rule;
  no validation is added for self-intersection in this task.

## Acceptance Criteria

- [x] Polygon AST node with optional points list and optional fill
- [x] Parser handles the nested `points` block and positions
- [x] Missing `points` produces `S008`
- [x] Fewer than three points produces `S009`
- [x] Missing fill produces `S006`
- [x] Non-pixel point coordinates produce `S007`
- [x] IR `Polygon` geometry stores validated points
- [x] AST lowers polygons in source order with paint arena resolution
- [x] Polygon renders as a closed filled path
- [x] Unit tests for parsing, validation, lowering, IR, and rendering
- [x] End-to-end CLI render test for a polygon
- [x] `cargo test --workspace` passes
- [x] `cargo fmt -- --check` passes
- [x] `cargo clippy --all-targets --all-features -- -D warnings` passes

## Specification References

- §7.2: Polygon
- §7.5: Fill
- §17: Graphics IR
- §2.2: Explicit semantics

## Handoff Notes

**Completed Work**:
- Added `Polygon` AST node and `DocumentChild::Polygon`; `points` is
  `Option<Vec<Position>>` so a missing block is a semantic error.
- Parser handles the nested `points` indentation block and `fill` in either
  order.
- Added stable codes `S008` (missing points) and `S009` (fewer than three
  points); reused `S006`/`S007`.
- Validation checks points presence/count, pixel-unit coordinates, and fill.
- IR: `Point { x, y }` and `Polygon { points }`; lowering preserves point
  order and resolves fill through the paint store.
- Renderer: closed path with nonzero winding, first point connected last;
  transform origin at the bounding-box center.
- Recorded conventions in `docs/decisions/DECISION-010.md`.
- Tests: 3 parser, 6 validation, 3 lowering, 1 IR, 2 renderer, and 2 CLI
  end-to-end tests for polygons.

**Remaining Work**:
- None for TASK-013.

**Test Results**:
- `cargo test --workspace`: passes (rupa-syntax 84, rupa-compiler 103,
  rupa-ir 13, rupa-geometry 9, rupa-image 7, rupa-renderer 28,
  rupa-cli 15 CLI + 5 integration).
- `cargo clippy --all-targets --all-features -- -D warnings`: passes.
- `cargo fmt -- --check`: passes.

**Known Issues**:
- Self-intersection is not validated; nonzero winding is used.
- Polygon points are document coordinates until layer-local coordinates arrive
  in TASK-021.
- Stroke support is TASK-016; fill is still required.

**Exact Next Action**:
- Create `agent/tasks/TASK-014.md` from the backlog and implement line and
  polyline primitives.
