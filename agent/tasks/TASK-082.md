# TASK-082: Formatter expansion and comment preservation

## Task Header

- **Task ID**: TASK-082
- **Title**: Canonical formatting of inline properties and `//` comments
- **Milestone**: M13 — Inline primitive properties
- **Priority**: Critical
- **Dependencies**: TASK-081
- **Status**: Complete

---

## Description

`rupa fmt` now expands an inline property list into the canonical one-property-
per-indented-line form, which is what makes the playground's Format button useful
for compact input.

The lexer also records `//` comments (`Comment { span, text }`,
`tokenize_with_comments`) instead of discarding them; `tokenize()` still returns
the old signature for every other caller. The formatter flushes comments in
source order, keeps a comment-only line at its authored nesting level, and keeps
a trailing comment on the line where the reflow it belongs to ends.

---

## Bugs found and fixed

| Bug | Cause | Fix |
|-----|-------|------|
| `rupa fmt` deleted every comment | Lexer discarded comments and the formatter never re-emitted them | Comments are recorded and flushed by the formatter |
| Comment-only lines were dropped | They are skipped for indentation, so they never reached `scan_token` | `record_comment_only_line` captures them before `skip_line` |
| Formatting added a blank line per pass (not idempotent) | A flushed comment ended the line, then the newline token added another; the flag also outlived the token | Suppress the redundant newline and clear the flag on the next real token |
| Comment inside a block was flushed at column 0 | A comment precedes its block's `Indent` token, so block depth was not yet known | Indent from the comment's authored column |

---

## Files Created/Modified

| File | Action | Notes |
|------|--------|-------|
| `crates/rupa-syntax/src/lexer.rs` | Modify | `Comment`, `tokenize_with_comments`, `scan_comment`, `record_comment_only_line` |
| `crates/rupa-syntax/src/formatter.rs` | Modify | Inline expansion, `flush_comments`, comment-aware newlines |
| `crates/rupa-syntax/tests/format_inline.rs` | Create | 5 formatter tests, each asserting idempotency |

---

## Acceptance Criteria

- [x] Inline properties expand to one property per indented line
- [x] Header geometry stays on the primitive line
- [x] Formatting is idempotent for inline, nested, and commented sources
- [x] Comment-only and trailing comments survive formatting
- [x] Existing formatted examples keep their layout
