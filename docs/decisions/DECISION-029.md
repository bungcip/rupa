# DECISION-029: Optional document size with 256x256 default

## Context

The specification required `size` at file level to appear exactly once. Request:
make `size` optional so trivial sketches need less boilerplate.

## Options considered

1. Keep `size` required.
2. Make `size` optional with a documented default canvas.

## Decision

`size` is optional. When omitted, the document canvas is **256px x 256px**.
When present it must appear at most once and be positive, as before. The
default is fixed and implementation-independent (§2.3, §5).

## Consequences

- Fewer lines for minimal examples; good for LLM authoring.
- Compiler/parser must treat a missing `size` as valid, not a diagnostic.
- The implicit root layer's bounds default to 256x256 when `size` is absent.
- Existing fixtures keep their explicit `size`; no migration required.
