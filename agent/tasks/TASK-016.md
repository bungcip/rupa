# TASK-016: Fill and stroke separation

## Task Header
- **Task ID**: TASK-016
- **Title**: Fill and stroke IR separation
- **Milestone**: M2 — Core Vector Geometry
- **Priority**: Critical
- **Estimated Effort**: 6-8 hours
- **Dependencies**: TASK-015

## Description
Complete the fill/stroke behavior required by specification §7.5 and §17:
geometry and appearance are separate, and any shape may have fill only, stroke
only, both, or neither. Closed shapes gain `stroke`/`width` properties, and the
M1 rule requiring fill on closed shapes is removed.

The IR already stores `ShapeNode.fill: Option<PaintId>` and
`ShapeNode.stroke: Option<Stroke>` independently. This task extends the AST,
parser, validation, lowering, and renderer behavior to use that separation for
all primitives.

## Files to Create/Modify

| File | Action | Notes |
|------|--------|-------|
| `crates/rupa-syntax/src/ast.rs` | Modify | Add stroke/width to closed shape nodes |
| `crates/rupa-syntax/src/parser.rs` | Modify | Parse stroke/width on closed shapes |
| `crates/rupa-compiler/src/shapes.rs` | Modify | Fill optional; validate optional strokes |
| `crates/rupa-compiler/src/validation.rs` | Modify | Validate stroke colors on closed shapes |
| `crates/rupa-compiler/src/lower.rs` | Modify | Lower stroke/width for closed shapes |
| `crates/rupa-renderer/src/vector.rs` | Modify | Verify fill+stroke combinations |
| `crates/rupa-cli/tests/cli.rs` | Modify | End-to-end fill+stroke test |
| `docs/decisions/DECISION-013.md` | Create | Appearance semantics |

## Semantics

- Closed primitives (`rectangle`, `circle`, `ellipse`, `square`, `triangle`,
  `polygon`) accept optional `fill`, optional `stroke`, and optional `width`.
- A shape may have fill only, stroke only, both, or neither. Neither is valid
  but invisible.
- `fill none` and `stroke none` explicitly disable that appearance component.
- Width is optional and defaults to `1px` when a stroke is present; explicit
  widths must be positive pixels.
- Line and polyline remain stroke-only; path already supports both.
- Fill and stroke are rendered independently, fill first, then stroke.
- Geometry remains independent of appearance; paint references live in the
  shape node, not in geometry.

## Acceptance Criteria

- [x] Closed shape AST nodes carry optional `stroke` and `width`
- [x] Parser accepts `stroke`/`width` on all closed shapes
- [x] Missing fill is no longer an error for closed shapes (`S006` removed from use)
- [x] Fill-only, stroke-only, both, and neither are all valid
- [x] Stroke paint colors are validated with existing `T001`/`T002`
- [x] Invalid stroke width produces `S011`; non-pixel width `S007`
- [x] Lowering resolves independent fill and stroke paint IDs for closed shapes
- [x] Renderer draws fill and stroke independently for closed shapes
- [x] IR still stores geometry separately from fill/stroke appearance
- [x] Unit tests for parser, validation, lowering, and rendering combinations
- [x] End-to-end CLI test with a filled+stroked rectangle
- [x] `cargo test --workspace` passes
- [x] `cargo fmt -- --check` passes
- [x] `cargo clippy --all-targets --all-features -- -D warnings` passes

## Specification References

- §7.5: Fill and stroke
- §17: Graphics IR (geometry/paint separation)
- §2.2: Explicit semantics

## Handoff Notes

**Completed Work**:
- Added optional `stroke` and `width` fields to all six closed shape AST nodes
  and parser property loops.
- Made fill optional for closed shapes; removed all active `S006` emissions
  while retaining the stable code in the registry.
- Added optional stroke validation and wired stroke paint color checks for all
  closed shapes.
- Lowering now resolves independent fill and stroke paint IDs for closed
  shapes, with default stroke width `1px`.
- Renderer verified for fill+stroke, stroke-only, and invisible (neither)
  closed shapes.
- Recorded appearance semantics in `docs/decisions/DECISION-013.md`.
- Tests: 1 parser, 8 validation, 3 lowering, 2 renderer, and 1 CLI end-to-end
  appearance combination test.

**Remaining Work**:
- None for TASK-016.

**Test Results**:
- `cargo test --workspace`: passes (rupa-syntax 92, rupa-compiler 132,
  rupa-ir 17, rupa-geometry 9, rupa-image 7, rupa-renderer 37,
  rupa-cli 20 CLI + 5 integration).
- `cargo clippy --all-targets --all-features -- -D warnings`: passes.
- `cargo fmt -- --check`: passes.

**Known Issues**:
- `S006` is a reserved but no-longer-emitted stable code.
- Stroke cap/join/dash remain defaults until TASK-018.
- Fill/stroke remain independent paint references in `ShapeNode`; no geometry
  embeds appearance data.

**Exact Next Action**:
- Create `agent/tasks/TASK-017.md` from the backlog and implement transform
  matrix composition.
