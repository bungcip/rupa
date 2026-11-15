# TASK-093: Reduce parser, wasm frame, and lowering duplication

## Task Header

- **Task ID**: TASK-093
- **Title**: Consolidate repeated primitive-body and lowering boilerplate
- **Milestone**: Maintenance
- **Priority**: Medium
- **Dependencies**: TASK-092
- **Status**: Complete

## Description

Several internal code paths repeated the same control flow:

- every shape primitive repeated the body entry, property loop, unknown-property
  diagnostic, and `Dedent` handling;
- `encode_frame`, `encode_format_frame`, and `encode_metadata_frame` each built
  the same `[total][meta_len][meta][body]` frame header;
- shape validation carried unused stroke parameters through every
  `validate_optional_stroke` call;
- lowering repeated the same missing-property diagnostic for pixels, brushes,
  pictures, and placements.

## Changes

- Added `Parser::parse_shape_body`, which owns the body loop, shared-property
  dispatch, unknown-property diagnostic, and dedent handling. Rectangle, circle,
  ellipse, square, triangle, polygon, line, polyline, path, picture, and text
  now supply only their primitive-specific properties.
- `parse_points_block` now receives the primitive name so its unexpected-token
  diagnostic names the right shape, with a regression test for polyline points.
- Added `encode_payload` in `rupa-wasm`; the three frame encoders now share the
  wire-header construction.
- Replaced `validate_optional_stroke` with `validate_optional_width`, removing
  unused `stroke` and `span` arguments.
- Reused the existing `required` lowering helper for pixel, brush, picture, and
  placement missing-property diagnostics.

No diagnostic codes, help text, grammar, or wire-format behavior changed.

## Verification

Run with `CARGO_INCREMENTAL=0`:

- `cargo fmt --all -- --check` — clean
- `cargo clippy --all-targets --all-features -- -D warnings` — clean
- `cargo build --workspace` — OK
- `cargo test --workspace` — 566 passed, 0 failed
