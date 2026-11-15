# TASK-049: Color space and linear compositing

**Milestone**: 7 — Advanced Static Graphics
**Dependencies**: TASK-008, TASK-009
**Status**: Complete

## Goal

Document and expose color-space conversion helpers for linear-light compositing.

## Scope

- Add sRGB ↔ linear conversion helpers to `Color`.
- Document the renderer's compositing color space and PNG output contract.
- Record the decision in `docs/decisions/DECISION-025.md`.

## Acceptance Criteria

- [x] Color conversion helpers implemented and tested
- [x] Color space/compositing contract documented
- [x] `cargo fmt`, `cargo clippy`, `cargo build`, `cargo test` pass
