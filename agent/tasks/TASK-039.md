# TASK-039: Indexed loops

**Milestone**: 6 — Procedural Language Features
**Dependencies**: TASK-038 (Repeat loops, counted)
**Status**: Complete

## Goal

Extend counted repeat loops with an optional loop index variable
(`repeat N times as i`), exposing the zero-based iteration index to the
loop body.

## Scope

- Parse `as <identifier>` after the `times` keyword.
- Store the index variable name on `Repeat`.
- Seed the index variable in validation and lowering scopes.

## Acceptance Criteria

- [x] Indexed repeat loops parse
- [x] Loop body can reference the index variable
- [x] Validation and lowering use zero-based indices
- [x] `cargo fmt`, `cargo clippy`, `cargo build`, `cargo test` pass