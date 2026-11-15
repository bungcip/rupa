# TASK-042: Function grammar and basics

**Milestone**: 6 — Procedural Language Features
**Dependencies**: TASK-037 (Expression parsing and evaluation)
**Status**: Complete

## Goal

Implement basic user-defined functions for reusable calculations.

## Scope

- Parse `fn name(params)` declarations with an indented `return` body or
  same-line `return`.
- Store function definitions on the document.
- Validate duplicate names/parameters.
- Evaluate function calls in expressions, with recursion protection.

## Acceptance Criteria

- [x] Function declarations parse
- [x] Function calls evaluate deterministically
- [x] `cargo fmt`, `cargo clippy`, `cargo build`, `cargo test` pass