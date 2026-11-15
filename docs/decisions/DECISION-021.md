# DECISION-021: Static layer-reference cycle detection

**Date**: 2026-11-10
**Task**: TASK-024 (Cycle detection for layer references)

## Context

Specification §6.5 allows nested placements but requires recursive
layer-reference cycles to be rejected. TASK-023 added nested composition plus
a renderer guard so cyclic IR cannot overflow the stack; that guard is not a
language diagnostic and would only fail at render time.

## Decision

- **Graph**: nodes are layer definitions; an edge `A -> B` exists for each
  nested `place "B"` inside layer `A`. Root-level placements are not graph
  edges because the document root cannot be referenced by name.
- **Coverage**: every definition is checked, including definitions that are
  never instantiated. A cycle is a document error even if it is unreachable.
- **Algorithm**: deterministic depth-first search in source order over
  definitions and their children, with `Visiting`/`Done` marks. A back edge to
  a `Visiting` node is a cycle; `Done` nodes are not revisited.
- **Diagnostic**: new stable code `S021` (`RECURSIVE_LAYER_REFERENCE`), severity
  error, reported at the `place` that closes the cycle. The message contains
  the reference path, for example ``recursive layer reference: `A` -> `B` -> `A` ``,
  and the help suggests removing or redirecting a placement.
- **Multiplicity**: one diagnostic is reported per back edge. Independent
  cycles are each reported; a strongly connected component may produce several
  back-edge diagnostics. This is intentional and deterministic.
- **Unknown references**: edges to names that are not defined are skipped.
  `S019` already reports the unknown reference, and cycle detection must not
  cascade a spurious `S021`.
- **Duplicates**: duplicate definition names are already `S016`; cycle
  detection maps each name to its first definition so it cannot loop over
  duplicate-name bookkeeping.
- **Integration**: detection runs as part of `LayerValidator::validate_layers`,
  so `check`, compilation, and rendering all reject the document before
  lowering.
- **Renderer guard retained**: `RenderError::CyclicLayer` from TASK-023 stays as
  defense in depth for hand-built IR that bypasses validation.

## Consequences

- Cyclic documents produce a compile-time error with an actionable cycle path.
- The CLI `render` command writes no output for a cyclic document.
- The renderer guard should now be unreachable through the normal compiler
  pipeline, but remains covered by its own renderer test.
