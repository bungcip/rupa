# TASK-040: Deterministic PRNG implementation

**Milestone**: 6 — Procedural Language Features
**Dependencies**: TASK-039 (Indexed loops)
**Status**: Complete

## Goal

Implement a documented, seedable deterministic PRNG and wire document-level
seeds through parsing, IR, and lowering.

## Scope

- Add `seed` to the document AST and parser (`seed <n>`).
- Add `seed` to `GraphicsDocument`.
- Lower the declared seed (default `0`).
- Implement an xoshiro256** PRNG seeded via splitmix64 in `rupa-syntax`,
  with aliases for common module/type names.

## Acceptance Criteria

- [x] Document seeds parse and lower deterministically
- [x] PRNG is deterministic for a given seed
- [x] `cargo fmt`, `cargo clippy`, `cargo build`, `cargo test` pass