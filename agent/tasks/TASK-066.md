# TASK-066: Simplify DiagnosticBag error handling

**Milestone**: Maintenance (post-M9)
**Dependencies**: TASK-065
**Status**: Complete

## Goal

Reduce repetitive diagnostic construction and direct bag internals access.

## Scope

- Add `DiagnosticBag::error`, `error_with_help`, and `append` convenience methods.
- Replace direct `bag.diagnostics.extend(other.diagnostics)` calls with
  `bag.append(other)`.
- Replace simple `bag.push(Diagnostic::error(...))` calls with `bag.error(...)`.
- Add unit coverage for the new convenience methods.
- Keep `push` and direct construction available for complex diagnostics.

## Acceptance Criteria

- [x] Convenience constructors and bag append exist
- [x] Compiler/parser call sites use the new append/error helpers
- [x] Diagnostic codes, ordering, and behavior unchanged
- [x] Full workspace validation passes

## Verification

- `cargo fmt -- --check` — pass
- `cargo clippy --all-targets --all-features -- -D warnings` — pass
- `cargo build --workspace` — pass
- `cargo test --workspace` — pass (rupa-syntax: 120 tests)
