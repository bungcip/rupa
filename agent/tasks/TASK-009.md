# TASK-009: PNG encoding

## Task Header
- **Task ID**: TASK-009
- **Title**: PNG encoding
- **Milestone**: M1 — Minimal Vertical Slice
- **Priority**: Critical
- **Estimated Effort**: 2-3 hours
- **Dependencies**: TASK-008

## Description
Encode rendered pixel buffer to PNG file using the `png` crate. Output dimensions must match document dimensions. Handle sRGB color space conversion.

## Files to Create/Modify

| File | Action | Notes |
|------|--------|-------|
| `crates/rupa-renderer/src/png.rs` | Create | PNG encoding |
| `crates/rupa-image/src/encode.rs` | Create | Image encoding utilities |
| `crates/rupa-image/src/lib.rs` | Modify | Export encode module |
| `crates/rupa-renderer/Cargo.toml` | Modify | Add png dependency |
| `crates/rupa-image/Cargo.toml` | Modify | Add png dependency |

## Acceptance Criteria

**All must pass to mark task complete:**

- [x] PNG file written successfully to specified path
- [x] Output dimensions match document `size` exactly
- [x] Pixel values match renderer output (opaque RGB identical; alpha unpremultiplied)
- [x] Transparent backgrounds produce alpha channel
- [x] Opaque backgrounds produce RGBA with alpha=255
- [x] sRGB color space used for output (sRGB chunk, perceptual intent)
- [x] Integration test with known output (200×100 minimal IR document)
- [x] `cargo test -p rupa-renderer -p rupa-image` passes
- [x] `cargo fmt -- --check` passes
- [x] `cargo clippy -- -D warnings` passes

## Implementation Steps

1. Add `png` crate to `rupa-renderer` and/or `rupa-image`
2. Implement `encode_png(pixels, width, height, path)` function
3. Convert renderer pixel buffer to PNG format
4. Handle premultiplied → straight alpha conversion if needed
5. Write integration test rendering minimal example to PNG
6. Verify output with image viewer or pixel comparison

## Testing Requirements

- Render minimal example → PNG file exists
- PNG dimensions = document size (200x100)
- Pixel at rectangle position = red
- Pixel at circle position = blue
- Background pixels = white
- Transparent document → alpha=0 at background
- File readable by standard PNG viewers

## Specification References

- §5: Document and export (PNG output)
- §18: Rendering architecture (PNG encoding)
- §18.1: Color and alpha (sRGB output)

## Handoff Notes

**Completed Work**:
- Added `png = "0.18"` to `rupa-image` and `rupa-renderer`.
- Implemented `rupa-image` PNG encoding:
  - `encode_rgba_png` / `encode_rgba_png_to_writer` (8-bit RGBA, sRGB chunk)
  - `encode_png(ImageBuffer)` convenience
  - `unpremultiply_rgba` with transparent normalization and clamping
  - buffer/dimension validation before writing
- Implemented renderer PNG boundary:
  - `encode_png(pixels, width, height, path)` unpremultiplies then encodes
  - `encode_render_output` convenience
- Added 7 image tests and 5 renderer PNG tests, including a 200×100 minimal
  document and semi-transparent premultiplied→straight conversion.
- Recorded the color contract in `docs/decisions/DECISION-006.md`.

**Remaining Work**:
- None for TASK-009.

**Test Results**:
- `cargo test --workspace`: passes (rupa-syntax 77, rupa-compiler 64,
  rupa-geometry 9, rupa-image 7, rupa-ir 8, rupa-renderer 23).
- `cargo clippy --all-targets --all-features -- -D warnings`: passes.
- `cargo fmt -- --check`: passes.

**Known Issues**:
- PNG `decode_image` in `rupa-image` remains a placeholder (not required for
  M1; TASK-026).
- CLI still uses a placeholder output path and does not lower AST to IR
  (TASK-010).

**Exact Next Action**:
- Begin TASK-010: CLI check and render commands.
