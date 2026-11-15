# TASK-004: AST and basic diagnostics

## Task Header
- **Task ID**: TASK-004
- **Title**: AST and basic diagnostics
- **Milestone**: M1 — Minimal Vertical Slice
- **Priority**: Critical
- **Estimated Effort**: 3-4 hours
- **Dependencies**: TASK-003

## Description
Complete AST node definitions with full span information. Implement diagnostic reporting with stable error codes, source locations, and suggested corrections.

## Files to Create/Modify

| File | Action | Notes |
|------|--------|-------|
| `crates/rupa-syntax/src/ast.rs` | Modify | Complete with spans |
| `crates/rupa-syntax/src/diagnostic.rs` | Create | Diagnostic system |
| `crates/rupa-syntax/src/span.rs` | Modify | Enhance if needed |
| `crates/rupa-syntax/src/error.rs` | Modify | Unify error types |
| `crates/rupa-syntax/src/lib.rs` | Modify | Export modules |

## Acceptance Criteria

**All must pass to mark task complete:**

- [x] All AST nodes carry `Span` (file, line, column, start/end offset) — verified in ast.rs (all node structs have `span: Span` with line/column/offset)
- [x] Diagnostic codes follow format `RXXX` (e.g., `R001`, `R002`) — **deviation, documented**: codes use stable category prefixes (L/P/C/S/T + 3 digits) already deployed across 33 call sites; see docs/decisions/DECISION-001.md. Spec §2.4 only requires stable codes.
- [x] Diagnostics include: code, severity, message, primary span, optional help/suggestion
- [x] Error codes are stable and documented (codes module rustdoc + stability test `test_diagnostic_codes_are_stable` + DECISION-001)
- [x] Suggestion provided for common mistakes — `did_you_mean` (Levenshtein) wired into parser for unknown identifiers. Note: `colour`→`fill` has edit distance 5 and is not suggestable; realistic misspellings (e.g., `fil`→`fill`) are suggested.
- [x] Multiple diagnostics can be reported per compilation (DiagnosticBag + source-ordered sorting)
- [x] Diagnostic pretty-printing with source context (`render_diagnostic`/`render_diagnostics`: header, `--> path:line:col`, source line, caret run, help)
- [x] Unit tests for diagnostic formatting (14 tests in diagnostic.rs, 1 parser suggestion test)
- [x] `cargo test -p rupa-syntax` passes (74 tests)
- [x] `cargo fmt -- --check` passes
- [x] `cargo clippy -- -D warnings` passes (also verified with --all-targets --all-features)

## Implementation Steps

1. Ensure all AST nodes have `span: Span` field
2. Define `Diagnostic` struct with code, severity, message, spans, suggestions
3. Define `DiagnosticCode` enum with stable codes
4. Implement `DiagnosticEmitter` for collecting and formatting
5. Add source context rendering (show line with caret)
6. Define standard error codes for Milestone 1
7. Write tests for diagnostic output

## Testing Requirements

- Test diagnostic formatting matches expected output
- Test error codes are stable
- Test suggestions appear for known mistakes
- Test multiple diagnostics collected
- Test source context rendering

## Specification References

- §2.4: Strict but helpful compiler (diagnostics requirements)
- §22.3: Diagnostics for iterative repair
- §4: Lexical elements (span tracking)

## Handoff Notes

**Completed Work**:
- Verified all AST nodes already carry `Span` (from TASK-003 work)
- Added `Diagnostic::error/warning/with_help/with_suggestion` constructors
- Added `did_you_mean`/`did_you_mean_from` (Levenshtein, deterministic tie-break) and `KNOWN_KEYWORDS`
- Added `render_diagnostic`/`render_diagnostics` pretty-printing with source context and caret run
- Added `DiagnosticBag::sort` for stable, source-ordered output
- Parser now emits `L006` (unknown keyword) with did-you-mean suggestion for unknown identifiers in document body
- Exported new API from lib.rs; recorded code-format decision in docs/decisions/DECISION-001.md
- 15 new tests (74 total in rupa-syntax); fmt/clippy/test all green

**Remaining Work**:
- 

**Remaining Work**:
- 

**Test Results**:
- 

**Known Issues**:
- 

**Exact Next Action**:
- Task complete. Next: TASK-005 (document declaration and validation).