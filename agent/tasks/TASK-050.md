# TASK-050: Canonical formatter

**Milestone**: 8 — LLM Authoring Experience
**Dependencies**: TASK-002, TASK-003
**Status**: Complete

## Goal

Implement `rupa fmt` to canonically format valid Rupa source.

## Scope

- Add a token-based canonical formatter in `rupa-syntax`.
- Wire `rupa fmt <input.rupa>` to format a file in place.
- Report lexer/parse errors via diagnostics.

## Acceptance Criteria

- [x] `rupa fmt` formats valid source
- [x] Output is idempotent
- [x] `cargo fmt`, `cargo clippy`, `cargo build`, `cargo test` pass
