# TASK-097: Baseline AST size, allocations, and interning spike

## Task Header

- **Task ID**: TASK-097
- **Title**: Measure the baseline and select the interning backend
- **Milestone**: M15 — Baseline and compact spans
- **Priority**: High
- **Estimated Effort**: 4–6 hours
- **Dependencies**: None
- **Status**: Complete

## Description

Establish measurable targets before changing AST representations. The audit
showed that `Span`, `Position`, `DocumentChild`, and repeated string fields are
the main contributors to AST memory. This task records the baseline, adds
regression guardrails, and validates the proposed interning dependency so later
tasks can proceed without design drift.

## Files to Create/Modify

| File | Action | Notes |
|------|--------|-------|
| `crates/rupa-syntax/tests/sizes.rs` | Create | `size_of` report and upper-bound assertions |
| `crates/rupa-syntax/tests/allocations.rs` | Create | Allocation-count fixture using an in-house test-only allocator (`stats_alloc` unavailable offline) |
| `crates/rupa-syntax/tests/fixtures/baseline.rupa` | Create | Synthetic repeated group/layer/text fixture |
| `crates/rupa-syntax/Cargo.toml` | Unchanged | `stats_alloc` is not in the offline registry |
| `Cargo.toml` | Unchanged | No new dependency selected |
| `docs/decisions/DECISION-042-compact-ast-memory.md` | Create | Baseline, interning scope, dependency decision |
| `agent/tasks/TASK-097.md` | Modify | Record measured baseline |

## Work Items

1. Record `size_of` for `FileId`, `Position`, `Span`, `Token`, `TokenKind`,
   `Document`, `DocumentChild`, `Text`, `Fill`, `Expression`,
   `GroupDefinition`, and `LayerDefinition`.
2. Add a synthetic fixture with repeated layer/group names and repeated text to
   expose deduplication opportunities.
3. Add an allocation counter that reports allocated bytes and allocation count
   for parsing the fixture.
4. Validate `ustr` as the primary `IStr` backend:
   - feature set needed for serde;
   - `wasm32-unknown-unknown` build;
   - license compatibility;
   - exact `size_of` on native and wasm32.
   If `ustr` is unavailable or unsuitable, choose the in-house `Arc<str>`
   fallback and record the larger size target.
5. Draft `DECISION-042` with the interning scope:
   - intern identifiers and finite names;
   - do not intern large unique payloads;
   - serialization stays plain strings.
6. Commit the baseline test fixture without changing production behavior.

## Acceptance Criteria

**All must pass to mark task complete:**

- [x] Baseline size table is recorded in `TASK-097.md` and `DECISION-042`.
- [x] `cargo test -p rupa-syntax --test sizes` prints or asserts the baseline.
- [x] Allocation-count test runs deterministically with `CARGO_INCREMENTAL=0`.
- [x] Interning backend choice and fallback are documented.
- [x] `ustr` (if selected) builds for `wasm32-unknown-unknown` and serializes strings.
  `ustr` is not selected; the `rupa-syntax` crate itself builds for
  `wasm32-unknown-unknown` and the fallback handle size is recorded.
- [x] No production syntax, diagnostic, JSON, or renderer behavior changes.
- [x] `cargo fmt --all -- --check` passes.
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes.
- [x] `cargo build --workspace` passes.
- [x] `cargo test --workspace` passes (535 passed, 0 failed).

## Testing Requirements

- Unit tests: none required for production code.
- Integration tests: size assertions and allocation counter in `rupa-syntax`.
- Golden images: existing goldens must remain unchanged.

## Specification References

- `rupa-specification.md` §20: repository and implementation constraints.
- `docs/decisions/DECISION-042-compact-ast-memory.md`: new decision record.

## Measured Baseline

`cargo test -p rupa-syntax --test sizes -- --nocapture` (x86_64, debug):

| Type | Bytes |
|------|------:|
| `FileId` | 8 |
| `Position` | 24 |
| `Span` | 56 |
| `Token` | 88 |
| `TokenKind` | 32 |
| `Document` | 456 |
| `DocumentChild` | 1456 |
| `Text` | 1432 |
| `Fill` | 600 |
| `Expression` | 200 |
| `GroupDefinition` | 304 |
| `LayerDefinition` | 992 |
| `Arc<str>` fallback handle | 16 |

`cargo test -p rupa-syntax --test allocations -- --nocapture` parses
`tests/fixtures/baseline.rupa` in **381 allocations / 150,297 bytes**; two
consecutive parses report identical counts. Guardrails are 420 allocations and
160,000 bytes.

## Backend Decision

`ustr` and `stats_alloc` are absent from the offline registry index
(`cargo add ... --offline` reports "could not be found in registry index").
TASK-097 therefore selects the documented fallback: `IStr` as a newtype over
`Arc<str>` (16 bytes native, 8 bytes on wasm32), implemented in TASK-099.
`stats_alloc` is replaced by a test-only counting global allocator. No
dependency changes were made. See `docs/decisions/DECISION-042-compact-ast-memory.md`.

**Update 2026-11-14**: the registry was later confirmed online. DECISION-042 now
selects `ustr` as the primary `IStr` backend for TASK-099, keeping `Arc<str>` as
the documented fallback.

## Handoff Notes

**Completed Work**:
- Added the size baseline, fallback-handle size check, and fixture parse test in
  `crates/rupa-syntax/tests/sizes.rs`.
- Added the deterministic allocation counter and guardrail in
  `crates/rupa-syntax/tests/allocations.rs`.
- Added `crates/rupa-syntax/tests/fixtures/baseline.rupa`.
- Recorded baseline, interning scope, and backend decision in
  `docs/decisions/DECISION-042-compact-ast-memory.md`.
- No production code changed.

**Remaining Work**:
- TASK-098 may start. It must tighten the `Span` guardrail from 56 to <= 40.
- TASK-099 implements `IStr` over the selected `Arc<str>` fallback.

**Test Results**:
- `cargo fmt --all -- --check`: clean.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: clean.
- `cargo build --workspace`: OK.
- `cargo test --workspace`: 535 passed, 0 failed.
- `cargo build -p rupa-syntax --target wasm32-unknown-unknown`: OK.
- Sizes: 3 passed. Allocations: 1 passed.

**Known Issues**:
- No network, so `ustr`/`stats_alloc` could not be evaluated directly; the
  fallback is documented and the tests avoid new dependencies.
- Allocation counts are tied to the current allocator/compiler behavior; the
  upper bound leaves a small margin and the test also pins run-to-run equality.

**Exact Next Action**:
- Implement TASK-098: shrink `FileId`/`Position`/`Span`, keep reported values
  unchanged, and lower the `Span` guardrail to `<= 40`.
