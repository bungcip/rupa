# TASK-008: Basic renderer with tiny-skia

## Task Header
- **Task ID**: TASK-008
- **Title**: Basic renderer with tiny-skia
- **Milestone**: M1 — Minimal Vertical Slice
- **Priority**: Critical
- **Estimated Effort**: 6-8 hours
- **Dependencies**: TASK-007

## Description
Implement CPU renderer using `tiny-skia` for vector rasterization. Convert IR geometry to `tiny-skia` paths, apply fills, handle transforms, and composite with premultiplied alpha.

## Files to Create/Modify

| File | Action | Notes |
|------|--------|-------|
| `crates/rupa-renderer/src/lib.rs` | Create | Renderer entry point |
| `crates/rupa-renderer/src/vector.rs` | Create | Vector rendering |
| `crates/rupa-geometry/src/transform.rs` | Create | Transform matrix |
| `crates/rupa-geometry/src/lib.rs` | Modify | Export transform |
| `crates/rupa-renderer/Cargo.toml` | Modify | Add tiny-skia dependency |
| `crates/rupa-geometry/Cargo.toml` | Modify | Add geometry deps |

## Acceptance Criteria

**All must pass to mark task complete:**

- [x] Rectangle renders at correct position/size with fill
- [x] Circle renders at correct center/radius with fill
- [x] Colors match expected sRGB values
- [x] Transform matrix: translate, rotate, scale, compose
- [x] Transform origin: rectangle center, circle center
- [x] Premultiplied alpha compositing
- [x] Opacity applied correctly (0.0–1.0)
- [x] Renderer takes IR document and produces pixel buffer
- [x] Unit tests for geometry conversion
- [x] Unit tests for transform composition
- [x] `cargo test -p rupa-renderer -p rupa-geometry` passes
- [x] `cargo fmt -- --check` passes
- [x] `cargo clippy -- -D warnings` passes

## Implementation Steps

1. Add `tiny-skia` to `rupa-renderer` dependencies
2. Add `kurbo` or `glam` to `rupa-geometry` for transforms
3. Define `Transform` (3x2 matrix or similar)
4. Implement transform composition (multiply)
5. Implement `VectorRenderer`:
   - Create `tiny_skia::Pixmap` with document dimensions
   - Convert IR `Rectangle` → `tiny_skia::PathBuilder` rect
   - Convert IR `Circle` → `PathBuilder` circle
   - Apply `Paint` → `tiny_skia::Paint` with shader
   - Apply `Transform` → `tiny_skia::Transform`
   - Draw with `Pixmap::fill_path` or `stroke_path`
6. Handle premultiplied alpha conversion
7. Write tests

## Testing Requirements

- Rectangle at (10,10) size 80x40 fill red → correct pixels
- Circle center (100,100) radius 30 fill blue → correct pixels
- Transform: translate → position shifts
- Transform: rotate 90deg → correct rotation
- Transform: scale 2.0 → doubles size
- Nested transforms compose correctly
- Opacity 0.5 → blends with background
- Transparent background → alpha channel correct

## Specification References

- §18: Rendering architecture (tiny-skia)
- §18.1: Color and alpha (premultiplied)
- §18.2: Determinism
- §11: Transformations and opacity
- §17: Graphics IR (geometry/paint separation)

## Handoff Notes

**Completed Work**:
- Added `tiny-skia = "0.12"` to `rupa-renderer`.
- Corrected and documented transform matrix semantics: column-vector 3x2,
  `multiply(self, other)` applies `other` first, positive rotation is
  clockwise in Y-down coordinates. Added `transform_point`, `rotate_about`,
  `scale_about`, and composition tests.
- Implemented `VectorRenderer` / `render_document`:
  - `RenderDocument` (dimensions, optional background, `PaintStore`, shapes)
    → `RenderOutput` (premultiplied RGBA8 buffer)
  - rectangle/circle path conversion via `tiny-skia`
  - solid paint resolution, opacity multiplication, source-order drawing
  - transforms pivoted on rectangle/circle center
  - errors for invalid dimensions/geometry and missing paint IDs
- Recorded conventions in `docs/decisions/DECISION-005.md`.
- Added 18 renderer tests and 9 geometry transform tests.

**Remaining Work**:
- None for TASK-008.

**Test Results**:
- `cargo test --workspace`: passes (rupa-syntax 77, rupa-compiler 64,
  rupa-ir 8, rupa-geometry 9, rupa-renderer 18).
- `cargo clippy --all-targets --all-features -- -D warnings`: passes.
- `cargo fmt -- --check`: passes.

**Known Issues**:
- PNG encoding (TASK-009) must unpremultiply or explicitly define the output
  color contract.
- AST→`RenderDocument` lowering is still needed for the CLI (TASK-010).
- Opacity is clamped in the renderer; compiler validation of out-of-range
  opacity is future work.

**Exact Next Action**:
- Begin TASK-009: PNG encoding.
