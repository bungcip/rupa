# TASK-100: Migrate AST string fields

## Task Header

- **Task ID**: TASK-100
- **Title**: Use `IStr` for names and `Box<str>` for unique payloads
- **Milestone**: M16 — Interned identifiers
- **Priority**: High
- **Estimated Effort**: 10–14 hours
- **Dependencies**: TASK-099
- **Status**: Complete

## Description

Apply the compact string representation to AST fields. Identifier-like values
become `IStr`; large unique payloads become `Box<str>`. Remove the duplicated
`Text.content` field if it only mirrors `Text.text`. Preserve the public JSON
shape, diagnostic text, compiler behavior, and rendered output.

## Files to Create/Modify

| File | Action | Notes |
|------|--------|-------|
| `crates/rupa-syntax/src/ast.rs` | Modify | Migrate string fields; remove duplicate text field |
| `crates/rupa-syntax/src/parser.rs` | Modify | Construct `IStr`/`Box<str>` values |
| `crates/rupa-syntax/src/formatter.rs` | Modify | Update any AST/token field access |
| `crates/rupa-compiler/src/*.rs` | Modify | Use `.as_str()` at compiler boundaries |
| `crates/rupa-cli` | Modify | Update inspect/JSON/test call sites if needed |
| `crates/rupa-wasm` | Modify | Update call sites if needed |
| `crates/rupa-syntax/tests/sizes.rs` | Modify | Tighten AST size guards |
| `crates/rupa-syntax/tests/allocations.rs` | Modify | Record allocation improvement |

## Field Migration

Interned with `IStr`:

- `GroupDefinition.name`
- `LayerDefinition.name`, `LayerDefinition.blend`
- `GroupInstance.name`, `GroupInstance.blend`
- `LayerEffect.kind`
- `Text.font`
- `Function.name`, `Function.params`
- `Variable.name`
- `Repeat.index`
- `Color::Named`, `Color::Hex`
- `PatternFill.kind`
- `Expression::Identifier`
- `Expression::Call.name`

Stored as `Box<str>`:

- `Text.text`
- `Picture.path`
- `LayerEffect.resource`
- `Expression::String`
- `Value::String`

Remove `Text.content` when confirmed redundant. If it is still required,
keep exactly one string field plus an accessor.

## Work Items

1. Change the AST field types.
2. Update parser construction and all pattern matches.
3. Update compiler environment maps and function tables to avoid unnecessary
   `String` allocation; use `IStr` keys where practical, otherwise `.as_str()`.
4. Preserve `inspect`/diagnostic JSON values as plain strings.
5. Add a regression test proving `Text` stores content once.
6. Compare `size_of` for `Text`, `GroupDefinition`, `LayerDefinition`, and
   `DocumentChild` against TASK-097.
7. Run the allocation fixture and record the improvement.

## Acceptance Criteria

**All must pass to mark task complete:**

- [x] No identifier/name field in the AST uses `String`.
- [x] Unique payloads use `Box<str>`; no large unique string is globally interned.
- [x] `Text` stores one content string.
- [x] Diagnostic messages, codes, spans, and JSON field values are unchanged.
- [x] Golden PNGs remain byte-identical.
- [x] `size_of::<Text>()`, `LayerDefinition`, and `GroupDefinition` are lower
      than baseline or the reason is documented.
- [x] `cargo fmt --all -- --check` passes.
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes.
- [x] `cargo build --workspace` passes.
- [x] `cargo test --workspace` passes (547 passed, 0 failed).
- [x] `cargo build -p rupa-wasm --target wasm32-unknown-unknown --release` passes.

## Testing Requirements

- Unit tests: parser construction, text content, name lookup, serde roundtrip.
- Integration tests: `rupa check --json`, inspect output, group/layer placement.
- Golden images: `tests/golden/*.png` unchanged.

## Specification References

- `rupa-specification.md` §7–§10: primitives, text, pictures.
- `rupa-specification.md` §22.3: diagnostic JSON.
- `DECISION-042`: interning and payload policy.

## Result

| Type | TASK-097 baseline | TASK-100 |
|------|------------------:|---------:|
| `Document` | 456 | 360 |
| `DocumentChild` | 1456 | 1056 |
| `Text` | 1432 | 1016 |
| `Fill` | 600 | 408 |
| `Expression` | 200 | 152 |
| `GroupDefinition` | 304 | 224 |
| `LayerDefinition` | 992 | 704 |

After intern-table warm-up the fixture parse is **278 allocations / 109,651
bytes**, down from 304 / 114,327 in TASK-099.

All AST identifier/name fields use `IStr`; unique payloads use `Box<str>`.
`Text` stores its content once and serializes only a `text` key. Compiler scopes
and function tables use `IStr` keys, while IR fields remain `String` with one
conversion at the lowering boundary. Diagnostic values, JSON, and golden images
are unchanged.

## Handoff Notes

**Completed Work**:
- Migrated AST names/identifiers to `IStr`; unique payloads to `Box<str>`.
- Removed duplicate `Text.content` and updated parser/compiler consumers.
- Moved compiler scopes and function tables to `IStr` keys.
- Added `tests/ast_strings.rs` (single text field, plain-string name JSON, serde
  roundtrip), tightened size guardrails, and updated allocation baseline.
- Recorded results in DECISION-042.

**Remaining Work**:
- TASK-101 measures and boxes large enum variants.
- TASK-102 verifies final memory/perf/wasm/diagnostic/golden behavior and closes
  M17.

**Test Results**:
- `cargo fmt --all -- --check`: clean.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: clean.
- `cargo build --workspace`: OK.
- `cargo test --workspace`: 547 passed, 0 failed.
- `cargo build -p rupa-wasm --target wasm32-unknown-unknown --release`: OK
  (1,388,244 bytes).
- `cargo test -p rupa-syntax --test ast_strings`: 2 passed.
- `cargo test -p rupa-syntax --test sizes -- --nocapture`: 3 passed.
- `cargo test -p rupa-syntax --test allocations -- --nocapture`: 1 passed.

**Known Issues**:
- IR layer/group/text/picture fields still use `String`; converting IR is out of
  scope for M16 and TASK-102 will verify the remaining layout.
- The intern table retains identifiers for process lifetime by design.

**Exact Next Action**:
- Implement TASK-101: measure enum variants and box only the variants whose size
  reduction justifies the extra allocation.
