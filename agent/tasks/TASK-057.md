# TASK-057: Migrate AST and parser to file-as-document

**Milestone**: 9 — Document Model & Root Layer Refactor
**Dependencies**: TASK-056
**Status**: Complete

## Goal

Parse a `.rupa` file as one document with file-level settings and no `document`
wrapper, while giving authors a clear migration diagnostic for the removed
keyword.

## Scope

- Remove document-wrapper parsing from `Parser::parse` / `parse_document`.
- Parse top-level document settings without indentation:
  `size`, `background`, `seed`, `export`.
- Parse top-level `layer`, `place`, `let`, `repeat`, `fn`, and drawing primitives.
- Remove `Document.name` from the AST, or replace it with source-path metadata
  that is not part of the language.
- Keep the `document` token for migration detection only. Emit a stable
  diagnostic with help: remove the line and unindent its body.
- Enforce one `size` and at most one each of `background`, `seed`, and `export`.
  Add stable diagnostic codes:
  - `P004` removed `document` declaration
  - `C009` duplicate document size
  - `C010` duplicate background
  - `C011` duplicate seed
  - `C012` duplicate export
- Update the canonical formatter to emit file-level settings followed by
  top-level content with no wrapper indentation.
- Update lexer/parser/formatter unit tests and valid/invalid fixtures.

## Files to Modify

- `crates/rupa-syntax/src/parser.rs`
- `crates/rupa-syntax/src/ast.rs`
- `crates/rupa-syntax/src/token.rs`
- `crates/rupa-syntax/src/lexer.rs` (only if keyword removal affects tokens)
- `crates/rupa-syntax/src/diagnostic.rs`
- `crates/rupa-syntax/src/formatter.rs`
- Compiler tests that construct/expect `Document.name`

## Acceptance Criteria

- [x] Old `document "Name"` source is rejected with `P004` and actionable help
- [x] New file-level syntax parses with no indentation under a wrapper
- [x] Missing `size` still produces `C005`
- [x] Duplicate document settings produce their stable new codes
- [x] Formatter emits canonical file-level output and is idempotent
- [x] `cargo test -p rupa-syntax` passes
- [x] Full workspace compiles after dependent compiler updates

## Implementation Notes

- `Document.name` was removed; `Parser::parse` always returns a file-level
  `Document`.
- The legacy `document "Name"` block is consumed as one unit and produces only
  `P004`, avoiding a cascade of unrelated errors.
- Duplicate settings use `C009`–`C012` and keep the first declaration.
- `format_source` rejects the removed wrapper with `P004` and formats new
  file-level source idempotently.
- To keep the workspace test suite green, TASK-057 also migrated checked-in test
  source strings, `examples/`, `benchmarks/`, and `docs/language-reference.json`
  to file-level syntax. TASK-061 remains for the final audit and any residual
  docs/prompts.
- Canonical reordering of document settings to `size`, `background`, `seed`,
  `export` is still handled by TASK-062; the current token formatter preserves
  source setting order.

## Testing

- Parser tests for each document setting at top level, in any order
- Migration diagnostic test for `document`
- Formatter idempotence and golden string tests
- Invalid tests for duplicate settings

## Verification

- `cargo fmt -- --check` — pass
- `cargo clippy --all-targets --all-features -- -D warnings` — pass
- `cargo build --workspace` — pass
- `cargo test --workspace` — pass
- `rupa check`/`rupa fmt` smoke tests: new syntax succeeds, legacy wrapper emits `P004`
