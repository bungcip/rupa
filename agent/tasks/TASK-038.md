# TASK-038: Repeat loops (counted)

**Milestone**: 6 — Procedural Language Features
**Dependencies**: TASK-037 (Expression parsing and evaluation)
**Status**: Complete

## Goal

Add counted `repeat N times` loops that duplicate their body children the
specified number of times during lowering.

## Scope

- Parse `repeat <expression> times` with an indented body in document and layer
  blocks.
- Store loops as `DocumentChild::Repeat(Repeat)`.
- Validate the count as a non-negative integer expression.
- Expand loops during lowering, with per-iteration variable scoping.

## Acceptance Criteria

- [x] Counted repeat loops parse in document and layer blocks
- [x] Invalid counts are reported as `S027`
- [x] Loops expand deterministically during lowering
- [x] `cargo fmt`, `cargo clippy`, `cargo build`, `cargo test` pass