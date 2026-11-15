# TASK-069: Remove `export`; CLI-determined output path

## Task Header

- **Task ID**: TASK-069
- **Title**: Remove `export` from the language; CLI output contract with an
  `input.png` default and a migration diagnostic
- **Milestone**: M10 — Groups, defaults, and CLI output
- **Priority**: High
- **Estimated Effort**: 3–5 hours
- **Dependencies**: TASK-067 (specification), TASK-068 (defaults)

---

## Description

Remove `export` from the language per specification §5, §5.1, and §21
(DECISION-031).

- `export` is no longer a document setting. The parser recognizes it only to
  emit the stable migration diagnostic `P005`; the formatter rejects it too.
- `Document::export` and the `Export` AST node are gone, so no later stage can
  read an output path from source.
- `rupa render input.rupa` writes `input.png` in the **current working
  directory**; `--output path.png` overrides it.
- `ValidationOptions` existed only to let the CLI override export validation,
  so it is removed; `compile_source` and the validator entry points no longer
  take options.
- Retired diagnostic codes stay defined for stability: `C004`
  (missing export), `C006` (invalid export path), `C012` (duplicate export) are
  marked legacy and never emitted.
- All sources are migrated: `examples/`, `benchmarks/`, inline test sources,
  and `docs/language-reference.json` (keywords, example, required settings,
  canonical order, top-level list).

---

## Files to Create/Modify

| File | Action | Notes |
|------|--------|-------|
| `crates/rupa-syntax/src/ast.rs` | Modify | Drop `export` field and `Export` node |
| `crates/rupa-syntax/src/parser.rs` | Modify | `parse_removed_export` → `P005` |
| `crates/rupa-syntax/src/formatter.rs` | Modify | Reject `export`; drop it from canonical order |
| `crates/rupa-syntax/src/diagnostic.rs` | Modify | Add `P005`; mark `C004`/`C006`/`C012` legacy |
| `crates/rupa-compiler/src/{document,compile,validation,lower}.rs` | Modify | Remove export validation and `ValidationOptions` |
| `crates/rupa-cli/src/{commands,main}.rs` | Modify | Default output path; help text |
| `crates/rupa-cli/src/benchmark.rs` | Modify | New `compile_source` signature |
| `crates/rupa-cli/tests/{cli,benchmark}.rs` | Modify | Default-output and migration tests |
| `docs/language-reference.json` | Modify | Remove `export`; document defaults |
| `examples/**`, `benchmarks/**` | Modify | Delete `export` lines |

---

## Acceptance Criteria

**All must pass to mark task complete:**

- [x] `export` is a parse error with `P005` and a `--output` hint
- [x] `rupa fmt` rejects `export` instead of reordering it
- [x] `rupa render input.rupa` writes `<stem>.png` in the working directory
- [x] `rupa render --output custom.png` writes exactly that path
- [x] `rupa render --output out.jpg` and an extensionless `--output` are
      rejected with a clear CLI error and write nothing; `.PNG` is accepted
- [x] `Document` carries no export path; `ValidationOptions` is gone
- [x] `C004`, `C006`, `C012` remain defined but are never emitted
- [x] Examples, benchmarks, inline fixtures, and the language reference are
      migrated; executable-example tests pass
- [x] `cargo fmt -- --check` passes
- [x] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [x] `cargo build --workspace` passes
- [x] `cargo test --workspace` passes

---

## Implementation Steps

1. Remove `export` from the AST and parser; add `parse_removed_export`.
2. Add `P005`, mark the export error codes legacy, update the catalog test.
3. Remove export validation and `ValidationOptions`; simplify `compile_source`.
4. Default the render output path in the CLI and update help text.
5. Migrate `.rupa` fixtures and inline test sources; retarget the
   `repair-invalid` benchmark to a real diagnostic (`S005`).
6. Update `docs/language-reference.json`.

---

## Testing Requirements

- Unit tests: `export` reports `P005`; empty source is now valid; the formatter
  rejects `export`; `export` duplicate diagnostics are gone.
- Integration tests: default output filename and directory; explicit
  `--output`; migration diagnostic in `rupa check`.
- Golden images: unchanged (examples no longer declare exports).

---

## Specification References

- §5 Document model and output
- §5.1 Migrating older sources
- §21 CLI
- §23.1 Lexer and parser tests

---

## Handoff Notes

**Completed Work**:
- `export` removed end to end, with `P005` migration diagnostics.
- CLI output defaults to `<input stem>.png` in the current directory.

**Remaining Work**:
- TASK-070 … TASK-072 (groups, layer semantics, migration close-out).

**Test Results**:
- fmt, clippy (`-D warnings`), build, and all workspace tests pass (483 tests).

**Known Issues**:
- Benchmark prompts still describe layers rather than groups; that migration is
  TASK-072.

**Follow-up (resolved)**:
- The export removal initially dropped output-path extension validation, so
  `rupa render --output out.jpg` wrote PNG bytes to a `.jpg` name. A CLI-level
  check now rejects any `--output` that is not a `.png` path (case-insensitive)
  with `CliError::NonPngOutput` before rendering; the default output path is
  always `.png`. Specified in §21 and covered by three CLI tests (`.jpg`,
  extensionless, `.PNG` accepted).

**Exact Next Action**:
- TASK-070: add `group` definitions and make `place` target groups only.
