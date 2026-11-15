# TASK-109: Simplify layer validation

- **Milestone**: Maintenance (after M19)
- **Priority**: Low
- **Status**: Complete
- **Dependencies**: TASK-108
- **Owner**: implementation-agent

## Goal

Reduce duplication and intermediate allocations in
`crates/rupa-compiler/src/layer.rs` without changing validation results.

## Scope

- Use `DiagnosticBag::error_with_help` instead of repeated
  `Diagnostic::error(..).with_help(..)` pushes.
- Validate into the caller's bag instead of building a nested bag and
  extending it.
- Keep the public `validate_layer` entry point and the existing tests.

## Non-Goals

- No diagnostic code/message/span changes, grammar changes, or new deps.
- No changes outside `rupa-compiler` except task/status documentation.

## Acceptance Criteria

- [x] `layer.rs` shrinks while preserving behavior.
- [x] All compiler and workspace tests pass.
- [x] Golden PNGs, `inspect` JSON, and invalid-fixture `check --json` are
      byte-identical to `a3cefc7`.
- [x] `cargo fmt --check`, clippy, workspace build, and workspace tests pass.
- [x] Task/status/current/handoff documentation is updated.
- [x] Commit only after explicit user authorization.

## Result

`layer.rs` fell from **242 to 227 lines** (-15, -6.2%); production code fell
from 127 to 112 lines (-11.8%) while the 114-line test module is unchanged.

Simplifications:

- Six raw `Diagnostic::error(..).with_help(..)` pushes became
  `bag.error_with_help`.
- `validate_layers` now calls a private `validate_plane(layer, &mut bag)`
  directly; the public `validate_layer` wraps the same helper, removing the
  nested bag plus `extend`.
- Removed the now-unused `Diagnostic` import.

Verification:

- 173 `rupa-compiler` tests and 557 workspace tests pass.
- fmt, clippy with `-D warnings`, and workspace build are clean.
- All example PNGs, `inspect` JSON, and invalid-fixture `check --json` outputs
  are byte-identical to `a3cefc7`.

## Handoff Notes

**Completed Work**:
- Simplified layer validation diagnostics and per-layer bag handling.

**Remaining Work**:
- Commit requires explicit user authorization.

**Known Issues**:
- None introduced; diagnostics are byte-identical.

**Exact Next Action**:
- Await commit authorization, then pick the next backlog item.
