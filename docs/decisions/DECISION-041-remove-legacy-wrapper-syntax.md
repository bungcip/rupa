# DECISION-041: Remove legacy wrapper syntax and catalog-only codes

**Date**: 2026-11-14
**Task**: TASK-095

## Context

Rupa has not been released, but the lexer and parser still recognized the
pre-M9 `document "Name"` wrapper and the removed `export "path"` setting solely
to emit migration diagnostics `P004` and `P005`. The same compatibility layer
kept `documented: false` keyword entries and several catalog-only diagnostic
codes (`C001`, `C002`, `C004`, `C005`, `C006`, `C012`, `S015`, `S019`, `S021`).

This adds branches, metadata fields, formatter checks, tests, and specification
text for syntax that no external source can depend on before release.

## Options considered

1. Keep the compatibility layer until after the first release.
2. Remove the special cases and let the old words fail like any unknown keyword;
   delete codes that cannot be emitted.
3. Reject the old words with a new generic error while keeping the old codes.

## Decision

Option 2.

- `document` and `export` are no longer lexer keywords. They are ordinary
  identifiers and produce the normal unknown-keyword diagnostic at file level.
- The parser no longer consumes a legacy wrapper or export declaration.
- The formatter no longer special-cases either word; it formats tokens and lets
  compile/render diagnostics report invalid source.
- The `documented` flag is removed from `KeywordSpec`; every keyword in the
  table is part of the language.
- Catalog-only codes from DECISION-034 are deleted. Remaining codes keep their
  existing numbers, so no renumbering is required.

## Consequences

- The lexer, parser, formatter, metadata, and diagnostic catalog have one fewer
  compatibility path.
- Old wrapper sources now report `L006` (unknown keyword) plus ordinary parse
  errors instead of `P004`/`P005`.
- No migration diagnostic exists; this is acceptable because no release has
  shipped those spellings.
- DECISION-034 is superseded.
