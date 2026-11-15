# TASK-061: Migrate examples, benchmarks, reference JSON, and fixtures

**Milestone**: 9 — Document Model & Root Layer Refactor
**Dependencies**: TASK-057, TASK-058
**Status**: Complete

## Goal

Move every checked-in Rupa source, test fixture, and structured reference to the
file-as-document and root-layer model.

## Scope

TASK-057 already migrated the main fixtures to keep the workspace test suite
green. This task is now a final audit plus any residual documentation/prompt
changes.

- Remove any remaining `document "Name"` wrappers and unindent contents in:
  - `examples/*.rupa`
  - `examples/invalid/*.rupa`
  - `benchmarks/solutions/*.rupa`
  - `benchmarks/starters/*.rupa`
  - all Rust test source strings
- Update `docs/language-reference.json`:
  - remove the document declaration from keywords/grammar
  - document file-level settings and the implicit root layer
  - update the embedded example
- Update `examples/invalid/expected-diagnostics.json` for any changed codes.
- Update benchmark manifests/prompts if wording refers to a document block.
- Verify golden images remain valid; regenerate only if an intentional change is
  documented.

## Files to Modify

- `examples/`
- `benchmarks/`
- `docs/language-reference.json`
- `crates/rupa-cli/tests/`
- Any source-string fixtures under `crates/**/src/**/tests`

## Acceptance Criteria

- [x] No checked-in `.rupa` or test fixture uses the old `document` wrapper
- [x] Executable-example tests pass with the new syntax
- [x] Benchmark suite reference solutions score perfectly
- [x] Golden images remain pixel-identical
- [x] `docs/language-reference.json` example compiles in CI

## Testing

- `cargo test --workspace`
- `rupa bench` reference smoke test
- Golden comparison tests

## Implementation Notes

- Audited all checked-in `.rupa` sources under `examples/` and `benchmarks/`:
  none use the removed wrapper.
- Audited Rust test sources: only intentional migration tests in
  `rupa-syntax`/`rupa-compiler` contain `document "Old"`.
- `docs/language-reference.json` has no `document` keyword and uses file-level
  settings plus the implicit root-layer description.
- Added `checked_in_sources_do_not_use_removed_document_wrapper` to
  `crates/rupa-cli/tests/executable_examples.rs`. It recursively scans examples
  and benchmarks, checks the reference example, and prevents regressions.
- Benchmark reference smoke run: 8/8 attempted, compilation 1.0, rendering 1.0,
  visual similarity 1.0.
- No golden images needed regeneration.

## Verification

- `cargo fmt -- --check` — pass
- `cargo clippy --all-targets --all-features -- -D warnings` — pass
- `cargo build --workspace` — pass
- `cargo test --workspace` — pass
- `rupa bench` reference smoke test — perfect
