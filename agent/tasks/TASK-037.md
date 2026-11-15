# TASK-037: Expression parsing and evaluation

**Milestone**: 6 — Procedural Language Features
**Dependencies**: TASK-036 (Variable declarations and scoping)
**Status**: Complete

## Goal

Add full expression parsing (arithmetic, comparisons, logical operators,
unary operators, function calls, and coordinate pairs) and deterministic
evaluation for variable declarations.

## Scope

- Expanded `Expression` AST with binary/unary operators and calls.
- Operator precedence and associativity.
- Evaluation of expressions against a variable scope.
- Validation of variable values, including unknown variables and type errors.
- Added `Value` runtime representation.

## Acceptance Criteria

- [x] Arithmetic, comparison, and logical expressions parse
- [x] Function call syntax parses
- [x] Coordinate-pair expressions parse
- [x] Unknown variables in expressions are reported
- [x] `cargo fmt`, `cargo clippy`, `cargo build`, `cargo test` pass