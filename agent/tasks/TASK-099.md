# TASK-099: Interned strings and token migration

## Task Header

- **Task ID**: TASK-099
- **Title**: Add `IStr` and migrate lexer tokens
- **Milestone**: M16 — Interned identifiers
- **Priority**: High
- **Estimated Effort**: 8–12 hours
- **Dependencies**: TASK-097, TASK-098
- **Status**: Complete

## Description

Identifiers and repeated names are currently stored as `String`, paying a
24-byte field plus one heap allocation per occurrence. Introduce a compact
interned string handle in `rupa-syntax` and use it for lexer identifiers. Keep
serde output as ordinary JSON strings and keep string literals that are unique
payloads out of the global intern table.

`IStr` is a newtype over the backend selected in TASK-097. The preferred backend
is `ustr`; the documented fallback is an in-house `Arc<str>` interner.

## Files to Create/Modify

| File | Action | Notes |
|------|--------|-------|
| `Cargo.toml` | Modify | Add `ustr` to workspace dependencies if selected |
| `crates/rupa-syntax/Cargo.toml` | Modify | Add `ustr` with serde support if needed |
| `crates/rupa-syntax/src/intern.rs` | Create | `IStr` wrapper, conversions, serde, tests |
| `crates/rupa-syntax/src/lib.rs` | Modify | Export `IStr` |
| `crates/rupa-syntax/src/token.rs` | Modify | `Identifier(IStr)`, string literal representation |
| `crates/rupa-syntax/src/lexer.rs` | Modify | Intern identifiers; keep payload literals uninterned |
| `crates/rupa-syntax/src/parser.rs` | Modify | Adapt token pattern matching |
| `crates/rupa-syntax/tests/sizes.rs` | Modify | Assert `IStr` size |
| `docs/decisions/DECISION-042-compact-ast-memory.md` | Modify | Record backend and memory-retention trade-off |

## Work Items

1. Implement `IStr` with `Clone`, `Copy` where supported, `PartialEq`, `Eq`,
   `Hash`, `Display`, `Debug`, `Deref<Target = str>`, `AsRef<str>`, and
   `Borrow<str>`.
2. Implement `Serialize`/`Deserialize` as a plain string without requiring the
   backend's serde feature when possible.
3. Intern `TokenKind::Identifier` through `IStr`.
4. Store string literals as `Box<str>` to avoid retaining large unique payloads
   globally.
5. Add tests for deduplication, equality, display/debug, serde roundtrip, and
   size.
6. Verify the lexer no longer allocates a new `String` per identifier.
7. Build for wasm32 and confirm the handle size and serialization behavior.

## Acceptance Criteria

**All must pass to mark task complete:**

- [x] With `ustr`, `size_of::<IStr>() <= 8` on native and `<= 4` on wasm32.
- [x] Equal identifiers share one interned string allocation.
- [x] `IStr` serializes/deserializes as a JSON string.
- [x] Tokenization and all diagnostic spans are unchanged.
- [x] String literals use `Box<str>` and are not globally interned.
- [x] `cargo fmt --all -- --check` passes.
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes.
- [x] `cargo build --workspace` passes.
- [x] `cargo test --workspace` passes (545 passed, 0 failed).
- [x] `cargo build -p rupa-wasm --target wasm32-unknown-unknown --release` passes.

## Testing Requirements

- Unit tests: `IStr` conversions, dedup, serde roundtrip, size.
- Integration tests: lexer token parity on existing valid/invalid sources.
- Golden images: unchanged.

## Specification References

- `rupa-specification.md` §3: lexical structure.
- `rupa-specification.md` §22.3: diagnostic serialization.

## Result

- Added `ustr = "1.1"` as a workspace dependency and to `rupa-syntax`; serde is
  implemented manually on `IStr`, so the backend's `serde` feature is not used.
- `IStr` is 8 bytes on native and has a compile-time `<= 4` bound for 32-bit
  targets, checked by the wasm release build.
- `TokenKind::Identifier` now stores `IStr`; `TokenKind::String` stores
  `Box<str>`. `TokenKind` fell from 32 to 24 bytes and `Token` from 72 to 64.
- `scan_identifier` slices the source instead of building a `String`; equal
  identifiers share one interned payload.
- String literals remain uninterned; a lexer test verifies equal literals keep
  distinct allocations.
- After intern-table warm-up, the fixture parse is **304 allocations /
  114,327 bytes** (from 381 / 123,097).
- Diagnostic values, token spans, JSON, and golden images are unchanged.

## Handoff Notes

**Completed Work**:
- Added `crates/rupa-syntax/src/intern.rs` with conversions, formatting,
  borrowed access, plain-string serde, dedup tests, and size guardrails.
- Exported `IStr` from `rupa-syntax`; migrated lexer identifier and string
  tokens; adapted parser matches to keep AST fields as `String` until TASK-100.
- Updated size/allocation guardrails and DECISION-042 with the `ustr` decision,
  memory-retention trade-off, and measured results.

**Remaining Work**:
- TASK-100 migrates AST identifier/name fields to `IStr`, switches unique
  payloads to `Box<str>`, and removes duplicate `Text.content`.
- TASK-101/TASK-102 follow in dependency order.

**Test Results**:
- `cargo fmt --all -- --check`: clean.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: clean.
- `cargo build --workspace`: OK.
- `cargo test --workspace`: 545 passed, 0 failed.
- `cargo build -p rupa-wasm --target wasm32-unknown-unknown --release`: OK
  (1,391,884 bytes).
- `cargo test -p rupa-syntax --test sizes -- --nocapture`: 3 passed; `IStr` 8 B.
- `cargo test -p rupa-syntax --test allocations -- --nocapture`: 1 passed.

**Known Issues**:
- The intern table retains identifiers for process lifetime by design; unique
  payloads are excluded. An `IStr` fallback via `Arc<str>` is documented but not
  compiled in because `ustr` is available.
- AST string fields still use `String` until TASK-100, so parser-side allocation
  remains for now.

**Exact Next Action**:
- Implement TASK-100: migrate AST string fields to `IStr`, use `Box<str>` for
  unique payloads, and remove the duplicate `Text.content` field.
