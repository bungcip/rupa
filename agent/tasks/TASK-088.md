# TASK-088: One canonical source for language metadata

## Task Header

- **Task ID**: TASK-088
- **Title**: Export `rupa_metadata()` and drive the playground highlighter and language reference from it
- **Milestone**: Post-M14 follow-ups
- **Priority**: High
- **Dependencies**: TASK-084, TASK-087
- **Status**: Complete

---

## Description

Keyword spellings, highlighter classes, named colours, units, and primitive
properties had been repeated in the lexer, `docs/language-reference.json`,
`WORD_CLASSES` in the playground, and the compiler's named-colour table. Two
drifts had already been fixed by hand (false colours in the highlighter and a
stale manual reference), so every new keyword risked a silently mis-coloured
editor.

The language facts now live in one place:

- `rupa-syntax::metadata` owns the keyword table, its highlighter class,
  documented/removed status, units, and primitive property lists. The lexer and
  `KNOWN_KEYWORDS` use the same table.
- `rupa-compiler::metadata_json` assembles those tables plus
  `paint::NAMED_COLORS` into JSON.
- `rupa-wasm` exposes the JSON as `metadata_json()` and the C ABI
  `rupa_metadata()`; the payload uses the existing frame header.
- `playground/app.js` builds its word map and unit regex from
  `rupa_metadata()` instead of a hard-coded list.
- The specification §22.1 names the CLI and wasm exports as the canonical
  source for the reference.
- `rupa metadata` prints the canonical JSON, `--output` writes it, and
  `--reference` merges its keys into the human-readable reference while
  preserving hand-authored prose fields.
- A CLI integration test compares the reference's canonical keys with the
  compiler metadata, so drift fails the suite.

Removed spellings (`document`, `export`) remain lexed for migration diagnostics
but are marked undocumented, so they never appear in the reference or the
playground highlighter. The reference's `text` primitive now lists the properties
the parser actually accepts (no `content` property).

## Acceptance Criteria

- [x] Adding a keyword to the canonical table updates the lexer, diagnostics,
      playground, and reference together.
- [x] Named colours and units come from the compiler/syntax tables.
- [x] `rupa_metadata()` works through the real wasm ABI.
- [x] `docs/language-reference.json` canonical keys match `rupa metadata`.
- [x] `cargo fmt --check`, clippy, build, and workspace tests pass.
