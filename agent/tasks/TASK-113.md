# TASK-113: Simplify the remaining compiler modules

- **Milestone**: Maintenance (after M19)
- **Priority**: Low
- **Status**: Complete
- **Dependencies**: TASK-112
- **Owner**: implementation-agent

## Goal

Finish the compiler-crate simplification pass on `validation.rs`, `error.rs`,
and `metadata.rs` without changing validation results or public behavior.

## Scope

- Replace `bag.diagnostics.extend(..)` with `bag.append(..)` and raw diagnostic
  literals with `DiagnosticBag::error_with_help` in `validation.rs`.
- Shorten fully-qualified `rupa_syntax`/`std` paths with imports.
- Avoid cloning the expression-error message.
- Build `CompilerError` diagnostics via `Diagnostic::error` in `error.rs`.
- Use the `serde_json::Map` alias in `metadata.rs`.

## Non-Goals

- No diagnostic code/message/span ordering changes, IR changes, or new deps.
- No public API removal; `lib.rs`, alias modules, and `test_support.rs` are
  already minimal and unchanged.
- No changes outside `rupa-compiler` except task/status documentation.

## Acceptance Criteria

- [x] The remaining modules shrink or stay minimal while preserving behavior.
- [x] All compiler and workspace tests pass.
- [x] Golden PNGs, `inspect` JSON, and invalid-fixture `check --json` are
      byte-identical to `a3cefc7`.
- [x] `cargo fmt --check`, clippy, workspace build, and workspace tests pass.
- [x] Task/status/current/handoff documentation is updated.
- [x] Commit only after explicit user authorization.

## Result

| File | Before | After | Delta | Production |
|------|-------:|------:|------:|-----------:|
| `validation.rs` | 638 | 593 | -45 (-7.1%) | 414 → 369 |
| `error.rs` | 76 | 70 | -6 (-7.9%) | 76 → 70 |
| `metadata.rs` | 109 | 109 | 0 | unchanged |
| `lib.rs` / aliases / `test_support.rs` | minimal | minimal | 0 | unchanged |

Test modules are unchanged.

Simplifications:

- `validation.rs`: every validator is appended via `bag.append`; all raw
  `Diagnostic` pushes use `error_with_help`; imports replace repeated
  `rupa_syntax::ast::*`, `std::collections::*`, and full `Span` paths;
  `expression_diagnostic` builds help before moving the message, removing a
  clone; pixel/brush fill handling uses `match`.
- `error.rs`: `to_diagnostic` uses `Diagnostic::error` plus the existing help;
  `Vec<CompilerError>` conversion maps and extends in one step.
- `metadata.rs`: uses the `Map` import; no behavior change.

Verification:

- 173 `rupa-compiler` tests and 557 workspace tests pass.
- fmt, clippy with `-D warnings`, and workspace build are clean.
- All example PNGs, `inspect` JSON, and invalid-fixture `check --json` outputs
  are byte-identical to `a3cefc7`.

## Handoff Notes

**Completed Work**:
- Completed the compiler-crate simplification pass across validation, error, and
  metadata modules.

**Remaining Work**:
- Commit requires explicit user authorization.

**Known Issues**:
- None introduced; outputs and diagnostics are byte-identical.

**Exact Next Action**:
- Await commit authorization, then pick the next backlog item.
