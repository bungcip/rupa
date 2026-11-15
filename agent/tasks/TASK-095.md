# TASK-095: Remove legacy document/export wrapper handling

## Task Header

- **Task ID**: TASK-095
- **Title**: Delete pre-release wrapper syntax and catalog-only legacy codes
- **Milestone**: Maintenance
- **Priority**: High
- **Dependencies**: TASK-094
- **Status**: Complete

## Description

Rupa has not been released, but the syntax layer still recognized the old
`document "Name"` wrapper and `export "path"` setting only to emit migration
diagnostics. This kept a compatibility path in the lexer, parser, formatter,
metadata table, diagnostics catalog, tests, and specification.

## Changes

- Removed `Document` and `Export` token variants and their metadata entries.
- Removed `Parser::parse_removed_document` / `parse_removed_export` and the
  top-level branches that called them.
- Removed formatter checks that rejected the old words.
- Removed the `documented` field from `KeywordSpec`; all remaining keywords are
  part of the language. `documented_keywords` became `keywords_in_class`.
- Deleted catalog-only codes `P004`, `P005`, `C001`, `C002`, `C004`, `C005`,
  `C006`, `C012`, `S015`, `S019`, and `S021`.
- Updated tests so legacy wrapper sources are rejected as ordinary unknown
  keywords (`L006`), and removed legacy-specific CLI/WASM/example tests.
- Updated specification §5.1, §23.1, and the M10 milestone note.
- Recorded `DECISION-041`; marked `DECISION-034` superseded.

## Verification

Run with `CARGO_INCREMENTAL=0`:

- `cargo fmt --all -- --check` — clean
- `cargo clippy --all-targets --all-features -- -D warnings` — clean
- `cargo build --workspace` — OK
- `cargo test --workspace` — 562 passed, 0 failed
- `cargo build -p rupa-wasm --target wasm32-unknown-unknown --release` — OK
