# DECISION-002: Document validation policy

**Date**: 2026-11-10
**Task**: TASK-005 (Document declaration and validation)

## Context

Specification §5 leaves two document-validation questions to the
implementation:

1. Whether `export` is mandatory.
2. How a CLI output path interacts with the `export` declaration.

It also states that for the first implementation document dimensions are
pixels, and that exactly one `document` declaration is allowed.

TASK-005's acceptance criteria refer to error codes `R100`–`R103`. The
codebase uses category-prefixed stable codes per
[DECISION-001](DECISION-001.md), so the compiler codes are:

| Task-doc code | Implemented code | Meaning |
|---|---|---|
| `R100` (missing document) | `C001` | no `document` declaration |
| `R100` (multiple documents) | `C002` | more than one `document` declaration |
| `R102` (invalid dimensions) | `C003` | non-positive or non-finite dimension |
| `R103` (missing export) | `C004` | no export path and no CLI override |
| `R101` (missing size) | `C005` | no `size` declaration |
| — | `C006` | export path is empty or not a `.png` path |
| — | `C007` | document dimension does not use `px` |

## Decision

- Exactly one `document` declaration is required. `Validator::validate_source`
  tokenizes first so missing and multiple declarations can be reported before
  parsing the first document.
- `size` is mandatory and must contain positive, finite pixel dimensions.
  Non-pixel units in document dimensions are rejected (`C007`) until unit
  conversion semantics are defined.
- `export` is mandatory by default (`C004`, severity error). A CLI output path
  overrides the declaration; when an override is supplied, a missing
  `export` declaration is not an error. The effective path (override or
  declaration) must be non-empty and end in `.png` (case-insensitive).
- `background` stays optional; omission means transparent.
- The document origin is the top-left corner, X right, Y down. This is
  documented in `crates/rupa-compiler/src/document.rs`.
- `export` is accepted both inside the document block and at top level, as
  shown in the specification §5 example.

## Consequences

- Documents without `export` fail `rupa check`; `rupa render --output out.png`
  succeeds because the CLI supplies an override.
- `DocumentValidator::validate(&document)` keeps its existing signature and
  uses the default (no override) policy. Callers that can override output use
  `validate_with_options`.
- Shape validation lives in `ShapeValidator` (TASK-006), not in
  `DocumentValidator`, avoiding duplicate diagnostics.
- Future output formats will require relaxing or parameterizing the `.png`
  extension check.
