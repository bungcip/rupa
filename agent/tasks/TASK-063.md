# TASK-063: End-to-end migration verification

**Milestone**: 9 — Document Model & Root Layer Refactor
**Dependencies**: TASK-057 … TASK-062
**Status**: Complete

## Goal

Verify the full repository after the document-model and root-layer migration,
audit for leftovers, and close Milestone 9.

## Scope

- Run full validation:
  - `cargo fmt -- --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo build --workspace`
  - `cargo test --workspace`
- Search for remaining `document "` syntax, `Document.name`, direct root-shape
  assumptions, and obsolete `C001`/`C002` behavior.
- Verify all examples, benchmarks, golden images, `inspect`, and `fmt`.
- Update `agent/STATUS.md`, `agent/ROADMAP.md`, `agent/BACKLOG.md`,
  `agent/CURRENT.md`, and `docs/language-reference.json` version if needed.
- Write the final milestone handoff.

## Acceptance Criteria

- [x] All required validation commands pass
- [x] No checked-in source/fixture uses the removed `document` wrapper
- [x] Root layer behavior and reserved name are tested end to end
- [x] Examples and benchmarks compile and render
- [x] Milestone 9 status is accurate
- [x] Handoff documents remaining risks and follow-up work

## Testing

- Full workspace test suite
- Executable-example tests
- Benchmark reference run
- Manual `rupa check`, `rupa fmt`, `rupa inspect`, and `rupa render` smoke tests

## Final Audit Results

- No checked-in `.rupa` source uses `document "Name"`.
- Legacy `document "Old"` strings exist only in intentional migration tests in
  `crates/rupa-syntax/src/parser.rs` and
  `crates/rupa-compiler/src/validation.rs`.
- No `Document.name`/`doc.name` references remain.
- AST `Document.children` references remain intentionally (top-level AST
  children); compiled IR uses `GraphicsDocument.root.children`.
- `C001`/`C002` remain only as stable legacy constants and stability tests; no
  compiler path emits them.
- `GraphicsDocument` is always constructed via `new`/`Default`, both creating an
  implicit root layer.
- All examples/benchmark fixtures, executable-example tests, benchmark reference
  run, and golden image tests pass.
- `rupa bench` reference run: 8/8 tasks, compile/render/similarity = 1.0.

## Verification

- `cargo fmt -- --check` — pass
- `cargo clippy --all-targets --all-features -- -D warnings` — pass
- `cargo build --workspace` — pass
- `cargo test --workspace` — pass
