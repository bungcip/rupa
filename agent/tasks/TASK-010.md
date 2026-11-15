# TASK-010: CLI check and render commands

## Task Header
- **Task ID**: TASK-010
- **Title**: CLI check and render commands
- **Milestone**: M1 — Minimal Vertical Slice
- **Priority**: Critical
- **Estimated Effort**: 4-5 hours
- **Dependencies**: TASK-009

## Description
Implement CLI entry points for `rupa check` and `rupa render` commands per specification §21. Wire together lexer, parser, compiler, IR, and renderer.

## Files to Create/Modify

| File | Action | Notes |
|------|--------|-------|
| `crates/rupa-cli/src/main.rs` | Create | CLI entry point |
| `crates/rupa-cli/src/commands.rs` | Create | Command implementations |
| `crates/rupa-cli/src/diagnostics.rs` | Create | Diagnostic output |
| `crates/rupa-cli/Cargo.toml` | Modify | Add clap, other deps |
| `crates/rupa-compiler/src/lib.rs` | Modify | Public compile API |
| `crates/rupa-renderer/src/lib.rs` | Modify | Public render API |

## Acceptance Criteria

**All must pass to mark task complete:**

- [x] `rupa check <input.rupa>` — parses, validates, no render
- [x] Exit code 0 on success
- [x] Exit code 1 on error
- [x] Diagnostics to stderr with source context
- [x] `rupa render <input.rupa> --output <output.png>` — compiles and renders
- [x] Exit code 0 on success
- [x] Exit code 1 on error
  - PNG written to output path
- [x] Diagnostics to stderr with source context
- [x] `rupa --help` shows both commands
- [x] `rupa check --help` and `rupa render --help` show options
- [x] Invalid input → non-zero exit, clear error to stderr
- [x] Missing input file → error
- [x] Missing `--output` for render → exit 1 error
- [x] Integration tests for both commands (12 CLI tests)
- [x] `cargo test -p rupa-cli` passes
- [x] `cargo fmt -- --check` passes
- [x] `cargo clippy -- -D warnings` passes

## Implementation Steps

1. Add `clap` with derive feature to `rupa-cli`
2. Define CLI struct with subcommands `check` and `render`
3. Implement `check` command:
   - Read file
   - Run lexer → parser → compiler validation
   - Print diagnostics
   - Exit with appropriate code
4. Implement `render` command:
   - Read file
   - Run full pipeline: lexer → parser → compiler → IR → renderer → PNG
   - Write PNG to `--output` path
   - Print diagnostics
   - Exit with appropriate code
5. Create public API in `rupa-compiler` and `rupa-renderer` for CLI use
6. Write integration tests

## Testing Requirements

- `check` on valid file → exit 0, no output to stdout
- `check` on invalid file → exit 1, error to stderr
- `render` on valid file → exit 0, PNG created
- `render` on invalid file → exit 1, no PNG created
- `render` missing `--output` → exit 1, usage error
- Diagnostics format matches specification (§22.3)
- Multiple errors reported together

## Specification References

- §21: CLI
- §22.3: Diagnostics for iterative repair
- §19: Compiler pipeline

## Handoff Notes

**Completed Work**:
- Added `rupa_ir::GraphicsDocument` and re-exported it as
  `rupa_renderer::RenderDocument`.
- Added compiler public APIs: `parse_source`, `compile_source`,
  `ParsedSource`, `CompiledSource`, and `lower_document`.
- Refactored `Validator::validate_source` to use the shared parse API.
- Implemented AST→IR lowering for dimensions, background, rectangles,
  circles, fills/`fill none`, and canvas rounding (`C008` for oversized).
- Rebuilt the CLI into `main.rs`, `commands.rs`, and `diagnostics.rs`:
  - `rupa check <input>` with source-context diagnostics on stderr
  - `rupa render <input> --output <file.png>` with required output, compile,
    render, and PNG encoding
  - `--help` for root/check/render
  - exit codes 0/1 including missing `--output`
- Added 12 CLI integration tests (help, success, multiple diagnostics, missing
  file, PNG contents, missing output, invalid source, non-PNG output, export
  override) plus compiler lowering/compile tests.
- Recorded the API/CLI contract in `docs/decisions/DECISION-007.md`.

**Remaining Work**:
- None for TASK-010.

**Test Results**:
- `cargo test --workspace`: passes (rupa-cli 12, rupa-compiler 74,
  rupa-geometry 9, rupa-image 7, rupa-ir 10, rupa-renderer 23, rupa-syntax 77).
- `cargo clippy --all-targets --all-features -- -D warnings`: passes.
- `cargo fmt -- --check`: passes.

**Known Issues**:
- `render` always requires `--output`; it does not fall back to the document
  `export` path.
- `fmt`/`inspect` remain not implemented.
- Golden-image end-to-end fixture is TASK-011.

**Exact Next Action**:
- Begin TASK-011: Integration test with golden image.
