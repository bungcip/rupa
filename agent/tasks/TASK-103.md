# TASK-103: Pack source locations and derive line/column on demand

- **Milestone**: M18 — Packed source locations
- **Priority**: High
- **Status**: Complete
- **Dependencies**: TASK-102 (M15–M17 closed)
- **Owner**: implementation-agent

## Goal

Reduce source-location memory beyond the M17 layout by making `Span` eight
bytes, while keeping human diagnostics and JSON positions unchanged for valid
source files.

## Scope

- Change `FileId` to `u8` and document a 256-file limit.
- Limit `.rupa` source files to 16,777,215 bytes (16 MiB - 1).
- Replace the line/column/offset `Position` with an offset-only position.
- Pack file id + 24-bit offset into a `Location`; make `Span` two packed words.
- Add `LineIndex` to derive one-based line/column at diagnostic boundaries.
- Add `L010` for oversized source, and stop before scanning it.
- Update CLI and wasm JSON positions to use `LineIndex`.
- Update formatter comment indentation to derive the column from the source.
- Update size/allocation guardrails and add packing/limit tests.
- Update the specification and DECISION-043.

## Non-Goals

- No change to IR JSON, diagnostic messages/codes for existing errors, render
  order, or golden PNGs.
- No include/import implementation; the 256-file limit reserves the identifier
  space.
- No new line-index storage in the AST or parser.

## Acceptance Criteria

- [x] `size_of::<Span>() == 8`, `size_of::<Position>() == 4`,
      `size_of::<FileId>() == 1`.
- [x] Packed locations round-trip file ids `0..=255` and offsets up to
      `16,777,215`.
- [x] Oversized source reports `L010` and produces no partial token stream.
- [x] Human diagnostics and `check --json` report the same one-based
      line/column values as before for existing invalid fixtures.
- [x] Golden PNGs and IR JSON are unchanged.
- [x] Formatter comment placement is unchanged.
- [x] `cargo fmt`, clippy, workspace build, tests, and wasm release build pass.

## Result

| Type | M17 | M18 |
|------|----:|----:|
| `FileId` | 4 | 1 |
| `Position` | 16 | 4 |
| `Span` | 40 | 8 |
| `Token` | 64 | 32 |
| `Document` | 360 | 200 |
| `DocumentChild` | 720 | 304 |
| `Text` | 680 | 296 |
| `Fill` | 72 | 40 |
| `Expression` | 72 | 40 |
| `GroupDefinition` | 224 | 96 |
| `LayerDefinition` | 368 | 208 |

- Fixture parse: 271 allocations / **50,387 bytes** (from 101,203; -50.2%).
- wasm release: **1,377,325 bytes** (from 1,400,685; -1.7%).
- All 556 workspace tests pass; clippy and fmt are clean.
- CLI `check --json` positions on `examples/invalid/unknown_property.rupa`
  remain line 3, column 5 / line 3, column 12; golden PNGs are byte-identical.

## Handoff Notes

**Completed Work**:
- Packed source locations and added `LineIndex`; updated all diagnostic output
  boundaries, formatter indentation, tests, spec, and DECISION-043.

**Remaining Work**:
- TASK-104: final M18 verification, performance comparison, and close-out.
- Commit authorization was received; TASK-103 is committed.

**Exact Next Action**:
- Run TASK-104 and close M18, then report the new measurements.
