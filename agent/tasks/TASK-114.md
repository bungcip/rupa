# TASK-114: Encapsulate `DiagnosticBag` storage

- **Milestone**: Maintenance (after M19)
- **Priority**: Medium
- **Status**: Complete
- **Dependencies**: TASK-113
- **Owner**: implementation-agent

## Goal

Make `DiagnosticBag::diagnostics` private so callers use the bag's methods, and
migrate every in-tree consumer without changing diagnostics, JSON, or images.

## Scope

- Add `len`, `is_empty`, `as_slice`, and `iter` read accessors.
- Implement `IntoIterator` for owned and borrowed bags.
- Implement `FromIterator<Diagnostic>` and `Extend<Diagnostic>`.
- Migrate compiler, CLI, wasm, syntax tests, and internal renderer loops.
- Record the API decision in `docs/decisions/`.

## Non-Goals

- No diagnostic content, ordering, span, JSON, or image changes.
- No performance-oriented storage change; the backing type stays `Vec`.
- No public field removal outside `DiagnosticBag`.

## Acceptance Criteria

- [x] `DiagnosticBag::diagnostics` is private.
- [x] All consumers build/test using the bag API.
- [x] New accessors and collection traits are covered by a test.
- [x] Golden PNGs, `inspect` JSON, and invalid-fixture `check --json` are
      byte-identical to `36d78c4`.
- [x] `cargo fmt --check`, clippy, workspace build, and workspace tests pass.
- [x] Decision, task, status, current, and handoff documentation are updated.
- [x] Commit after explicit user authorization.

## Result

`DiagnosticBag` now stores `diagnostics: Vec<Diagnostic>` privately and exposes
`len`, `is_empty`, `as_slice`, `iter`, `IntoIterator` (owned and borrowed),
`FromIterator`, and `Extend` alongside the existing mutation/filter methods.
Eighteen files across `rupa-syntax`, `rupa-compiler`, `rupa-cli`, and
`rupa-wasm` were migrated.

Verification:

- 132 `rupa-syntax` tests and the full workspace suite pass (558 tests).
- fmt, clippy with `-D warnings`, and workspace build are clean.
- All example PNGs, `inspect` JSON, and invalid-fixture `check --json` outputs
  are byte-identical to `36d78c4`.

## Handoff Notes

**Completed Work**:
- Encapsulated the bag storage, added the collection API, migrated all
  consumers, and recorded DECISION-045.

**Remaining Work**:
- None; committed as `4a94fe9`.

**Known Issues**:
- None introduced; outputs are byte-identical.

**Exact Next Action**:
- Pick the next backlog item.
