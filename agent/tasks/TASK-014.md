# TASK-014: Line and polyline primitives

## Task Header
- **Task ID**: TASK-014
- **Title**: Line and polyline primitives
- **Milestone**: M2 — Core Vector Geometry
- **Priority**: Critical
- **Estimated Effort**: 6-8 hours
- **Dependencies**: TASK-013

## Description
Implement `line` and `polyline` primitives per specification §7.3. Lines have
no fill and are drawn with a stroke color and width. Polyline is an open path
unless a later path primitive explicitly closes it.

This task adds the minimal stroke paint/style needed to make lines visible.
Full fill/stroke IR separation remains TASK-016 and cap/join/dash properties
remain TASK-018.

## Files to Create/Modify

| File | Action | Notes |
|------|--------|-------|
| `crates/rupa-syntax/src/token.rs` | Modify | Add `stroke` keyword token |
| `crates/rupa-syntax/src/lexer.rs` | Modify | Map `stroke` keyword |
| `crates/rupa-syntax/src/ast.rs` | Modify | Add line/polyline/stroke nodes |
| `crates/rupa-syntax/src/parser.rs` | Modify | Parse line/polyline/stroke/width |
| `crates/rupa-syntax/src/lib.rs` | Modify | Re-export new AST types |
| `crates/rupa-syntax/src/diagnostic.rs` | Modify | Add line/stroke codes |
| `crates/rupa-compiler/src/shapes.rs` | Modify | Validate line/polyline |
| `crates/rupa-compiler/src/validation.rs` | Modify | Dispatch validation/stroke paint |
| `crates/rupa-compiler/src/lower.rs` | Modify | Lower line/polyline/stroke |
| `crates/rupa-ir/src/geometry.rs` | Modify | Add line/polyline/stroke IR |
| `crates/rupa-ir/src/lib.rs` | Modify | Re-export new geometry/stroke |
| `crates/rupa-renderer/src/vector.rs` | Modify | Render open paths with stroke |
| `docs/decisions/DECISION-011.md` | Create | Line/polyline/stroke conventions |

## Syntax

```rupa
line from (50px, 50px) to (300px, 200px)
    stroke black
    width 4px

polyline
    points
        (50px, 300px)
        (150px, 200px)
        (250px, 300px)
        (350px, 200px)
    stroke blue
    width 5px
```

## Semantics

- `line` requires both `from (x, y)` and `to (x, y)` endpoints.
- `polyline` requires a `points` block with at least two points.
- Lines and polylines have no fill; they are rendered with a stroke.
- `stroke <color>` is required; `stroke none` is accepted and draws no stroke.
- `width` is optional and defaults to `1px`; explicit widths must be positive
  pixels.
- Polyline is open: the last point is not connected to the first.
- Endpoints/points use the current coordinate system (document for now) and
  pixels for this milestone.
- Stroke cap/join are defaulted (`butt`/`miter`); configuring them is TASK-018.

## Acceptance Criteria

- [x] `stroke` is a lexer keyword
- [x] Line AST node with optional from/to endpoints, stroke, and width
- [x] Polyline AST node with optional points, stroke, and width
- [x] Parser handles `line from ... to ...` and nested polyline points
- [x] Missing endpoints produce `S003`
- [x] Missing line/polyline `stroke` produces `S010`
- [x] Missing polyline points produces `S008`; fewer than two points `S012`
- [x] Non-positive stroke width produces `S011`; non-pixel width/points `S007`
- [x] IR has `Line` and `Polyline` geometry plus stroke paint/style
- [x] AST lowers lines/polylines with paint-arena stroke resolution
- [x] Renderer draws open stroked paths with correct width/color
- [x] Unit tests for parsing, validation, lowering, IR, and rendering
- [x] End-to-end CLI render tests for line and polyline
- [x] `cargo test --workspace` passes
- [x] `cargo fmt -- --check` passes
- [x] `cargo clippy --all-targets --all-features -- -D warnings` passes

## Specification References

- §7.3: Lines and polylines
- §7.5: Fill and stroke
- §17: Graphics IR
- §2.2: Explicit semantics

## Handoff Notes

**Completed Work**:
- Added `stroke` lexer keyword/token (and lexer mapping for the previously
  unmapped `polyline` keyword).
- Added AST `Stroke`, `Line`, and `Polyline` nodes plus `DocumentChild`
  variants. Missing required properties are retained as `None`.
- Parser handles `line from ... to ...`, polyline `points`, `stroke`, `width`,
  and `stroke none`.
- Added stable codes `S010` (missing stroke), `S011` (invalid stroke width),
  and `S012` (polyline with fewer than two points).
- Validation checks endpoints/points, stroke presence, width positivity/pixel
  units, and stroke paint colors.
- IR: `Line`, `Polyline`, and `Stroke { paint, style }`; `ShapeNode.stroke`
  now carries both paint and style.
- Lowering resolves stroke paint through the paint store and defaults width to
  `1px`.
- Renderer draws open paths with stroked paint/width/opacity/transforms and
  supports fill + stroke independently.
- Recorded conventions in `docs/decisions/DECISION-011.md`.
- Tests: 4 parser, 11 shape-validation, 3 lowering, 3 IR, 3 renderer, and 2
  CLI end-to-end line/polyline tests.

**Remaining Work**:
- None for TASK-014.

**Test Results**:
- `cargo test --workspace`: passes (rupa-syntax 88, rupa-compiler 117,
  rupa-ir 16, rupa-geometry 9, rupa-image 7, rupa-renderer 31,
  rupa-cli 17 CLI + 5 integration).
- `cargo clippy --all-targets --all-features -- -D warnings`: passes.
- `cargo fmt -- --check`: passes.

**Known Issues**:
- Stroke cap/join/dash are defaults only; syntax arrives in TASK-018.
- Full fill/stroke IR separation is partially advanced here and remains
  TASK-016.
- `stroke none` is valid and produces no visible path.

**Exact Next Action**:
- Create `agent/tasks/TASK-015.md` from the backlog and implement path
  geometry (start, line, curve, quad, close).
