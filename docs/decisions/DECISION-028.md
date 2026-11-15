# DECISION-028: Unified primitive parsing and lowering

**Date**: 2026-11-12
**Task**: TASK-056 (Document model and root layer specification)

## Context

Primitive parsing and lowering grew by copying per-shape blocks for
`rectangle`, `circle`, `ellipse`, `square`, `triangle`, `polygon`, `line`,
`polyline`, `path`, `picture`, and `text`. Fill, stroke, transform, opacity,
position, size, and error handling are repeated with small variations. The
repetition increases the cost of language changes such as the root-layer
migration and makes diagnostics drift between primitives.

## Options Considered

1. Keep per-shape implementations and only add the new document/root rules.
2. Replace the AST with a single untyped property map.
3. Keep typed shape nodes but extract shared parsing, validation, and lowering
   helpers.

## Decision

Option 3. Keep explicit typed AST nodes for readability and validation, but
introduce shared property descriptors/helpers for common fields:

- parser helpers for position/size, fill, stroke, transform, opacity, and
  unexpected-property diagnostics
- shared validation for closed shapes and paints
- shared lowering of common shape fields into IR

Shape-specific requirements remain explicit (for example a circle's radius or a
path's command sequence). The refactor must preserve diagnostics, IR output, and
golden images.

## Consequences

- Adding a property or diagnostic fix happens in one place.
- Parser and lowering tests must cover every primitive after refactoring.
- This is an internal refactor, not a language change; no new syntax beyond
  DECISION-026 and DECISION-027 is introduced.
- The work is split across TASK-059 (parsing) and TASK-060 (validation/lowering)
  so behavior-preserving changes remain independently verifiable.
