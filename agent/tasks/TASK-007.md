# TASK-007: Solid fill paint representation

## Task Header
- **Task ID**: TASK-007
- **Title**: Solid fill paint representation
- **Milestone**: M1 — Minimal Vertical Slice
- **Priority**: Critical
- **Estimated Effort**: 3-4 hours
- **Dependencies**: TASK-006

## Description
Implement paint IR for solid colors including named colors and hex strings. Separate geometry from appearance per specification §8.1 and §17.

## Files to Create/Modify

| File | Action | Notes |
|------|--------|-------|
| `crates/rupa-ir/src/paint.rs` | Create | Paint IR types |
| `crates/rupa-compiler/src/paint.rs` | Create | Paint validation/lowering |
| `crates/rupa-compiler/src/error.rs` | Modify | Add paint error codes |
| `crates/rupa-ir/src/lib.rs` | Modify | Export paint module |
| `crates/rupa-compiler/src/lib.rs` | Modify | Export paint module |

## Acceptance Criteria

**All must pass to mark task complete:**

- [x] `Paint` enum with `Solid(Color)` variant
- [x] `Color` struct with straight RGBA (`f32` 0.0–1.0), premultiply at render time
- [x] Named colors: `white`, `black`, `red`, `green`, `blue`, `yellow`, `orange`, `purple`, `gray`, `transparent`
- [x] Hex strings: `"#RRGGBB"` and `"#RRGGBBAA"` (parsed from string tokens)
- [x] Invalid hex → error `T001` (task-doc `R300`) with format help/suggestion
- [x] Unknown named color → error `T002` (task-doc `R301`) with suggestion
- [x] `fill none` → `Option<PaintId>` = None
- [x] Paint IDs for IR referencing (`PaintStore` arena)
- [x] Unit tests for all color formats
- [x] `cargo test -p rupa-ir -p rupa-compiler` passes
- [x] `cargo fmt -- --check` passes
- [x] `cargo clippy -- -D warnings` passes

## Implementation Steps

1. Define `Color` with `r, g, b, a` (f32 0.0–1.0)
2. Define `Paint` enum (Solid for now, extensible)
3. Implement named color lookup table
4. Implement hex string parsing
5. Define `PaintId` type for arena references
6. Implement paint lowering from AST to IR
7. Define paint error codes (`R300`–`R399`)
8. Write tests

## Testing Requirements

- All 10 named colors → correct RGBA
- `#RRGGBB` → correct RGB, alpha=1.0
- `#RRGGBBAA` → correct RGBA
- Invalid hex length → error `R300`
- Invalid hex characters → error `R300`
- Unknown name (e.g., `colour`) → error `R301`
- `fill none` → no paint
- Case sensitivity: names lowercase, hex case-insensitive

## Specification References

- §4.3: Colors
- §8.1: Solid fill
- §17: Graphics IR (paint abstraction)
- §2.1: Canonical syntax (`fill` not `color`/`paint`)

## Handoff Notes

**Completed Work**:
- Reworked `rupa-ir/src/paint.rs`: straight-alpha `Color` (`f32` 0.0–1.0),
  `Paint::Solid(Color)`, single `PaintId`, arena-backed `PaintStore`.
- Implemented `PaintValidator` in `rupa-compiler/src/paint.rs`:
  - exact, case-sensitive named color table (10 canonical names)
  - `#RRGGBB` / `#RRGGBBAA` parsing (case-insensitive digits)
  - `T001` invalid hex with format help, `T002` unknown named color with
    valid-name list and "did you mean" suggestion
  - lowering to `Option<Paint>` and `Option<PaintId>`, including `fill none`
- AST `Fill.color` is now `Option<Color>`; parser accepts `fill none`.
- `Validator` now reports invalid fill colors and invalid background colors.
- Recorded conventions in `docs/decisions/DECISION-004.md`.
- Tests: 19 paint lowering tests, 4 IR paint tests, 1 parser test, 4 validator
  integration tests.

**Remaining Work**:
- None for TASK-007.

**Test Results**:
- `cargo test --workspace`: passes (rupa-syntax 77, rupa-compiler 64, rupa-ir 8).
- `cargo clippy --all-targets --all-features -- -D warnings`: passes.
- `cargo fmt -- --check`: passes.

**Known Issues**:
- AST-to-IR shape lowering and background lowering are TASK-008.
- `rgb(r, g, b)` syntax is not parsed yet; the AST variant already lowers.

**Exact Next Action**:
- Begin TASK-008: Basic renderer with tiny-skia.
