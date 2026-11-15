# DECISION-026: One file is one document

**Date**: 2026-11-12
**Task**: TASK-056 (Document model and root layer specification)

## Context

Rupa originally required a `document "Name"` block wrapping every file. The
wrapper created a second source level with no semantic value: the compiler
enforced exactly one declaration, output came from `export` or the CLI, and the
name was not used by rendering. The extra level also made every example, test,
formatter rule, and migration more complex.

## Options Considered

1. Keep the `document` wrapper and add a document name field to the IR.
2. Make the `document` block optional while keeping both forms.
3. Make one `.rupa` file exactly one document and remove the wrapper.

## Decision

Option 3. A `.rupa` file is exactly one document. Document settings (`size`,
`background`, `seed`, `export`) are file-level declarations. There is no
source-level document name; tooling may derive a display name from the filename
or export path. For a migration window, the parser recognizes `document` only
to emit a stable, actionable diagnostic; it is not valid syntax.

## Consequences

- Top-level source is one indentation level shallower and has one canonical
  form, which is easier for LLMs to generate and repair.
- `Document.name`, parser document-wrapper handling, formatter indentation, and
  `C001`/`C002`-based tests must be retired or migrated.
- Duplicate document settings need explicit diagnostics.
- Existing `.rupa` files, examples, benchmarks, fixtures, and documentation must
  be migrated by TASK-061.
- Multiple output images require multiple files or repeated CLI invocations.
