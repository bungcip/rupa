# TASK-005: Document declaration and validation

## Task Header
- **Task ID**: TASK-005
- **Title**: Document declaration and validation
- **Milestone**: M1 — Minimal Vertical Slice
- **Priority**: Critical
- **Estimated Effort**: 3-4 hours
- **Dependencies**: TASK-004

## Description
Implement semantic validation for document-level constructs in the compiler crate. Validate document declaration, size, background, and export per specification §5.

## Files to Create/Modify

| File | Action | Notes |
|------|--------|-------|
| `crates/rupa-compiler/src/document.rs` | Create | Document validation |
| `crates/rupa-compiler/src/validation.rs` | Create | Validation framework |
| `crates/rupa-compiler/src/error.rs` | Create | Compiler error codes |
| `crates/rupa-compiler/src/lib.rs` | Modify | Export modules |

## Acceptance Criteria

**All must pass to mark task complete:**

- [x] Valid document passes validation
- [x] Exactly one `document` declaration required → error `C001` (missing) / `C002` (multiple), per DECISION-001/002
- [x] `size` required, positive dimensions → error `C005` if missing, `C003` if invalid
- [x] `background` optional; transparent default
- [x] `export` path validated; CLI may override → error `C004` if missing and no CLI arg
- [x] Coordinate system: top-left origin, Y down (documented)
- [x] Dimensions in pixels for first implementation (`C007` for other units)
- [x] Validation errors include stable code, location, suggestion
- [x] Unit tests for all validation rules
- [x] `cargo test -p rupa-compiler` passes
- [x] `cargo fmt -- --check` passes
- [x] `cargo clippy -- -D warnings` passes

## Implementation Steps

1. Define `DocumentValidator` struct
2. Implement validation methods for each rule
3. Define compiler error codes (`R100`–`R199` for document)
4. Collect and report all validation errors (don't stop at first)
5. Write tests for each validation rule

## Testing Requirements

- Valid document → no errors
- No document → `R100`
- Multiple documents → `R100`
- Missing size → `R101`
- Zero/negative width/height → `R102`
- Missing export (no CLI override) → `R103`
- Background omitted → OK (transparent)
- Valid export path → OK

## Specification References

- §5: Document and export
- §2.3: Predictable defaults
- §2.4: Strict but helpful compiler
- §22.3: Diagnostics for iterative repair

## Handoff Notes

**Completed Work**:
- Implemented `DocumentValidator` with `ValidationOptions` (CLI output override).
- Implemented `Validator::validate_source` / `validate_source_with_options`
  covering exactly-one-document, lexer/parser diagnostics, and semantics.
- Added stable codes `C005` (missing size), `C006` (invalid export path),
  `C007` (non-pixel document dimension); corrected duplicate shape validation.
- Added top-level `export` parsing to match specification §5.
- CLI `check`/`render` now use the source-level validator.
- Added 25 compiler unit tests and 1 parser test.
- Recorded policy in `docs/decisions/DECISION-002.md`.

**Remaining Work**:
- None for TASK-005.

**Test Results**:
- `cargo test --workspace`: passes (75 rupa-syntax, 25 rupa-compiler).
- `cargo clippy --all-targets --all-features -- -D warnings`: passes.
- `cargo fmt -- --check`: passes.

**Known Issues**:
- Shape/paint validation wiring remains for TASK-006/TASK-007.

**Exact Next Action**:
- Begin TASK-006: Rectangle and circle primitives.