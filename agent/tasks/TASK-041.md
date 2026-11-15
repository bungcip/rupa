# TASK-041: Random function integration

**Milestone**: 6 — Procedural Language Features
**Dependencies**: TASK-040 (Deterministic PRNG implementation)
**Status**: Complete

## Goal

Integrate the deterministic `random(min, max)` function into expression
evaluation and compiler pipelines.

## Scope

- Parse `random(...)` calls (random keyword).
- Implement `random(min, max)` in `Expression::evaluate`, supporting numbers
  and dimensions with matching units.
- Thread the document PRNG through validation and lowering.
- Consume RNG deterministically in source order.

## Acceptance Criteria

- [x] `random(min, max)` parses and evaluates
- [x] Deterministic output for a given seed
- [x] `cargo fmt`, `cargo clippy`, `cargo build`, `cargo test` pass