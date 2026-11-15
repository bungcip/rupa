# DECISION-043: Pack source locations and derive line/column on demand

- **Status**: Accepted
- **Date**: 2026-11-14
- **Related**: TASK-103, TASK-104, DECISION-042, specification §2.6

## Context

DECISION-042 closed M15–M17 with a 40-byte `Span`, a 16-byte `Position`
(line, column, 64-bit offset), and a 4-byte `FileId`. Those locations appear in
every token and almost every AST node, so they still dominate parsed memory even
after strings were interned and large enum payloads were boxed.

Rupa is a file-scoped document language. A compilation needs far fewer than 256
source files and a practical source file is far smaller than 16 MiB. Line and
column numbers are only needed when a diagnostic is displayed or serialized;
they are not needed by parsing, validation, lowering, or rendering.

## Options considered

| Option | `Span` | Trade-off |
|--------|-------:|-----------|
| Keep `FileId(u32)` + `Position { line, column, offset }` | 40 B | No new limits, but wastes memory on every token and node |
| `FileId(u8)` + offset-only `Position(u32)` | 12 B | Simpler, but still stores the file id separately |
| Pack `FileId(u8)` + 24-bit offset into one `u32` per endpoint | **8 B** | Requires a 256-file limit and a 16 MiB - 1 source limit |
| Remove file id entirely | 8 B | Loses the identifier needed for multi-file diagnostics |

Line/column storage was also reconsidered:

- Storing bookmarks in the AST would save recomputation only at the cost of more
  AST memory.
- Building a `LineIndex` from the source when a diagnostic is rendered or
  serialized keeps the AST minimal and gives exactly the old one-based,
  Unicode-scalar positions. The source is already available at every output
  boundary (CLI, wasm, formatter, diagnostic renderer).

## Decision

1. **`FileId` is a `u8`.** A compilation may reference at most 256 source files,
   indexed `0..=255`. `FileId::new` rejects larger indices.
2. **A source file is at most `2^24 - 1` bytes (16,777,215 bytes, 16 MiB - 1).**
   This bound includes the file length, because a span's end offset can equal the
   length. A larger file is rejected with `L010`; the lexer does not scan it.
3. **`Position` stores only a 24-bit-compatible byte offset.** `Location` packs
   the file id into the high 8 bits and the offset into the low 24 bits. `Span`
   is exactly two packed words: **8 bytes** on all targets.
4. **Line and column are derived, not stored.** `LineIndex` records the byte
   offset of each line start and maps an offset to a one-based line and
   Unicode-scalar column. It is built on demand at diagnostic boundaries.
   `DiagnosticBag::sort` orders by file id and byte offset, which is equivalent
   to source order and avoids needing the index.
5. **Diagnostic/JSON values do not change for valid files.** Human output and
   `check --json` still report one-based line/column and the same start/end
   offsets. The only new diagnostic is `L010` for a file over the source limit.
6. **The public source-location API may change.** Packed fields are accessed via
   accessors (`span.start.offset()`, `span.start_position()`), and
   `Position::new` takes a byte offset.

## Consequences

- `FileId` 4 -> 1 B, `Position` 16 -> 4 B, `Location` 4 B, `Span` 40 -> 8 B.
- `Token` 64 -> 32 B; `Document` 360 -> 200 B; `DocumentChild` 720 -> 304 B;
  `Text` 680 -> 296 B; `LayerDefinition` 368 -> 208 B.
- The repetitive fixture parse keeps 271 allocations but drops from 101,203 to
  **50,387 bytes** (-50.2%); against the original TASK-097 baseline it is
  **66.5% fewer bytes**.
- The wasm release module is 1,377,325 bytes, down from 1,400,685 (-1.7%).
- Lexing no longer maintains line/column counters; diagnostics pay a
  source scan only when they are emitted. Golden PNGs and IR JSON are unchanged.
- Source files larger than 16 MiB - 1 are intentionally unsupported. If this
  becomes restrictive, a future format can widen the offset at the cost of 4
  bytes per span.
- Span serde now represents the packed locations rather than line/column
  objects. AST serde is internal; diagnostic JSON is assembled explicitly and
  remains stable.

## Final verification (TASK-104)

- 556 workspace tests pass; fmt, clippy, workspace build, and wasm release pass.
- 40 alternating release render runs against `7bc951f`: +2.1%, +1.3%, +0.4%,
  and +0.3% for `minimal`, `layers`, `text`, and `paints`; all PNGs identical.
- `check --json` for every invalid fixture and `inspect` JSON for every valid
  example are byte-identical to the M17 build.
- Fixture parse is 271 allocations / 50,387 bytes; wasm is 1,377,325 bytes.
