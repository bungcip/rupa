# TASK-072: Final migration — formatter, inspect, language-reference, goldens, end-to-end verification

## Task Header

- **Task ID**: TASK-072
- **Title**: Migrate examples, fixtures, formatter, language-reference JSON,
  inspect, and goldens to the group model; end-to-end verification
- **Milestone**: M10 — Groups, defaults, and CLI output
- **Priority**: Critical
- **Estimated Effort**: 3–4 hours
- **Dependencies**: TASK-068, TASK-069, TASK-070, TASK-071

---

## Description

Close out Milestone 10 by migrating the remaining surfaces to the group/layer
model and running the final end-to-end verification:

- The token-based formatter preserves `group`, `layer`, and `place`
  declarations and remains idempotent. Unary-minus spacing is fixed so
  `rotate -8deg`, `z-index -1`, and `scale -0.5` stay attached while a
  binary `-` keeps its spaces.
- `rupa inspect` serializes groups, layers, and the implicit root layer,
  including layer planes and group instances.
- `docs/language-reference.json` describes the group and layer models
  (properties, children, placement, clipping, single-use planes).
- Golden PNGs `tests/golden/layers.png` and `tests/golden/layer-fill.png`
  cover group placement and layer background fill; integration tests compare
  `examples/layers.rupa` and `examples/layer-fill.rupa` against them.
- Full validation (`fmt`, `clippy`, `build`, `test`) passes and closes
  Milestone 10.

---

## Files Created/Modified

| File | Action | Notes |
|------|--------|-------|
| `crates/rupa-syntax/src/formatter.rs` | Modify | Preserve group/layer/place declarations; unary-minus spacing; tests |
| `crates/rupa-cli/tests/cli.rs` | Modify | `inspect_reports_groups_and_layer_planes` |
| `crates/rupa-cli/tests/integration_test.rs` | Modify | Golden tests for `layers.png` and `layer-fill.png` |
| `docs/language-reference.json` | Modify | Group/layer model details |
| `tests/golden/layers.png` | Create | Golden reference image |
| `tests/golden/layer-fill.png` | Create | Golden reference image |
| `agent/tasks/TASK-072.md` | Create | This document |
| `agent/handoffs/HANDOFF-2026-11-12-TASK-072.md` | Create | Handoff note |
| `agent/STATUS.md`, `agent/CURRENT.md`, `agent/BACKLOG.md` | Modify | Mark TASK-072/M10 complete |

---

## Acceptance Criteria

- [x] The formatter preserves and round-trips `group`, `layer`, and `place`
      declarations
- [x] `rupa fmt` keeps unary minus attached to its operand
- [x] `rupa inspect` reports groups, layers, root children, and layer planes
- [x] `docs/language-reference.json` describes group and layer models
- [x] `examples/layers.rupa` matches `tests/golden/layers.png`
- [x] `examples/layer-fill.rupa` matches `tests/golden/layer-fill.png`
- [x] All checked-in examples and benchmarks compile and render
- [x] `cargo fmt -- --check` passes
- [x] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [x] `cargo build --workspace` passes
- [x] `cargo test --workspace` passes (501 tests, 0 failures)

---

## Specification References

- §6.0–§6.9 Group and layer model
- §21 CLI (`inspect`, `fmt`)
- §22.2 Executable documentation and golden images

---

## Handoff Notes

**Completed Work**:
- Formatter preserves group/layer/place declarations and fixes unary-minus
  spacing; round-trip tests pass.
- Inspect serialization verified for groups, layers, and layer planes.
- Language-reference JSON enriched for groups and layers.
- Golden PNGs for `layers` and `layer-fill` generated and compared.
- Full validation passes: fmt clean, clippy clean, build OK, 501 tests, 0
  failures.

**Remaining Work**:
- None. Milestone 10 is closed.

**Known Issues**:
- The formatter still collapses blank lines (lexer skips them); this is
  existing canonical behavior, not a regression.
- `Cargo.toml` retains an unrelated pre-existing `resolver = "2"` → `"3"`
  edit that must stay unstaged.

**Exact Next Action**:
- Commit TASK-072, excluding `Cargo.toml`.
