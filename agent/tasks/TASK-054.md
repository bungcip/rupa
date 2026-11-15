# TASK-054: CI for executable examples

**Milestone**: 8 — LLM Authoring Experience
**Dependencies**: TASK-010, TASK-051
**Status**: Complete

## Goal

Make examples executable documentation: valid examples must compile and render in CI, intentionally invalid examples must fail with documented diagnostic codes, and the structured language-reference example must compile.

## Scope

- Add a set of self-contained valid examples under `examples/` covering primitives, layers, procedural constructs, raster operations, and advanced paints.
- Add intentionally invalid examples under `examples/invalid/` with an `expected-diagnostics.json` manifest of expected codes.
- Add `crates/rupa-cli/tests/executable_examples.rs`:
  - checks and renders every top-level `examples/*.rupa` file
  - verifies every invalid example fails with its documented diagnostic codes and that the manifest matches the directory contents
  - compiles the `example` embedded in `docs/language-reference.json`
- Add a GitHub Actions workflow that runs fmt, clippy, build, and the full test suite.
- Exclude specification snippets that are fragments or use provisional/unimplemented syntax (for example `random(...)` inside shape positions) from the executable set; they are documentation proposals, not currently executable programs.

## Files Created/Modified

- `examples/shapes.rupa`
- `examples/layers.rupa`
- `examples/procedural.rupa`
- `examples/raster.rupa`
- `examples/paints.rupa`
- `examples/invalid/unknown_property.rupa`
- `examples/invalid/missing_radius.rupa`
- `examples/invalid/unknown_layer.rupa`
- `examples/invalid/bad_path.rupa`
- `examples/invalid/expected-diagnostics.json`
- `crates/rupa-cli/tests/executable_examples.rs`
- `crates/rupa-cli/tests/common/mod.rs` (allow dead code per test binary)
- `.github/workflows/ci.yml`

## Acceptance Criteria

- [x] Every `examples/*.rupa` file compiles and renders
- [x] Every `examples/invalid/*.rupa` file fails with the codes in `expected-diagnostics.json`
- [x] The `docs/language-reference.json` example compiles
- [x] CI workflow runs format, clippy, build, and tests
- [x] `cargo fmt -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --workspace`, and `cargo test --workspace` pass

## Verification

- `cargo fmt -- --check` — pass
- `cargo clippy --all-targets --all-features -- -D warnings` — pass
- `cargo build --workspace` — pass
- `cargo test --workspace` — pass (syntax 110, compiler 180, ir 29, geometry 9, image 21, renderer 67, CLI 30 + 7 + 3 executable-example tests)
