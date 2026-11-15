# TASK-107: Simplify the lexer implementation and tests

- **Milestone**: Maintenance (after M19)
- **Priority**: Medium
- **Status**: Complete
- **Dependencies**: TASK-106
- **Owner**: implementation-agent

## Goal

Reduce duplication and incidental complexity in `crates/rupa-syntax/src/lexer.rs`
without changing token streams, diagnostics, or rendered output.

## Scope

- Add token-emission helpers and use them for punctuation, operators, and
  indentation tokens.
- Replace repeated raw `Diagnostic` literals with `DiagnosticBag` helpers.
- Share unterminated-string handling and compute unit spans once.
- Simplify blank/comment-line detection.
- Add lexer test macros for successful parses, expected errors, and token-kind
  assertions.
- Verify parser/lexer tests, workspace gates, output parity, wasm, and timings.

## Non-Goals

- No grammar, token kind, diagnostic code/message, span, JSON, or image changes.
- No new dependencies.
- No changes outside `rupa-syntax` except task/status documentation.

## Acceptance Criteria

- [x] Lexer source and test module shrink while preserving behavior.
- [x] All parser/lexer/syntax tests pass.
- [x] Golden PNGs, `inspect` JSON, invalid-fixture `check --json`, and the wasm
      render are byte-identical to `82c9ccf`.
- [x] `cargo fmt --check`, clippy, workspace build, and workspace tests pass.
- [x] Task/status/current/handoff documentation is updated.
- [x] Commit only after explicit user authorization.

## Result

`lexer.rs` fell from **1,074 to 852 lines** (-222, -20.7%): 503 lines removed
and 281 added. Production code fell from 666 to 507 lines; the test module fell
from 408 to 345 lines.

Simplifications:

- `push_token` / `emit` replace repeated span construction and token pushes.
- `match` guards collapse the paired-operator branches (`==`, `!=`, `<=`, `>=`)
  and comment-slash detection into single arms.
- 10 raw `Diagnostic { ... }` literals now use
  `DiagnosticBag::error` / `error_with_help`.
- `finish_unterminated_string` shares the newline/EOF recovery path.
- Unit spans are built once per suffix instead of twice.
- `is_blank_or_comment_line` uses a line slice instead of a manual character
  loop.
- Test macros `lex_ok!`, `assert_lex_error!`, and `assert_tokens!` replace
  repeated parse/assert/kind-vector boilerplate.

Verification:

- 131 `rupa-syntax` library tests and 557 workspace tests pass.
- fmt, clippy with `-D warnings`, and workspace build are clean.
- All PNGs, valid-example `inspect` JSON, invalid-fixture `check --json`, and
  the Node wasm render are byte-identical to `82c9ccf`.
- wasm release is 1,350,685 bytes.
- 20 alternating release runs: `check minimal` -7.4%, `render minimal` -13.6%,
  `layers` -2.8%, `text` -6.0%, `paints` -1.7%. `fmt` and `check --json` on a
  42 KB source were neutral; the human `check` path varied by about +7%, which
  is diagnostic-output/code-layout bound rather than lexing work.

## Handoff Notes

**Completed Work**:
- Simplified lexer emission, operator dispatch, diagnostics, strings, units,
  blank-line detection, and test assertions; verified parity and gates.

**Remaining Work**:
- Commit requires explicit user authorization.

**Known Issues**:
- None introduced; token streams, diagnostics, and outputs are byte-identical.
- The large human-readable `check` timing is noisy because it is dominated by
  rendering 1.7 MB of diagnostics; structured output and formatting are neutral.

**Exact Next Action**:
- Await commit authorization, then pick the next backlog item.
