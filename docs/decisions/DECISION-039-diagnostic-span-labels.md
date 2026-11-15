# DECISION-039: Source quotes and span labels in human diagnostics

- **Status**: Accepted
- **Date**: 2026-11-14
- **Related**: TASK-090, TASK-004, specification §22.3

## Context

`render_diagnostic` printed the header, `--> path:line:column`, one source line,
and an unlabelled caret run. For single-line errors this was usable, but the
underline did not say what it marked, and a span crossing lines was collapsed to
a one-caret marker on the first line. Human and LLM repair are faster when the
exact source region is visible.

## Decision

1. `render_diagnostic` quotes the primary span from the source:
   - same-line spans show one line;
   - multi-line spans show the first line, up to one middle line, and the final
     line (at most three source lines);
   - the first line underlines from the start column, middle lines in full, and
     the final line to the end column.
2. The first caret run is labelled with the diagnostic message. The header keeps
   the same message, matching conventional compiler output where the span label
   may repeat part of the header.
3. Caret widths are derived from character columns, not byte offsets, so UTF-8
   source stays aligned.
4. Spans beyond the available source still render
   `(source line unavailable)` instead of panicking.
5. Machine-readable output (`check --json`, diagnostic edits, wasm metadata) is
   unchanged.

## Alternatives considered

- **Keep one line and add only a label**: cheaper, but hides the end of
  multi-line spans and still gives the reader an incomplete quote.
- **Full rustc-style multi-line annotation with vertical bars**: more faithful
  but more rendering code than the requested 1–3 line quote.
- **Add a separate label field to `Diagnostic`**: allows bespoke labels, but
  every producer would need to populate it; the message is already a useful
  label and no existing producer changes.

## Consequences

- Human diagnostics are more verbose but immediately actionable.
- The rendering rule is deterministic and bounded at three source lines.
- Any producer that puts a newline in a message gets its first line as the
  caret label; the rest remains in the header.
- `rupa check --json` consumers see no format change.
