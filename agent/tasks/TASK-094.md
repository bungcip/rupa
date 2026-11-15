# TASK-094: Cross-crate duplication cleanup

## Task Header

- **Task ID**: TASK-094
- **Title**: Reduce duplication in IR, geometry, image, renderer, and CLI
- **Milestone**: Maintenance
- **Priority**: Medium
- **Dependencies**: TASK-093
- **Status**: Complete

## Description

After the syntax/compiler pass, the remaining crates still repeated several
small patterns:

- `GraphicsDocument` had four near-identical root-child accessors with
  exhaustive match arms;
- `rotate_about` and `scale_about` repeated pivot transform composition;
- `ResourceStore` and `FontStore` duplicated path resolution, cache lookup, and
  cache iteration;
- five renderer filters repeated per-pixel RGB loops, and blend-mode conversion
  lived inline in the compositor;
- CLI `check`, `render`, and `inspect` repeated read-then-compile and base-dir
  setup.

## Changes

- `rupa-ir`: added a private `root_nodes` helper; `shapes`, `pictures`, `texts`,
  and `instances` now only provide the matching arm.
- `rupa-geometry`: added `Transform::about_pivot`; `rotate_about` and
  `scale_about` share it.
- `rupa-image`: added private `cache::PathCache<T>` for base-dir resolution,
  lookup, insertion, count, and path iteration. `ResourceStore` and `FontStore`
  wrap the cache while keeping their public APIs and error types unchanged.
  Added `FontStore` cache and error-path tests.
- `rupa-renderer`: added `to_skia_blend_mode` and `for_each_rgb`; brightness,
  contrast, saturation, grayscale, hue-rotate, noise, and vignette now share the
  RGB traversal. Added brightness and grayscale filter tests.
- `rupa-cli`: added `compile_input` and `input_base_dir`; `check`, `render`, and
  `inspect` reuse them.

No filter math, render order, diagnostic codes, public APIs, or output formats
changed.

## Verification

Run with `CARGO_INCREMENTAL=0`:

- `cargo fmt --all -- --check` — clean
- `cargo clippy --all-targets --all-features -- -D warnings` — clean
- `cargo build --workspace` — OK
- `cargo test --workspace` — 570 passed, 0 failed
