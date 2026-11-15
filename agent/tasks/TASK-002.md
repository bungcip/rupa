# TASK-002: Lexer with indentation tracking

## Task Header
- **Task ID**: TASK-002
- **Title**: Lexer with indentation tracking
- **Milestone**: M1 — Minimal Vertical Slice
- **Priority**: Critical
- **Estimated Effort**: 4-6 hours
- **Dependencies**: TASK-001

## Description
Implement indentation-aware lexer for Rupa syntax per specification §3 and §4. The lexer must track indentation levels (4 spaces = 1 level), reject tabs, and emit `Indent`/`Dedent` tokens for the parser.

## Files to Create/Modify

| File | Action | Notes |
|------|--------|-------|
| `crates/rupa-syntax/src/token.rs` | Create | Token types and kinds |
| `crates/rupa-syntax/src/lexer.rs` | Create | Lexer implementation |
| `crates/rupa-syntax/src/span.rs` | Create | Source position tracking |
| `crates/rupa-syntax/src/error.rs` | Create | Lexer errors |
| `crates/rupa-syntax/src/lib.rs` | Modify | Export modules |

## Acceptance Criteria

**All must pass to mark task complete:**

- [x] Valid minimal example tokenizes correctly
- [x] Indentation tracking: 4 spaces = 1 level, emits `Indent`/`Dedent`
- [x] Tabs rejected with clear error and location
- [x] Unknown keywords produce error with location
- [x] Keywords recognized: `document`, `size`, `background`, `export`, `rectangle`, `circle`, `at`, `center`, `radius`, `fill`, `x`
- [x] Units recognized: `px`, `deg`, `%`, `pt`
- [x] Colors: named colors, hex strings (`"#RRGGBB"`, `"#RRGGBBAA"`)
- [x] Numbers: integer and float literals
- [x] Strings: double-quoted with escapes (`\"`, `\\`, `\n`, `\t`)
- [x] Comments: `//` to end of line
- [x] Coordinate pairs: `(x, y)` tokenized as group
- [x] Operator tokens: `+`, `-`, `*`, `/`, `%`, `==`, `!=`, `<`, `<=`, `>`, `>=`, `and`, `or`, `not`
- [x] All tokens carry source span (file, line, column)
- [x] Unit tests for all token types and error cases
- [x] `cargo test -p rupa-syntax` passes
- [x] `cargo fmt -- --check` passes
- [x] `cargo clippy -- -D warnings` passes

## Implementation Steps

1. Define `TokenKind` enum with all token types
2. Define `Token` struct with kind and span
3. Implement `Span` with file ID, line, column, offset
4. Implement `Lexer` with:
   - Input string and position tracking
   - Indentation stack for tracking levels
   - `next_token()` method
   - Indentation handling at line starts
5. Handle keywords vs identifiers
6. Handle numeric literals with units
7. Handle string literals with escapes
8. Handle comments
9. Write comprehensive unit tests

## Testing Requirements

- Unit tests for each token type
- Tests for valid indentation (consistent, nested)
- Tests for invalid indentation (inconsistent, tabs)
- Tests for unknown keywords
- Tests for malformed numbers, strings, hex colors
- Tests for coordinate pair tokenization
- Tests for operator precedence tokens

## Specification References

- §3: Source syntax overview (indentation rules)
- §4.1: Identifiers and strings
- §4.2: Numbers and units
- §4.3: Colors
- §4.4: Expressions (operators)

## Handoff Notes

**Completed Work**:
- Rewrote `lexer.rs` with accurate source span tracking (captures start position before scanning each token)
- Fixed blank/comment-only line handling: they no longer emit spurious `Indent`/`Dedent` tokens
- Tabs are now rejected everywhere (not just in indentation) with error code L002
- Fixed single `=` to emit `Assign` token (not `Colon`); `==` emits `Eq`
- Added `%` unit support (`Unit::Percent`)
- Added error diagnostics for unknown units (L009), invalid characters (L007), invalid escapes (L008), unterminated strings (L003), invalid indentation (L001)
- Wrote 27 comprehensive unit tests covering all token types, indentation scenarios, and error cases
- All validation commands pass: `cargo fmt -- --check`, `cargo clippy -- -D warnings`, `cargo build --workspace`, `cargo test --workspace`

**Remaining Work**:
- None (task complete)

**Test Results**:
- 27 unit tests in `rupa-syntax` pass
- All workspace tests pass (27 in rupa-syntax, 0 in other crates)
- No clippy warnings
- No fmt issues

**Known Issues**:
- None

**Exact Next Action**:
- Begin TASK-003: Implement parser for minimal grammar (document, rectangle, circle)