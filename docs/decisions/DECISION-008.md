# DECISION-008: Golden-image test location and policy

**Date**: 2026-11-10
**Task**: TASK-011 (Integration test with golden image)

## Context

TASK-011 proposes `tests/integration_test.rs`, `tests/common.rs`, and golden
files at the workspace root. The workspace root is a virtual manifest without
a package, so root-level test files are not compiled or run by
`cargo test --workspace`.

Cargo exposes `CARGO_BIN_EXE_rupa` only to integration tests inside the package
that defines the `rupa` binary (`rupa-cli`). Spawning the CLI is required for
an end-to-end test of the actual command-line contract.

## Decision

- Integration and golden-image tests live in the `rupa-cli` package:
  - `crates/rupa-cli/tests/integration_test.rs`
  - `crates/rupa-cli/tests/common/mod.rs` (shared helpers, not a test target)
- Fixtures remain at the workspace root as specified:
  - `examples/minimal.rupa`
  - `tests/golden/minimal.png`
- Helpers resolve the workspace root from `CARGO_MANIFEST_DIR`, so tests run
  from any working directory.
- Comparison is pixel-exact on decoded straight-alpha RGBA and also verifies
  PNG dimensions.
- Determinism is verified by rendering the same example twice and requiring
  byte-identical PNG files. The fresh output is additionally required to be
  byte-identical to the committed golden.
- `assert_render_matches_golden(source, golden_name)` is the reusable helper
  for future golden tests. It renders through the real `rupa render` binary,
  decodes both images, and reports the first mismatching pixel.
- Golden images are regenerated only intentionally:
  `cargo run -p rupa-cli -- render examples/minimal.rupa --output tests/golden/minimal.png`.

## Consequences

- The acceptance criterion "runs in `cargo test --workspace`" is met without
  adding a synthetic root package or a separate test-only workspace member.
- Future golden tests add a `.rupa` fixture and a committed PNG, then call the
  reusable helper.
- Byte-identical golden comparison may require intentional regeneration if
  the PNG encoder or renderer version changes.
