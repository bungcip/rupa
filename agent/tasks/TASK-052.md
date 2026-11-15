# TASK-052: Inspect command JSON output

**Milestone**: 8 — LLM Authoring Experience
**Dependencies**: TASK-005
**Status**: Complete

## Goal

Implement `rupa inspect` to print a JSON representation of a compiled document.

## Scope

- Wire `rupa inspect <input.rupa>` to compile and serialize the document as JSON.
- Report diagnostics and fail on errors.

## Acceptance Criteria

- [x] `rupa inspect` outputs valid JSON
- [x] `cargo fmt`, `cargo clippy`, `cargo build`, `cargo test` pass
