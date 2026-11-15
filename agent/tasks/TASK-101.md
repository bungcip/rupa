# TASK-101: Optimize enum layout by measurement

## Task Header

- **Task ID**: TASK-101
- **Title**: Box large enum variants without speculative overhead
- **Milestone**: M17 — Layout and verification
- **Priority**: Medium
- **Estimated Effort**: 6–10 hours
- **Dependencies**: TASK-098, TASK-100
- **Status**: Complete

## Description

An enum is as large as its largest variant. `DocumentChild` and `Fill` can
therefore make every node pay for a rarely used large variant. After the span
and string changes, measure variant sizes and box only variants whose size
reduction justifies the extra allocation.

This task is measurement-driven. Do not box variants merely because they look
large; record the before/after size and allocation cost.

## Files to Create/Modify

| File | Action | Notes |
|------|--------|-------|
| `crates/rupa-syntax/src/ast.rs` | Modify | Box selected variants |
| `crates/rupa-syntax/src/parser.rs` | Modify | Construct boxed variants |
| `crates/rupa-compiler/src/*.rs` | Modify | Update pattern matches |
| `crates/rupa-cli`, `crates/rupa-wasm` | Modify | Update AST consumers if any |
| `crates/rupa-syntax/tests/sizes.rs` | Modify | Guard final enum sizes |
| `crates/rupa-syntax/tests/allocations.rs` | Modify | Track boxing overhead |
| `docs/decisions/DECISION-042-compact-ast-memory.md` | Modify | Record measurements and choices |

## Candidate Changes

Evaluate in this order:

1. `DocumentChild::Text(Box<Text>)` if `Text` is the largest or near-largest
   variant.
2. `DocumentChild::Path(Box<Path>)` if it materially reduces the enum size.
3. Gradient/pattern variants inside `Fill` if they dominate `Option<Fill>`.
4. `Expression::Position(Box<Position>)` only if measurement shows a clear win.

Also evaluate converting finite string fields such as `blend`, filter `kind`,
and pattern `kind` into enums. Only do this if diagnostic text and serialized
output remain identical; otherwise keep `IStr`.

## Acceptance Criteria

**All must pass to mark task complete:**

- [x] Before/after variant sizes are recorded.
- [x] `size_of::<DocumentChild>()` is reduced by at least 25% from the
      TASK-097 baseline, or the stop decision is documented with data.
- [x] Each boxed variant adds at most one allocation per node of that variant.
- [x] No AST behavior, diagnostic, JSON, or render order changes.
- [x] Render benchmark regression is at most 5%.
- [x] `cargo fmt --all -- --check` passes.
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes.
- [x] `cargo build --workspace` passes.
- [x] `cargo test --workspace` passes (551 passed, 0 failed).
- [x] `cargo build -p rupa-wasm --target wasm32-unknown-unknown --release` passes.

## Testing Requirements

- Unit tests: construction and matching for every boxed variant.
- Integration tests: full workspace parsing, checking, rendering.
- Golden images: unchanged.
- Benchmarks: render time and allocation count before/after.

## Specification References

- `rupa-specification.md` §7–§10: AST-backed language constructs.
- `DECISION-042`: measured layout policy.

## Result

Measured payloads showed `Fill` (408 B) dominated every shape node. Boxing its
gradient/pattern payloads reduced `Fill` to 72 B and `DocumentChild` to 720 B.

| Item | Before | After |
|------|-------:|------:|
| `Fill` | 408 | 72 |
| `DocumentChild` | 1056 | 720 |
| Rectangle/Ellipse/Triangle | 1048 | 712 |
| `Text` | 1016 | 680 |
| `Path` | 768 | 432 |
| `LayerDefinition` | 704 | 368 |
| `Expression` | 152 | 72 |
| `Repeat` | 224 | 144 |
| `Variable` | 200 | 120 |

`DocumentChild` is 50.5% below the TASK-097 baseline (1456 -> 720).

- Boxed `Fill.gradient`, `Fill.pattern`, and `Expression::Position`; each pays
  exactly one allocation only when that payload is present.
- Did not box `DocumentChild::Text` or `Path`: after the `Fill` change the
  largest variant is the 712-byte rectangle/ellipse/triangle, so boxing them
  would not shrink the 720-byte enum and would add an allocation for common
  nodes.
- Kept finite names as `IStr`: closed enums would change diagnostics for unknown
  values or require an extra raw-name variant, with no size win over 8 bytes.
- Fixture parse: 271 allocations / 101,203 bytes (from 278 / 109,651).
- Render benchmark on `examples/paints.rupa`: 40 alternating release runs,
  median 20.97 -> 20.94 ms (-0.17%), mean -0.9%; output PNG byte-identical.

## Handoff Notes

**Completed Work**:
- Boxed `Fill.gradient`, `Fill.pattern`, and `Expression::Position`.
- Added `tests/boxed_variants.rs` covering gradient, pattern, and position
  construction/serialization/evaluation.
- Added permanent enum-variant measurement output and tightened all size
  guardrails; updated allocation guardrails and DECISION-042.
- Verified no behavior, diagnostic, JSON, render order, or golden changes.

**Remaining Work**:
- TASK-102 performs the final memory/perf/wasm/diagnostic/golden verification
  and closes M17.

**Test Results**:
- `cargo fmt --all -- --check`: clean.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: clean.
- `cargo build --workspace`: OK.
- `cargo test --workspace`: 551 passed, 0 failed.
- `cargo build -p rupa-wasm --target wasm32-unknown-unknown --release`: OK
  (1,400,685 bytes; +0.9% vs TASK-100).
- `cargo test -p rupa-syntax --test boxed_variants`: 3 passed.
- `cargo test -p rupa-syntax --test sizes -- --nocapture`: 4 passed.
- `cargo test -p rupa-syntax --test allocations -- --nocapture`: 1 passed.

**Known Issues**:
- Boxed payloads add one allocation only when used; the baseline fixture has no
  gradient, pattern, or position expression and remains allocation-stable.
- IR fields still use `String`; TASK-102 verifies the final boundaries.

**Exact Next Action**:
- Implement TASK-102: final verification of memory, parsing/render performance,
  wasm size, diagnostics, JSON, and golden images; then close M17.
