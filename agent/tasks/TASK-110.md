# TASK-110: Simplify the front-end pipeline

- **Milestone**: Maintenance (after M19)
- **Priority**: Low
- **Status**: Complete
- **Dependencies**: TASK-109
- **Owner**: implementation-agent

## Goal

Reduce nesting and repeated bag plumbing in
`crates/rupa-compiler/src/compile.rs` without changing compile results.

## Scope

- Use `DiagnosticBag::append` instead of extending inner diagnostic vectors.
- Replace the nested document/validation/lowering `match` with a single
  `and_then` pipeline.
- Merge the `rupa_syntax` imports.

## Non-Goals

- No diagnostic, IR, AST, or output changes; no new dependencies.
- No changes outside `rupa-compiler` except task/status documentation.

## Acceptance Criteria

- [x] `compile.rs` shrinks while preserving behavior.
- [x] All compiler and workspace tests pass.
- [x] Golden PNGs, `inspect` JSON, and invalid-fixture `check --json` are
      byte-identical to `a3cefc7`.
- [x] `cargo fmt --check`, clippy, workspace build, and workspace tests pass.
- [x] Task/status/current/handoff documentation is updated.
- [x] Commit only after explicit user authorization.

## Result

`compile.rs` fell from **179 to 168 lines** (-11, -6.1%); production code fell
from 101 to 90 lines (-10.9%) while the 78-line test module is unchanged.

Simplifications:

- Both diagnostic merges now use `DiagnosticBag::append` instead of manually
  extending `.diagnostics`.
- The nested `match &parsed.document` / `if` / `match lower_document` block is
  now one `as_ref().and_then(..)` pipeline with an early `return None`.
- Consolidated the `rupa_syntax` imports.

Verification:

- 173 `rupa-compiler` tests and 557 workspace tests pass.
- fmt, clippy with `-D warnings`, and workspace build are clean.
- All example PNGs, `inspect` JSON, and invalid-fixture `check --json` outputs
  are byte-identical to `a3cefc7`.

## Handoff Notes

**Completed Work**:
- Simplified compile-source bag merging and document/lowering control flow.

**Remaining Work**:
- Commit requires explicit user authorization.

**Known Issues**:
- None introduced; outputs and diagnostics are byte-identical.

**Exact Next Action**:
- Await commit authorization, then pick the next backlog item.
