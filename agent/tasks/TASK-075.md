# TASK-075: Renderer hardening and golden parity

## Task Header

- **Task ID**: TASK-075
- **Title**: Remove browser-trap panics and prove byte parity with the CLI golden
- **Milestone**: M11 — Browser renderer core (WASM)
- **Priority**: Critical
- **Dependencies**: TASK-073, TASK-074
- **Status**: Complete

---

## Description

A WASM trap is unrecoverable in the browser, so renderer panics had to become
errors:

- `Paint::Pattern` allocated its tile with `Pixmap::new(...).unwrap()`; it now
  returns the new `RenderError::PatternTile`.
- `picture` primitives in the browser fail fast through
  `RenderError::MissingResource` (rendering without a `ResourceStore`), which
  becomes `R001` with an explanation instead of filesystem access.

Native tests then pin the browser output to the CLI:

| Test | Purpose |
|------|---------|
| `valid_source_renders_png_with_document_dimensions` | PNG signature + metadata |
| `invalid_source_reports_stable_diagnostic_with_position` | `S005` with 1-based positions |
| `rendering_is_deterministic` | `examples/procedural.rupa` renders identically twice |
| `png_matches_the_cli_golden_image` | byte-for-byte match with `tests/golden/minimal.png` |
| `frame_wire_format_roundtrips_metadata_and_png` | header/length layout |
| `picture_without_file_access_reports_resource_diagnostic` | `R001` path |
| `abi::tests::*` | alloc/free and ABI render parity |

The built module was also driven through its ABI from Node
(`target/wasm-check.mjs`): exports are `memory`, `rupa_alloc`, `rupa_free`,
`rupa_free_frame`, `rupa_render`, `rupa_set_font`; `examples/minimal.rupa`
matches the 1,180-byte golden image; repeat renders are identical; invalid
source reports `S005`.

---

## Files Created/Modified

| File | Action | Notes |
|------|--------|-------|
| `crates/rupa-renderer/src/vector.rs` | Modify | `PatternTile` error instead of `unwrap` |
| `crates/rupa-wasm/tests/render.rs` | Create | Six native tests |
| `crates/rupa-wasm/src/abi.rs` | Create | ABI tests |

---

## Acceptance Criteria

- [x] No `unwrap` remains on the pattern-tile allocation path
- [x] Browser render of `examples/minimal.rupa` equals `tests/golden/minimal.png`
- [x] Two renders of the same source are byte-identical
- [x] Invalid source reports a stable code with line and column
- [x] `cargo fmt -- --check` passes
- [x] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [x] `cargo build --workspace` passes
- [x] `cargo test --workspace` passes

---

## Known Issues / Environment Notes

- `cargo build` fails on this UNC-mounted workspace with
  "incremental compilation: could not create session directory lock file"; set
  `CARGO_INCREMENTAL=0` for all cargo commands here.
- `clippy::items_after_test_module` is allowed on the existing
  `vector.rs` test module, as it was before the refactor.
