# DECISION-036: Machine-applicable edits on diagnostics

- **Status**: Accepted
- **Date**: 2026-11-14
- **Related**: TASK-087, specification §22.3

## Context

Diagnostics that already know the exact fix — a misspelled keyword (`TASK`'s
`L006`, `suggest_keyword`) or an unknown named colour (`T002`,
`did_you_mean_from(NAMED_COLOR_NAMES)`) — only carried that fix inside the
human-readable `help` string. A tool (editor, LLM agent, batch repair script)
had to parse prose to apply the fix, which is brittle and loses the span.

## Decision

1. `rupa-syntax::diagnostic::Diagnostic` gains
   `edits: Vec<SuggestedEdit>`, where
   `SuggestedEdit { span: Span, replacement: String }`.
   - The field is `#[serde(default)]` and skipped when empty, so existing
     serialized diagnostics are unchanged.
   - Edits are emitted only when the replacement is exact and the span is
     exactly the text to replace. Today that is:
     - `Diagnostic::with_suggestion` (unknown keyword): the primary span (the
       identifier token) is replaced by the suggested keyword.
     - Unknown named colour: the colour-name span is replaced by the suggested
       name.
   - Diagnostics whose suggestion is advisory only (unknown group/variable
     names, where the primary span covers a larger construct) keep their help
     text and no edit.
2. `rupa check --json` prints one object:
   `{ "path", "status": "ok" | "error", "diagnostics": [...] }`.
   Each diagnostic has `code`, `severity`, `message`, `line`, `column`,
   `endLine`, `endColumn`, `help`, and `edits`. Each edit has `startLine`,
   `startColumn`, `startOffset`, `endLine`, `endColumn`, `endOffset`, and
   `replacement`. `edits` is always present (possibly `[]`).
3. Human output (`rupa check` without `--json`) is unchanged: same messages,
   codes, and exit status.

## Alternatives considered

- **Separate side-table of fixes** returned beside the diagnostic bag: avoids
  touching the struct, but every producer and consumer has to carry and match a
  second collection, and fixes can drift from their diagnostic.
- **`Option<SuggestedEdit>`**: only supports one fix; a `Vec` leaves room for
  multi-span fixes later without another breaking change.

## Consequences

- All `Diagnostic { .. }` literals must initialize `edits` (tests and internal
  producers included); `Diagnostic::error` and both `to_diagnostic` adapters
  default it to empty.
- Consumers get deterministic, position-addressed repairs without parsing help
  text. Applying every edit of a diagnostic yields the corrected source.
- `rupa-wasm` metadata keeps its existing shape; extending it is a separate
  change.
