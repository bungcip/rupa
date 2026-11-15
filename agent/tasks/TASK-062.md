# TASK-062: Update CLI inspect/formatter output and docs

**Milestone**: 9 — Document Model & Root Layer Refactor
**Dependencies**: TASK-057, TASK-058
**Status**: Complete

## Goal

Make CLI tooling and documentation reflect the file-as-document and implicit
root-layer model.

## Scope

- `rupa fmt`: emit canonical file-level output without a wrapper and with
  document settings in canonical order.
- `rupa inspect`: report document settings, the implicit root layer, explicit
  layer definitions, instances, and composition order. Do not report a
  source-level document name.
- Update CLI help text and JSON schema documentation.
- Update `benchmarks/README.md` and any docs that describe the old wrapper.
- Update `docs/language-reference.json` if TASK-061 has not already covered it.

## Acceptance Criteria

- [x] `rupa fmt` output is canonical and idempotent for new syntax
- [x] `rupa fmt` reports migration diagnostics for old syntax
- [x] `rupa inspect` exposes the implicit root layer
- [x] CLI docs/help do not mention a document declaration
- [x] `cargo test -p rupa-cli` passes

## Testing

- CLI integration tests for fmt and inspect
- Idempotence test on all examples/benchmark solutions
- JSON inspection assertions for root-layer structure

## Implementation Notes

- `format_source` now canonicalizes top-level document settings before token
  formatting:
  - `size`
  - `background`
  - `seed`
  - `export`
  - then remaining top-level content in source order
- Only column-zero declarations are moved; indented properties with the same
  names are untouched.
- `rupa fmt` already rejects the removed wrapper via `P004`; a failed format does
  not rewrite the input file.
- `rupa inspect` already reports compiled graphics IR with the implicit root
  layer from TASK-058.
- Added formatter unit tests for canonical ordering, indented-property safety,
  and idempotence.
- Added CLI tests for fmt ordering/idempotence and legacy-wrapper rejection.
- Updated inspect/CLI help text from the older “not implemented” wording in
  earlier tasks.

## Verification

- `cargo fmt -- --check` — pass
- `cargo clippy --all-targets --all-features -- -D warnings` — pass
- `cargo build --workspace` — pass
- `cargo test --workspace` — pass
