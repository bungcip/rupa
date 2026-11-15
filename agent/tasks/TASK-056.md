# TASK-056: Document model and root layer specification

**Milestone**: 9 — Document Model & Root Layer Refactor
**Dependencies**: TASK-055
**Status**: Complete

## Goal

Update the language specification and record the architectural decisions for the
one-file-one-document model, the implicit root layer, and the follow-up
redundancy refactor.

## Scope

- Rewrite specification §§2, 3, 5, 6, 7, 16, 17, 21, 22, 23, 24, and 25 for the
  new document model.
- Remove `document "Name"` wrappers from full-document examples.
- Define migration behavior for the removed `document` keyword.
- Define the implicit root layer, reserved `root` name, top-level primitive
  shorthand, and root ordering/clipping semantics.
- Add `docs/decisions/DECISION-026.md`, `DECISION-027.md`, and `DECISION-028.md`.
- Add Milestone 9 implementation tasks to the backlog.

## Files Modified

- `rupa-specification.md`
- `docs/decisions/DECISION-026.md`
- `docs/decisions/DECISION-027.md`
- `docs/decisions/DECISION-028.md`
- `agent/BACKLOG.md`
- `agent/ROADMAP.md`
- `agent/STATUS.md`
- `agent/CURRENT.md`
- `agent/tasks/TASK-056.md` … `TASK-063.md`

## Acceptance Criteria

- [x] Specification states that one `.rupa` file is exactly one document
- [x] Specification removes the `document` wrapper and defines migration behavior
- [x] Specification defines the implicit root layer and root primitive semantics
- [x] Decision records 026–028 exist
- [x] Implementation tasks TASK-057…TASK-063 are in the backlog
- [x] Roadmap shows M1–M8 complete and M9 in progress
- [x] No parser/compiler behavior is changed by this task

## Verification

- `cargo fmt -- --check` — pass
- `cargo clippy --all-targets --all-features -- -D warnings` — pass
- `cargo build --workspace` — pass
- `cargo test --workspace` — pass
- `git diff --check` — pass

## Notes

This task is specification-first. The repository's code and fixtures still use
the old `document` syntax until TASK-057 onward. `docs/language-reference.json`,
examples, and benchmarks must not be migrated until the parser and lowering
tasks make the new syntax executable.
