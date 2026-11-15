# TASK-096: Prune redundant unit tests

- **Priority**: Low
- **Dependencies**: TASK-095
- **Status**: Complete (uncommitted)
- **Owner**: implementation-agent

## Description

Remove unit tests that duplicate another test's coverage, and stop declaring
`size` in `.rupa` test sources when the test does not depend on the canvas
dimensions (the document default is 256×256).

## Changes

- `rupa-compiler/src/shapes.rs`: replaced 16 per-primitive validity tests with
  two table-driven tests (`valid_primitives_pass`,
  `closed_shape_fill_is_optional`).
- `rupa-syntax/src/parser.rs`: removed eight per-primitive missing-property
  tests, three redundant document/comment parse tests, and the count-only
  `test_parse_multiple_shapes`; extended
  `test_missing_required_properties_are_retained` to cover rectangle, circle,
  ellipse, square, and triangle.
- `rupa-syntax/src/lexer.rs`: removed `test_full_minimal_example`.
- `rupa-syntax/src/metadata.rs`: removed `every_keyword_has_a_token`; the
  keyword table and token conversion are generated from the same macro, so the
  test could never fail independently.
- `rupa-compiler/src/validation.rs`: merged the two omitted-size tests into
  `omitted_size_and_background_use_defaults`.
- `rupa-image/src/font.rs`: removed the `FontStore` cache test; the shared
  `PathCache<T>` now owns that behavior and is tested by `ResourceStore`.
- `rupa-cli/tests/cli.rs`: merged the `.jpg` and missing-extension output-path
  tests into `render_rejects_non_png_output_paths`.
- `examples/invalid/*.rupa`: removed the unnecessary leading
  `size 100px x 100px` (and the now-leading blank line) from all seven fixtures;
  the default 256×256 canvas is used and the documented diagnostic codes still
  match.

## Acceptance Criteria

- [x] No remaining test duplicates another test's assertions.
- [x] Tests that assert pixel dimensions keep their explicit `size`.
- [x] `cargo fmt --all -- --check` passes.
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes.
- [x] `cargo test --workspace` passes (531 tests, 0 failures).
- [x] `cargo build -p rupa-wasm --target wasm32-unknown-unknown --release` passes.
- [x] `examples/invalid/*.rupa` omit `size`; `executable_examples` remains green.
