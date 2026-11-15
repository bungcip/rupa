# TASK-065: Simplify compiler unit tests

**Milestone**: Maintenance (post-M9)
**Dependencies**: TASK-064
**Status**: Complete

## Goal

Reduce repeated assertion/helper code in compiler unit tests without reducing
coverage or changing test behavior.

## Scope

- Add shared `crates/rupa-compiler/src/test_support.rs` with `codes_of`.
- Remove duplicate `codes_of` helpers from document, layer, shapes, and
  validation test modules.
- Add `assert_valid` / `assert_invalid` helpers in shape tests.
- Replace repetitive shape valid/invalid assertion boilerplate with the helpers.
- Keep tests that inspect diagnostic help text explicit.

## Acceptance Criteria

- [x] One shared diagnostic-code test helper for compiler unit tests
- [x] Repeated shape valid/invalid assertion blocks reduced
- [x] Test count, coverage, and diagnostics remain unchanged
- [x] All workspace validation passes

## Verification

- `cargo fmt -- --check` — pass
- `cargo clippy --all-targets --all-features -- -D warnings` — pass
- `cargo build --workspace` — pass
- `cargo test --workspace` — pass (rupa-compiler: 185 tests)
