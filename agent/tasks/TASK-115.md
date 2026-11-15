# TASK-115: Use `stats_alloc` for the parse allocation baseline

- **Milestone**: Maintenance (after M19)
- **Priority**: Low
- **Status**: Complete
- **Dependencies**: TASK-114
- **Owner**: implementation-agent

## Goal

Replace the hand-written counting global allocator in
`crates/rupa-syntax/tests/allocations.rs` with the `stats_alloc` crate and
refresh the regression baseline for the new metric.

## Scope

- Add `stats_alloc` as a workspace/`rupa-syntax` dev-dependency.
- Instrument the global allocator with `StatsAlloc<System>` and measure the
  parse region with `Region`.
- Keep the warm-up parse, determinism check, and allocation/byte guardrails.
- Update baseline bounds and comments for `stats_alloc` accounting.

## Non-Goals

- No changes to parser/lexer/AST behavior or public APIs.
- No runtime dependency; `stats_alloc` is dev-only.
- No changes outside the test, manifests, and status docs.

## Acceptance Criteria

- [x] The custom `GlobalAlloc` implementation is removed.
- [x] The test measures with `stats_alloc` and remains deterministic.
- [x] Baseline thresholds reflect the measured `stats_alloc` values.
- [x] `cargo fmt --check`, clippy, workspace build, and workspace tests pass.
- [x] Task/status/current/handoff documentation is updated.
- [x] Commit after explicit user authorization.

## Result

`allocations.rs` fell from **126 to 80 lines** (-46, -36.5%). The test now uses
`stats_alloc 0.1.10` (`StatsAlloc<System>`, `Region`) instead of a custom
allocator. The measured baseline is **222 allocations / 32_477 bytes**;
guardrails are 230 allocations and 34_000 bytes. The previous custom counter
reported 271 / 50_387 because it also counted reallocations as allocations.

Verification:

- 558 workspace tests pass (132 in `rupa-syntax`).
- fmt, clippy with `-D warnings`, and workspace build are clean.

## Handoff Notes

**Completed Work**:
- Migrated the allocation baseline to `stats_alloc`, updated bounds, and
  recorded TASK-115.

**Remaining Work**:
- None; committed as `414d34b`.

**Known Issues**:
- None. `stats_alloc` is dev-only and does not ship in the library.

**Exact Next Action**:
- Pick the next backlog item.
