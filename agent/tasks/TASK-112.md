# TASK-112: Simplify shape validation

- **Milestone**: Maintenance (after M19)
- **Priority**: Low
- **Status**: Complete
- **Dependencies**: TASK-111
- **Owner**: implementation-agent

## Goal

Reduce duplication and raw diagnostic literals in
`crates/rupa-compiler/src/shapes.rs` without changing validation results.

## Scope

- Add a `new_bag` helper for the transform-validation preamble.
- Share axis-size validation between `size` properties and picture frames.
- Reuse the optional-width validator from stroke and path validation.
- Replace raw `Diagnostic::error(..).with_help(..)` pushes with
  `DiagnosticBag::error_with_help`.

## Non-Goals

- No diagnostic code/message/span/help changes, grammar changes, or new deps.
- No changes outside `rupa-compiler` except task/status documentation.

## Acceptance Criteria

- [x] `shapes.rs` shrinks while preserving behavior.
- [x] All compiler and workspace tests pass.
- [x] Golden PNGs, `inspect` JSON, and invalid-fixture `check --json` are
      byte-identical to `a3cefc7`.
- [x] `cargo fmt --check`, clippy, workspace build, and workspace tests pass.
- [x] Task/status/current/handoff documentation is updated.
- [x] Commit only after explicit user authorization.

## Result

`shapes.rs` fell from **1,131 to 1,052 lines** (-79, -7.0%); production code
fell from 624 to 545 lines (-12.7%) while the 507-line test module is unchanged.

Simplifications:

- `new_bag(transform, shape)` replaces eleven copies of "new bag + extend
  transform diagnostics" and now uses `append`.
- `validate_size_axes` shares width/height checks between `validate_size_property`
  and picture frame sizes.
- `validate_stroke` delegates its width check to `validate_optional_width`;
  path width uses the same helper.
- All remaining raw `Diagnostic` pushes became `bag.error_with_help`; redundant
  `.to_string()` calls were dropped.
- Signatures use the imported `Span` alias instead of full paths.

Verification:

- 173 `rupa-compiler` tests and 557 workspace tests pass.
- fmt, clippy with `-D warnings`, and workspace build are clean.
- All example PNGs, `inspect` JSON, and invalid-fixture `check --json` outputs
  are byte-identical to `a3cefc7`.

## Handoff Notes

**Completed Work**:
- Simplified shape validation bag setup, size-axis checks, width checks, and
  diagnostic construction.

**Remaining Work**:
- Commit requires explicit user authorization.

**Known Issues**:
- None introduced; outputs and diagnostics are byte-identical.

**Exact Next Action**:
- Await commit authorization, then pick the next backlog item.
