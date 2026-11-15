# TASK-027: Picture placement and fit modes

**Milestone**: 4 — Resources and Text
**Dependencies**: TASK-026 (image decoding), TASK-017 (transforms), TASK-021 (local coordinates)
**Status**: Completed

## Goal

Make `picture "path"` a first-class scene node that renders a decoded image
into a frame with one of the four fit modes from specification §10.

## Syntax

```rupa
picture "photo.png"
    at (100px, 100px)       // frame top-left (required)
    size 400px x 300px      // optional frame size
    fit cover               // contain (default), cover, fill, none
    rotate 15deg            // optional, about the frame center
    scale 1.2               // optional, about the frame center
    opacity 0.8             // optional
```

`at` is required and is the frame's top-left corner. `size` is optional; when
omitted the frame takes the image's natural pixel size. `fit` defaults to
`contain`.

## Semantics

- `contain`: scale uniformly to fit inside the frame, centered.
- `cover`: scale uniformly to cover the frame, centered and cropped to the
  frame.
- `fill`: scale non-uniformly to the frame exactly.
- `none`: draw at natural pixel size from the frame's top-left corner.
- The frame is clipped in device space (intersected with any enclosing layer
  clip), so rotated frames and `cover` crops use the transformed frame.
- Rotation/scale pivot on the frame center; opacity multiplies down from the
  enclosing layer.
- `picture` participates at z-index `0` with direct shapes, preserving source
  order among equal z-index children.

## Resource resolution

- The CLI resolves picture paths relative to the input document's directory.
- `render_document` (no resources) reports `MissingResource` for any picture;
  `render_document_with_resources` loads through `ResourceStore` and reports
  `Resource { path, detail }` for missing or undecodable images.
- `check` does not read resource files; missing-resource reporting happens at
  render time. Richer check-time resource diagnostics remain TASK-030.

## Files

- `crates/rupa-syntax/src/ast.rs`, `parser.rs`, `diagnostic.rs`
- `crates/rupa-compiler/src/shapes.rs`, `validation.rs`, `lower.rs`
- `crates/rupa-ir/src/document.rs`, `lib.rs`
- `crates/rupa-renderer/src/vector.rs`
- `crates/rupa-cli/src/commands.rs`, `Cargo.toml`
- `docs/decisions/DECISION-024.md`

## Acceptance Criteria

- [x] `picture "path"` parses with `at`, `size`, `fit`, and transform properties
- [x] `at` may be on the header or a property line
- [x] Missing `at` is reported as `S003`
- [x] Empty resource path is reported as `S022`
- [x] Non-positive or non-pixel frame size uses existing size diagnostics
- [x] Invalid fit mode is a syntax error
- [x] IR `PictureNode` carries path, frame, fit, transform, and opacity
- [x] Default fit is `contain`
- [x] `fill` stretches the image to the frame
- [x] `contain` preserves aspect ratio and centers inside the frame
- [x] `cover` preserves aspect ratio, fills the frame, and crops
- [x] `none` draws at natural size from the frame's top-left
- [x] Pictures are clipped by enclosing layers
- [x] Rendering without a resource store reports `MissingResource`
- [x] Missing resource files report the path
- [x] CLI resolves resources relative to the input file
- [x] CLI render of a picture document produces expected pixels
- [x] Existing golden output is unchanged
- [x] `cargo fmt -- --check` passes
- [x] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [x] `cargo build --workspace` passes
- [x] `cargo test --workspace` passes

## Handoff Notes

**Completed Work**:
- Added `picture` AST/parser support, validation, IR lowering, and rendering.
- Implemented device-space frame clipping and the four fit modes.
- Wired CLI rendering through `ResourceStore` rooted at the document directory.
- Added 7 renderer fit/clip/error tests, parser tests, compiler tests,
  validation tests, and 2 CLI integration tests.

**Remaining Work**:
- TASK-028: text primitive and font loading.

**Known Issues**:
- `check` does not verify that picture files exist (see TASK-030).
- `cover`/`fill` scale with nearest-neighbor sampling from `draw_pixmap`; high
  quality resampling is future work.
- Transform validation is reused, so rotation must use `deg` and scale must be
  positive.

**Exact Next Action**:
- Create `agent/tasks/TASK-028.md` and implement the text primitive and font
  loading.
