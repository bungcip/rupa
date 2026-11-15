# TASK-090: Source quote, carets, and span labels in `rupa check`

## Task Header

- **Task ID**: TASK-090
- **Title**: Show a 1–3 line source quote with caret run and span label
- **Milestone**: Post-M14 follow-ups
- **Priority**: High
- **Dependencies**: TASK-089
- **Status**: Complete

---

## Description

`render_diagnostic` already had the primary span and printed one source line
with a caret run, but it did not show multi-line spans and did not label the
underlined range. `rupa check` now prints:

```text
error[L006]: unknown keyword or property `fil` in rectangle
  --> poster.rupa:4:5
  |
4 |     fil red
  |     ^^^ unknown keyword or property `fil` in rectangle
  |
  = help: Expected ..., did you mean `fill`?
```

- A single-line span quotes that line.
- A span crossing lines quotes up to three lines: the first from the start
  column, middle lines in full, and the final line to the end column.
- The first caret run carries the diagnostic message as its label.
- Caret width uses character columns rather than byte offsets, so non-ASCII
  source does not shift the underline.
- Source-less or out-of-range spans keep the `(source line unavailable)` path.

The JSON output is unchanged; diagnostics still carry the same spans and edits.

## Acceptance Criteria

- [x] Single-line diagnostics show the source line, carets, and label.
- [x] Multi-line diagnostics show up to three source lines and carets per line.
- [x] The label appears once, on the first caret run.
- [x] Long spans stop after three source lines.
- [x] `rupa check` integration test asserts the quote, caret, and label.
- [x] Spec §22.3 documents the human output.
- [x] fmt/clippy/build/workspace tests pass.
