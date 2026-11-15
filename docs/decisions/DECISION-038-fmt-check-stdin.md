# DECISION-038: Formatter check and stdin modes

- **Status**: Accepted
- **Date**: 2026-11-14
- **Related**: TASK-089, TASK-050, specification §21

## Context

`rupa fmt` rewrote one file in place. The repository already had a
`checked_in_examples_are_fmt_stable` test, but there was no CLI way for CI or a
pre-commit hook to enforce the same rule, and editors had no filter mode for
formatting a buffer or selection.

## Decision

Extend `rupa fmt` with two independent flags:

- `--check`: format in memory, write nothing, and exit 1 when the input differs
  from its canonical form. Success is 0. Unformattable input keeps the existing
  diagnostics on stderr and exits 1.
- `--stdin`: read source from stdin and write canonical source to stdout. No
  file is touched. With `--check`, suppress the formatted output and use only
  the exit status.

The positional input is optional only with `--stdin`; combining a path with
`--stdin` is a clap usage error. In-place formatting remains the default when
neither flag is present.

## Alternatives considered

- **Always print formatted output instead of rewriting**: breaks the existing
  in-place workflow and would require editors to redirect files themselves.
- **A separate `rupa fmt-check` command**: duplicates parsing and error handling
  for no benefit.
- **`--check` printing a unified diff**: useful, but more output and dependency
  surface than requested; the stderr message names the file and a tool can run
  `rupa fmt <file>` to produce the canonical text.

## Consequences

- CI and pre-commit hooks can enforce canonical formatting with
  `rupa fmt --check <file>`.
- Editors can implement format-on-save or format-selection with
  `rupa fmt --stdin`.
- Exit status 1 means either "differs from canonical" or "could not format";
  stderr distinguishes the two.
- Existing in-place behavior, formatter output, and diagnostics are unchanged.
