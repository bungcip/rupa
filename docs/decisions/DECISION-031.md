# DECISION-031: Remove `export` from the language; output is CLI-determined

## Context

`export "file.png"` is currently a file-level document setting implemented by
the parser, formatter, and CLI. Request: remove it. Output should be decided by
the CLI invocation, not encoded in the `.rupa` source. (The user also
reconfirmed that the `document` wrapper stays removed per DECISION-026.)

## Options considered

1. Keep `export` as a document setting (status quo).
2. Remove `export`; the CLI determines the output path.

## Decision

Remove `export` from the language entirely.

- `export` is no longer a document setting; using it is a parse/semantic error
  with a migration diagnostic pointing to the CLI `--output` option.
- `rupa render input.rupa` writes `input.png` by default (input filename with
  the `.png` extension, in the current working directory).
- `rupa render input.rupa --output path.png` overrides the default.
- Batch generation is expressed by invoking the CLI repeatedly, not by
  declaring exports in source.

## Consequences

- One `.rupa` source can render to any path without modification; sources
  become output-agnostic.
- Parser: reject `export` with a stable migration diagnostic (mirrors the
  removal of the `document` wrapper, §5.1).
- Formatter: canonical document-setting order becomes `size`, `background`,
  `seed`.
- CLI, language-reference JSON, fixtures, and examples that use `export` must
  be migrated (tracked in the backlog as a follow-up implementation task).
