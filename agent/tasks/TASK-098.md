# TASK-098: Compact source spans

## Task Header

- **Task ID**: TASK-098
- **Title**: Shrink `Span`, `Position`, and `FileId`
- **Milestone**: M15 — Baseline and compact spans
- **Priority**: High
- **Estimated Effort**: 6–10 hours
- **Dependencies**: TASK-097
- **Status**: Complete

## Description

`Span` is embedded in almost every AST node and token, so its size dominates the
AST footprint. `FileId(usize)` and three `usize` fields per `Position` make a
`Span` much larger than necessary. Shrink the source-location types while
preserving all reported line/column/offset values.

The document still supports large source offsets by keeping `offset` at 64 bits;
line and column counters fit in `u32`. The target is `size_of::<Span>() <= 40`.

## Files to Create/Modify

| File | Action | Notes |
|------|--------|-------|
| `crates/rupa-syntax/src/span.rs` | Modify | Compact field types and constructors |
| `crates/rupa-syntax/src/lexer.rs` | Modify | Update position construction and arithmetic |
| `crates/rupa-syntax/src/parser.rs` | Modify | Update span construction and test helpers |
| `crates/rupa-syntax/src/diagnostic.rs` | Modify | Convert positions at display/serialization boundaries |
| `crates/rupa-syntax/tests/sizes.rs` | Modify | Tighten span bound |
| `crates/rupa-compiler`, `rupa-cli`, `rupa-wasm` tests | Modify | Update direct `Span`/`Position` construction if any |
| `docs/decisions/DECISION-042-compact-ast-memory.md` | Modify | Record final span layout |

## Work Items

1. Change `FileId(usize)` to `FileId(u32)`.
2. Change `Position.line` and `Position.column` to `u32`; keep `offset: u64`.
3. Add checked constructors/helpers so lexer counters cannot silently wrap.
4. Update `Span::new`, `Span::single`, `Span::merge`, and `Display`.
5. Update parser, diagnostic JSON, tests, and any direct field construction.
6. Confirm serialized positions remain numbers with the same values.
7. Tighten the size guardrail from the TASK-097 baseline to `Span <= 40`.

## Acceptance Criteria

**All must pass to mark task complete:**

- [x] `size_of::<Span>() <= 40`.
- [x] No line, column, or byte-offset value changes in diagnostics or JSON.
- [x] No diagnostic codes, messages, or help text change.
- [x] Golden PNGs remain byte-identical.
- [x] `cargo fmt --all -- --check` passes.
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes.
- [x] `cargo build --workspace` passes.
- [x] `cargo test --workspace` passes (541 passed, 0 failed).
- [x] `cargo build -p rupa-wasm --target wasm32-unknown-unknown --release` passes.

## Testing Requirements

- Unit tests: span merge/display, UTF-8 positions, line/column conversion.
- Integration tests: existing diagnostic JSON and CLI span-label tests.
- Golden images: no visual changes expected.

## Specification References

- `rupa-specification.md` §22.3: diagnostic positions and source context.
- `docs/decisions/DECISION-039-diagnostic-span-labels.md`: span labeling behavior.

## Result

| Type | Baseline | Compact |
|------|---------:|--------:|
| `FileId` | 8 | 4 |
| `Position` | 24 | 16 |
| `Span` | 56 | **40** |
| `Token` | 88 | 72 |
| `Document` | 456 | 376 |
| `DocumentChild` | 1456 | 1168 |
| `Text` | 1432 | 1160 |
| `Fill` | 600 | 504 |
| `Expression` | 200 | 152 |
| `GroupDefinition` | 304 | 240 |
| `LayerDefinition` | 992 | 832 |

The fixture parse is now 381 allocations / 123,097 bytes (was 150,297 bytes).
`Position::new`, `Position::from_usize`, and `Position::offset_usize` provide
checked host boundaries; lexer `u32` counters saturate rather than wrap.
Serialized positions remain the same JSON numbers.

## Handoff Notes

**Completed Work**:
- `FileId` is `u32`; `Position` is `line: u32`, `column: u32`, `offset: u64`;
  `Span` is 40 bytes.
- Updated lexer counters and constructors, parser fallback spans, formatter
  offset slicing, diagnostic rendering conversion, and compiler test helpers.
- Added span merge/display/layout tests and a UTF-8 lexer span test.
- Tightened `Span`/`Position`/`FileId` guardrails and updated DECISION-042.

**Remaining Work**:
- TASK-099 migrates identifiers to `IStr`; registry is online, so add `ustr` to
  the workspace and `rupa-syntax` as recorded in DECISION-042.
- TASK-100 migrates AST string fields and removes duplicate `Text.content`.

**Test Results**:
- `cargo fmt --all -- --check`: clean.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: clean.
- `cargo build --workspace`: OK.
- `cargo test --workspace`: 541 passed, 0 failed.
- `cargo build -p rupa-wasm --target wasm32-unknown-unknown --release`: OK
  (1,396,128 bytes).
- `cargo test -p rupa-syntax --test sizes -- --nocapture`: 3 passed.
- `cargo test -p rupa-syntax --test allocations -- --nocapture`: 1 passed.

**Known Issues**:
- `u32` line/column counters saturate at `u32::MAX`; this is documented and
  unreachable for practical source files.
- The wasm release module grew by 8,072 bytes versus TASK-097; TASK-102 will
  verify the final budget.

**Exact Next Action**:
- Implement TASK-099: add `ustr` with serde support, introduce `IStr`, intern
  lexer identifiers, and keep string literals as `Box<str>`.
