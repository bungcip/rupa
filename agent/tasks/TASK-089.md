# TASK-089: `rupa fmt --check` and `--stdin`

## Task Header

- **Task ID**: TASK-089
- **Title**: Make the formatter usable as a CI gate and editor filter
- **Milestone**: Post-M14 follow-ups
- **Priority**: High
- **Dependencies**: TASK-088
- **Status**: Complete

---

## Description

`rupa fmt <input>` only rewrote the file in place. It could not answer "is this
source already canonical?" with an exit code, and an editor could not pipe a
selection through it.

The command now has:

- `rupa fmt <input> --check` — writes nothing; exits 1 when the file is not
  canonical, with a short stderr message naming the file. Diagnostics for
  unformattable source keep their existing rendering.
- `rupa fmt --stdin` — reads source from standard input and writes canonical
  source to standard output, with no extra newline or file write. Diagnostics go
  to stderr and stdout stays empty when formatting fails.
- `rupa fmt --stdin --check` — reads stdin, emits no formatted output, and uses
  the same 0/1 exit status contract.

`<input>` is optional only when `--stdin` is present; supplying both is rejected
by clap. In-place formatting and its existing tests are unchanged.

## Acceptance Criteria

- [x] `fmt --check` returns 0 for canonical input and 1 for non-canonical input,
      without modifying the file.
- [x] `fmt --stdin` writes canonical source to stdout and leaves files alone.
- [x] `fmt --stdin --check` reports through the exit status only.
- [x] Formatting errors produce diagnostics on stderr, no stdout, exit 1.
- [x] `fmt --check` and `fmt --stdin` appear in `rupa fmt --help`.
- [x] Spec §21 documents the new behavior.
- [x] fmt/clippy/build/workspace tests pass.
