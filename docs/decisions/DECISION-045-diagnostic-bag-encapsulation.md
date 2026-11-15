# DECISION-045: Encapsulate `DiagnosticBag` storage

- **Status**: Accepted
- **Date**: 2026-11-15
- **Related**: TASK-114, DECISION-036 (diagnostic edits), specification §22

## Context

`rupa_syntax::diagnostic::DiagnosticBag` exposed its storage as
`pub diagnostics: Vec<Diagnostic>`. Callers in `rupa-compiler`, `rupa-cli`,
`rupa-wasm`, and the syntax tests read, extended, indexed, and printed that
vector directly. This bypassed the bag's own contract (`push`, `append`,
`has_errors`, `errors`, `warnings`, `sort`) and made it impossible to change
the storage representation without touching every consumer.

## Options considered

| Option | Result |
|--------|--------|
| Keep the public field | No breakage, but callers keep bypassing `append`/`has_errors` and storage stays part of the public API. |
| Keep the field, document it as read-only | Rust cannot enforce read-only fields; mutation and replacement remain possible. |
| `Deref<Target = [Diagnostic]>` | Gives read access through methods, but also exposes inherent slice methods and does not provide a clear collection API. Rejected. |
| Private field plus collection traits | Enforces all access through the bag, keeps read paths ergonomic via `iter`, `as_slice`, `len`, `is_empty`, `IntoIterator`, and keeps construction/merging via `FromIterator`, `Extend`, `append`. Selected. |

## Decision

Make `DiagnosticBag::diagnostics` private and expose:

- `len`, `is_empty`, `as_slice`, and `iter` for reading in emission order.
- `IntoIterator` for `DiagnosticBag` and `&DiagnosticBag`.
- `FromIterator<Diagnostic>` and `Extend<Diagnostic>` so callers can build and
  merge bags without naming the storage vector.
- Existing `push`, `error`, `error_with_help`, `append`, `has_errors`,
  `errors`, `warnings`, and `sort` remain the mutation/filtering API.

All in-tree consumers were migrated: compiler tests and validators use
`as_slice`/`iter`/`len`/`is_empty`, the CLI prints by iterating the bag, and the
wasm bridge uses `sort` plus iteration. Serialization is unchanged because the
serde field name (`diagnostics`) is unchanged.

## Consequences

- Storage is now an implementation detail; a future change (for example a
  small-vector or boxed slice) no longer breaks callers.
- Direct mutation or replacement of the vector is a compile error outside the
  module.
- The public API grows by four read methods and three trait implementations;
  no behavior, diagnostic ordering, JSON, or rendered output changes.
- `rupa-syntax` 132 tests and the full 558-test workspace suite pass; all
  example PNGs, `inspect` JSON, and invalid-fixture `check --json` outputs are
  byte-identical to `36d78c4`.
