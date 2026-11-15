# DECISION-007: Compilation API and CLI contract

**Date**: 2026-11-10
**Task**: TASK-010 (CLI check and render commands)

## Context

Specification §21 defines `check` and `render`; §19 describes the compiler
pipeline; §22.3 requires machine-readable diagnostics with source context.
TASK-010 requires `render --output` to be mandatory, while §5 allows the CLI
to override a document's `export` declaration.

Before this task the CLI only validated source and wrote a placeholder file,
and the compiler had no public way to obtain both diagnostics and lowered IR.

## Decision

### IR document

- `rupa_ir::GraphicsDocument` is the backend-independent compiled document:
  canvas dimensions, optional background color, `PaintStore`, and ordered
  `ShapeNode`s.
- `rupa-renderer` re-exports it as `RenderDocument`; rendering consumes the IR
  document directly.

### Compiler API

- `rupa_compiler::parse_source(source, file_id)` returns
  `ParsedSource { document, diagnostics }` and performs the lexer,
  missing/multiple document checks, and parser.
- `rupa_compiler::compile_source(source, file_id, options)` returns
  `CompiledSource { document, graphics, diagnostics }`. It validates, then
  lowers to `GraphicsDocument` only when no errors were produced.
- `lower_document(&Document)` is public for callers that already validated.
  Shapes are lowered in source order into the paint arena.
- Canvas dimensions are rounded to the nearest integer pixel; dimensions that
  cannot fit a `u32` canvas produce `C008`.
- `fill none` lowers to no paint and allocates nothing.
- Lowering is defensive: missing required properties still produce diagnostics
  instead of panicking, even if the caller skipped validation.

### CLI contract

- `rupa check <input>` compiles without rendering. Exit 0 on success, exit 1
  on diagnostics errors or input read failure. Diagnostics go to stderr with
  source context; stdout stays empty.
- `rupa render <input> --output <file.png>` requires `--output`. The output
  path overrides any `export` declaration; the effective path must end in
  `.png`. Missing `--output` is an error with exit 1 (not clap's exit 2) so
  the CLI contract stays consistent.
- PNG is written only after a successful compile and render. Invalid source
  never creates the output file.
- `rupa --help`, `rupa check --help`, and `rupa render --help` are provided by
  clap. `fmt` and `inspect` remain explicit not-implemented failures.

## Consequences

- TASK-011 can reuse `compile_source` plus `encode_render_output` for an
  end-to-end golden-image test.
- The document `export` path is currently only used for validation; rendering
  always requires `--output`. A future task may allow defaulting to `export`.
- Fractional pixel dimensions are not rejected; they round deterministically.
