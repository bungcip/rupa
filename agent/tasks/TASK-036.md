# TASK-036: Variable declarations and scoping

**Milestone**: 6 — Procedural Language Features
**Dependencies**: TASK-030 (Resource diagnostics), TASK-005 (Document declaration)
**Status**: Complete

## Goal

Add `let` variable declarations and block-level duplicate-name validation, so
procedural values can be named and scoped before expression evaluation arrives
in TASK-037.

## Scope

- Parse `let name = value` in document and layer blocks.
- Store declarations as `DocumentChild::Let(Variable)`.
- Support literal values: numbers, dimensions, strings, booleans, identifiers,
  and coordinate pairs.
- Reject duplicate variable names within the same block (`S024`).
- Lowering ignores declarations (no graphics IR impact).

## Acceptance Criteria

- [x] `let` declarations parse in document and layer blocks
- [x] Duplicate declarations in one block are reported as `S024`
- [x] Declarations in different blocks do not conflict
- [x] `cargo fmt`, `cargo clippy`, `cargo build`, `cargo test` pass