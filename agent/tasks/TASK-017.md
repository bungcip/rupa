# TASK-017: Transform matrix composition

## Task Header
- **Task ID**: TASK-017
- **Title**: Transform matrix composition
- **Milestone**: M2 — Core Vector Geometry
- **Priority**: Critical
- **Estimated Effort**: 6-8 hours
- **Dependencies**: TASK-016

## Description
Wire shape transform properties (`rotate`, `scale`, `opacity`) from source
syntax into the existing affine transform matrix and renderer. The renderer
already applies a shape matrix around the primitive's geometric center; this
task adds parsing, AST representation, validation, and lowering.

## Files to Create/Modify

| File | Action | Notes |
|------|--------|-------|
| `crates/rupa-syntax/src/ast.rs` | Modify | Add transform properties to all shapes |
| `crates/rupa-syntax/src/parser.rs` | Modify | Parse rotate/scale/opacity |
| `crates/rupa-syntax/src/diagnostic.rs` | Modify | Add invalid-transform code |
| `crates/rupa-compiler/src/shapes.rs` | Modify | Validate transform values |
| `crates/rupa-compiler/src/lower.rs` | Modify | Compose matrix and opacity |
| `crates/rupa-renderer/src/vector.rs` | Modify | Verify transformed rendering |
| `crates/rupa-cli/tests/cli.rs` | Modify | End-to-end transformed render |
| `docs/decisions/DECISION-014.md` | Create | Transform composition order |

## Syntax

```rupa
rectangle at (100px, 100px)
    size 200px x 100px
    fill blue
    rotate 30deg
    scale 0.8
    opacity 0.8
```

- `rotate <angle>deg` — rotation around the primitive's geometric center.
- `scale <factor>` — uniform scale around the center.
- `scale <sx> x <sy>` — non-uniform scale around the center.
- `opacity <value>` — dimensionless value in `[0.0, 1.0]`.

## Semantics

- Transform properties apply to all primitives (closed shapes, lines,
  polylines, paths).
- Composition order is `rotate * scale` (scale first, then rotation), applied
  around the primitive center by the renderer.
- Position/`at` is geometry, not part of the transform matrix.
- Rotation requires `deg`; scale and opacity are dimensionless.
- Scale factors must be finite and strictly positive.
- Opacity must be finite and within `0.0..=1.0`.
- Invalid values produce `S014`; non-degree rotation units also `S014`.
- Opacity is applied to both fill and stroke.

## Acceptance Criteria

- [x] All shape AST nodes carry optional transform properties
- [x] Parser accepts `rotate`, `scale` (uniform/non-uniform), and `opacity`
- [x] Validation rejects invalid rotation units, scale, and opacity (`S014`)
- [x] Lowering composes `rotate * scale` matrix and carries opacity
- [x] Renderer applies the composed transform around shape centers
- [x] Opacity applies to fill and stroke
- [x] Unit tests for parser, validation, lowering, and renderer
- [x] End-to-end CLI test with rotate/scale/opacity
- [x] `cargo test --workspace` passes
- [x] `cargo fmt -- --check` passes
- [x] `cargo clippy --all-targets --all-features -- -D warnings` passes

## Specification References

- §11: Transformations and opacity
- §2.3: Predictable defaults
- §2.2: Explicit semantics

## Handoff Notes

**Completed Work**:
- Added AST `Transform { rotate, scale, opacity, span }` and `Scale`; all nine
  primitive nodes carry an optional transform.
- Parser accepts `rotate <deg>`, uniform `scale <factor>`, non-uniform
  `scale <sx> x <sy>`, and `opacity <value>` on every primitive.
- Added stable code `S014` for invalid rotation units, non-positive/non-finite
  scale, and out-of-range opacity.
- Lowering composes `rotate * scale` and carries opacity; missing properties
  default to identity/1.0.
- Renderer already pivots around primitive centers; added tests proving
  opacity applies to fill and stroke and transforms compose correctly.
- Recorded composition order and syntax in
  `docs/decisions/DECISION-014.md`.
- Tests: 2 parser, 5 validation, 2 lowering, 1 renderer, and 1 CLI end-to-end
  transform test.

**Remaining Work**:
- None for TASK-017.

**Test Results**:
- `cargo test --workspace`: passes (rupa-syntax 94, rupa-compiler 139,
  rupa-ir 17, rupa-geometry 9, rupa-image 7, rupa-renderer 38,
  rupa-cli 21 CLI + 5 integration).
- `cargo clippy --all-targets --all-features -- -D warnings`: passes.
- `cargo fmt -- --check`: passes.

**Known Issues**:
- Negative scale (mirroring) is rejected by `S014`; not specified.
- Transform order is fixed to rotate-then-scale application (`rotate * scale`).
- Stroke cap/join/dash remain TASK-018.

**Exact Next Action**:
- Create `agent/tasks/TASK-018.md` from the backlog and implement stroke style
  properties (width, cap, join).
