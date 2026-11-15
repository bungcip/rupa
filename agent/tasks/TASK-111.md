# TASK-111: Simplify group, document, and paint validation

- **Milestone**: Maintenance (after M19)
- **Priority**: Low
- **Status**: Complete
- **Dependencies**: TASK-110
- **Owner**: implementation-agent

## Goal

Reduce duplication, raw diagnostic literals, and redundant branches in
`group.rs`, `document.rs`, and `paint.rs` without changing validation or paint
lowering results.

## Scope

- Use `DiagnosticBag::error_with_help` / `DiagnosticBag::error` and
  `DiagnosticBag::append` throughout `group.rs` and `document.rs`.
- Simplify `document.rs` by folding `validate_size` into `validate`.
- In `paint.rs`, replace raw `Diagnostic` structs, use `map_err(bag_with)?`,
  collapse duplicated unit branches, and derive named-color suggestions from
  `NAMED_COLORS` instead of a second list.

## Non-Goals

- No diagnostic code/message/span/help/edit changes, IR changes, or new deps.
- No changes outside `rupa-compiler` except task/status documentation.

## Acceptance Criteria

- [x] All three files shrink while preserving behavior.
- [x] All compiler and workspace tests pass.
- [x] Golden PNGs, `inspect` JSON, and invalid-fixture `check --json` are
      byte-identical to `a3cefc7`.
- [x] `cargo fmt --check`, clippy, workspace build, and workspace tests pass.
- [x] Task/status/current/handoff documentation is updated.
- [x] Commit only after explicit user authorization.

## Result

| File | Before | After | Delta | Production |
|------|-------:|------:|------:|-----------:|
| `group.rs` | 458 | 435 | -23 (-5.0%) | 319 → 296 |
| `document.rs` | 201 | 191 | -10 (-5.0%) | 69 → 59 |
| `paint.rs` | 494 | 424 | -70 (-14.2%) | 324 → 254 |
| **Total** | **1,153** | **1,050** | **-103 (-8.9%)** | **712 → 609 (-14.5%)** |

Test modules are unchanged.

Simplifications:

- `group.rs`: nine raw `Diagnostic::error(..).with_help(..)` pushes became
  `bag.error_with_help`; two `extend(..diagnostics)` calls became `bag.append`;
  the unknown-group suggestion now builds its help up front.
- `document.rs`: `validate_size` folded into `validate`; both dimension
  diagnostics use `bag.error_with_help`.
- `paint.rs`: four raw `Diagnostic { .. }` literals became
  `Diagnostic::error(..).with_help(..)` / `bag.error`; five
  parse-and-return matches became `map_err(bag_with)?`; identical `Unit` match
  arms collapsed to `d.value as f32`; `NAMED_COLOR_NAMES` deleted and derived
  from `NAMED_COLORS`; `lower_fill`/`lower_fill_to_store` use `map_or`/`map`;
  single-stop gradients use slice matching.

Verification:

- 173 `rupa-compiler` tests and 557 workspace tests pass.
- fmt, clippy with `-D warnings`, and workspace build are clean.
- All example PNGs, `inspect` JSON, and invalid-fixture `check --json` outputs
  are byte-identical to `a3cefc7`.

## Handoff Notes

**Completed Work**:
- Simplified diagnostics and control flow in the group, document, and paint
  validators.

**Remaining Work**:
- Commit requires explicit user authorization.

**Known Issues**:
- None introduced; outputs and diagnostics are byte-identical.

**Exact Next Action**:
- Await commit authorization, then pick the next backlog item.
