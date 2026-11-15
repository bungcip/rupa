# TASK-026: Image resource loading and decoding

**Milestone**: 4 — Resources and Text
**Dependencies**: TASK-009 (PNG encoding), TASK-025
**Status**: Completed

## Goal

Decode external raster images into the renderer's canonical pixel layout and
provide a deterministic, path-aware loader that later picture placement
(TASK-027) can use.

## Scope

- Decode PNG files and bytes into 8-bit straight-alpha RGBA8, row-major with a
  top-left origin.
- Detect the format from the PNG signature, not the file extension.
- Expand grayscale, grayscale+alpha, RGB, indexed, and RGBA inputs; reduce
  16-bit samples to 8-bit.
- Produce diagnostics that name the failing resource path.
- Resolve resource references relative to a document/project directory and
  cache decoded buffers by resolved path.

Out of scope: `picture` syntax, IR nodes, placement, fit modes, rendering
(TASK-027), and non-PNG input formats (specification §10 allows adding them
later).

## Files

- `crates/rupa-image/src/decode.rs`
- `crates/rupa-image/src/resource.rs`
- `crates/rupa-image/src/lib.rs`
- `docs/decisions/DECISION-023.md`

## Acceptance Criteria

- [x] PNG bytes decode to straight-alpha RGBA8 with correct dimensions
- [x] RGBA alpha values are preserved exactly
- [x] RGB and grayscale inputs expand to opaque RGBA
- [x] Grayscale+alpha expands to RGBA
- [x] Indexed PNG expands through its palette
- [x] Non-PNG input reports `UnsupportedFormat` naming the path
- [x] Corrupt/truncated PNG reports a decoding error
- [x] Missing file reports a read error naming the path
- [x] Zero dimensions and malformed buffer lengths are rejected
- [x] `ImageBuffer::pixel` is bounds-checked
- [x] `ResourceStore` resolves relative references against its base directory
- [x] Decoded images are cached by resolved path (repeat loads do not read disk)
- [x] Missing resource errors report the resolved path
- [x] `cargo fmt -- --check` passes
- [x] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [x] `cargo build --workspace` passes
- [x] `cargo test --workspace` passes

## Testing

- `cargo test -p rupa-image`: 21 tests pass (14 decode/resource tests plus 7
  existing encoding tests).
- No new end-to-end CLI behavior in this task; picture rendering is TASK-027.

## Handoff Notes

**Completed Work**:
- Implemented PNG decoding with `png` 0.18 and normalized all color types to
  RGBA8.
- Added `ResourceStore` with base-directory resolution, by-path caching, and
  sorted cache inspection.
- Added path-carrying `ImageError` plus path-independent `ImageErrorKind`.
- Added `ImageBuffer::new` validation and bounds-checked `pixel`.
- Recorded conventions in `docs/decisions/DECISION-023.md`.

**Remaining Work**:
- TASK-027: `picture` syntax, IR node, placement, and fit modes.

**Known Issues**:
- Only PNG input is supported, as allowed by specification §10. Other formats
  should be added through decoding dependencies in a later task.
- Decoding ignores color-space metadata and applies no gamma conversion;
  samples are copied as stored, matching deterministic output.
- `ResourceStore` does not canonicalize paths, so the same file referenced via
  different spellings is cached twice. This keeps diagnostics readable.

**Exact Next Action**:
- Create `agent/tasks/TASK-027.md` and implement picture placement and fit
  modes.
