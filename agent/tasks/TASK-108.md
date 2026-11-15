# TASK-108: Simplify the lowering implementation

- **Milestone**: Maintenance (after M19)
- **Priority**: Medium
- **Status**: Complete
- **Dependencies**: TASK-107
- **Owner**: implementation-agent

## Goal

Reduce duplication and parameter noise in `crates/rupa-compiler/src/lower.rs`
without changing IR output, diagnostics, or rendered images.

## Scope

- Gather lowering state (`graphics`, diagnostics, functions, PRNG) in a
  `Lowerer` so free functions stop threading the same arguments.
- Move per-node lowering (picture, text, instance, pixel, brush, repeat,
  layer effects, layers, groups) onto that state as methods.
- Share the shape appearance/geometry tail across all primitives.
- Collapse repeated zero-bounds literals and repeated unit/cap/join branches.
- Keep the existing test module unchanged.

## Non-Goals

- No IR, diagnostic code/message, span, JSON, or image changes.
- No grammar/AST changes and no new dependencies.
- No changes outside `rupa-compiler` except task/status documentation.

## Acceptance Criteria

- [x] Lowering source shrinks while preserving behavior.
- [x] All compiler and workspace tests pass.
- [x] Golden PNGs, `inspect` JSON, and invalid-fixture `check --json` are
      byte-identical to `a3cefc7`.
- [x] `cargo fmt --check`, clippy, workspace build, and workspace tests pass.
- [x] Task/status/current/handoff documentation is updated.
- [x] Commit only after explicit user authorization.

## Result

`lower.rs` production code fell from **1,275 to 1,090 lines** (-185, -14.5%);
the 508-line test module is unchanged. The file as a whole fell from 1,783 to
1,598 lines (-10.4%).

Simplifications:

- `Lowerer` owns `graphics`, `bag`, `functions`, and `rng`, so
  `graphics`/`bag`/`functions` are no longer threaded through ~25 functions.
- Shape lowering shares `ShapeProps` plus the `props!` macro and a single
  `Lowerer::shape` tail; each primitive now reads as "require, then geometry".
- `required`, `points`, `stroke`, `fill`, `paint`/`pixel`, `repeat`, `layer`,
  `groups`, and `filter` are state methods instead of free functions.
- Removed the dead second `DocumentChild::Let` arm and the `ShapeAppearance`
  wrapper.
- `ZERO_BOUNDS` replaces four identical `Bounds` literals; cap/join and
  text/size defaults use single-line matches.
- Point lists derive their "two"/"three" wording and help text from the shape.

Verification:

- 173 `rupa-compiler` tests and 557 workspace tests pass.
- fmt, clippy with `-D warnings`, and workspace build are clean.
- All eight example PNGs, their `inspect` JSON, and all invalid-fixture
  `check --json` outputs are byte-identical to `a3cefc7`.

## Handoff Notes

**Completed Work**:
- Refactored lowering around `Lowerer`, shared shape construction, and
  simplified helpers; verified parity and gates.

**Remaining Work**:
- Commit requires explicit user authorization.

**Known Issues**:
- None introduced; IR, diagnostics, and all outputs are byte-identical.

**Exact Next Action**:
- Await commit authorization, then pick the next backlog item.
