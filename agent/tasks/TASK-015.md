# TASK-015: Path geometry

## Task Header
- **Task ID**: TASK-015
- **Title**: Path geometry (cubic/quadratic Bézier)
- **Milestone**: M2 — Core Vector Geometry
- **Priority**: Critical
- **Estimated Effort**: 8-10 hours
- **Dependencies**: TASK-014

## Description
Implement the `path` primitive with `start at`, `line to`, `curve to`
(cubic Bézier), `quad to` (quadratic Bézier), and `close` commands per
specification §7.4. Paths may have fill and/or stroke.

## Files to Create/Modify

| File | Action | Notes |
|------|--------|-------|
| `crates/rupa-syntax/src/ast.rs` | Modify | Add path + command nodes |
| `crates/rupa-syntax/src/parser.rs` | Modify | Parse path commands/control blocks |
| `crates/rupa-syntax/src/lib.rs` | Modify | Re-export path types |
| `crates/rupa-syntax/src/diagnostic.rs` | Modify | Add missing-start code |
| `crates/rupa-compiler/src/shapes.rs` | Modify | Validate path commands |
| `crates/rupa-compiler/src/validation.rs` | Modify | Dispatch validation |
| `crates/rupa-compiler/src/lower.rs` | Modify | Lower path commands |
| `crates/rupa-ir/src/geometry.rs` | Modify | Add path geometry |
| `crates/rupa-ir/src/lib.rs` | Modify | Re-export path geometry |
| `crates/rupa-renderer/src/vector.rs` | Modify | Build Bézier paths |
| `docs/decisions/DECISION-012.md` | Create | Path conventions |

## Syntax

```rupa
path
    start at (100px, 300px)
    line to (200px, 100px)
    curve to (400px, 100px)
        control (250px, 50px) and (350px, 50px)
    line to (500px, 300px)
    close
    fill purple
    stroke black
    width 3px
```

Quadratic segment control syntax (not shown in the specification example):

```rupa
path
    start at (10px, 10px)
    quad to (100px, 10px)
        control (55px, 80px)
    close
    fill blue
```

## Semantics

- A path must have at least one command and begin with `start at`.
- `start at` begins a subpath; additional `start at` commands begin new
  subpaths.
- `line to` adds a straight segment.
- `curve to` is a cubic Bézier with two control points supplied on an
  indented `control (c1) and (c2)` line.
- `quad to` is a quadratic Bézier with one control point supplied on an
  indented `control (c)` line.
- `close` closes the current subpath.
- Fill and stroke are both optional; a path may have fill only, stroke only,
  both, or neither (invisible).
- Width is optional and defaults to `1px` when a stroke is present; explicit
  widths must be positive pixels.
- Coordinates use the current coordinate system and pixels for this milestone.
- An unclosed path that is filled is implicitly closed by the renderer's fill
  rule; explicit `close` is still preferred for stroked outlines.

## Acceptance Criteria

- [x] Path AST node and command enum (`Start`, `LineTo`, `CurveTo`, `QuadTo`, `Close`)
- [x] Parser handles all five commands and nested control blocks
- [x] Empty path or a path not starting with `start at` produces `S013`
- [x] Non-pixel coordinates produce `S007`
- [x] Invalid stroke width produces `S011`
- [x] IR has path command geometry with cubic/quadratic segments
- [x] AST lowers paths with fill and/or stroke
- [x] Renderer builds and draws line, cubic, quadratic, and close commands
- [x] Unit tests for parsing, validation, lowering, IR, and rendering
- [x] End-to-end CLI render test for a filled+stroked path
- [x] `cargo test --workspace` passes
- [x] `cargo fmt -- --check` passes
- [x] `cargo clippy --all-targets --all-features -- -D warnings` passes

## Specification References

- §7.4: Paths and curves
- §7.5: Fill and stroke
- §17: Graphics IR
- §2.2: Explicit semantics

## Handoff Notes

**Completed Work**:
- Added AST `Path` and `PathCommand` (`Start`, `LineTo`, `CurveTo`, `QuadTo`,
  `Close`) plus `DocumentChild::Path`.
- Parser handles all commands, cubic control pairs, quadratic single controls,
  and optional fill/stroke/width properties.
- Added stable code `S013` for missing/incorrect path start; reused `S007` and
  `S011` for coordinate/width validation.
- IR: `Path { commands }` with `MoveTo`, `LineTo`, `CubicTo`, `QuadTo`, and
  `Close`.
- Lowering maps all commands and resolves optional fill/stroke paint.
- Renderer builds tiny-skia paths for lines, cubic/quadratic Béziers, and
  close; fill and stroke render independently.
- Recorded conventions in `docs/decisions/DECISION-012.md`, including the
  chosen quadratic control syntax.
- Tests: 3 parser, 6 validation, 3 lowering, 1 IR, 4 renderer, and 2 CLI
  end-to-end path tests.

**Remaining Work**:
- None for TASK-015.

**Test Results**:
- `cargo test --workspace`: passes (rupa-syntax 91, rupa-compiler 125,
  rupa-ir 17, rupa-geometry 9, rupa-image 7, rupa-renderer 35,
  rupa-cli 19 CLI + 5 integration).
- `cargo clippy --all-targets --all-features -- -D warnings`: passes.
- `cargo fmt -- --check`: passes.

**Known Issues**:
- Quadratic control syntax is an implementation-defined interpretation
  (DECISION-012).
- Explicit `close` is recommended for stroked outlines; fill implicitly closes
  open paths.
- Stroke cap/join/dash remain defaults until TASK-018.

**Exact Next Action**:
- Create `agent/tasks/TASK-016.md` from the backlog and implement fill/stroke
  IR separation.
