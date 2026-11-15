# DECISION-001: Diagnostic code format

**Date**: 2026-11-10
**Task**: TASK-004 (AST and basic diagnostics)

## Context

TASK-004's acceptance criteria state that diagnostic codes "follow format
`RXXX` (e.g., `R001`, `R002`)". However, the codebase (committed work from
TASK-002/003 and the existing rupa-compiler skeleton) already uses stable
category-prefixed codes:

| Prefix | Category          | Examples                          |
|--------|-------------------|-----------------------------------|
| `L`    | Lexer errors      | `L001` invalid indentation        |
| `P`    | Parser errors     | `P001` expected token             |
| `C`    | Compiler errors   | `C001` missing document           |
| `S`    | Shape errors      | `S005` missing radius             |
| `T`    | Paint errors      | `T002` unknown named color        |

33 call sites across `rupa-syntax` and `rupa-compiler` already reference
these constants.

## Options Considered

1. **Renumber to a flat `R001…R0xx` sequence** — satisfies the literal task
   wording, but loses category information, forces renumbering of all call
   sites, and makes future codes harder to assign without a central registry.
2. **Prefix the category codes with `R`** (e.g., `R-L001`) — hybrid, but
   ugly and still a breaking change.
3. **Keep category prefixes** (`L/P/C/S/T` + 3 digits) — stable codes are
   already deployed; the specification (§2.4, §22.3) only requires *stable*
   codes with location, explanation, and suggestions; it does not mandate a
   specific literal format.

## Decision

Option 3: keep the category-prefixed format. The specification is treated as
the authoritative source for language behavior (AGENTS.md §4), and it does
not specify the `RXXX` literal. The task doc's "RXXX" is interpreted as
"stable, documented codes"; category prefixes are a strictly more informative
superset.

## Consequences

- Existing code and tests remain valid.
- Codes are documented in `crates/rupa-syntax/src/diagnostic.rs` (`codes`
  module) and must only be added to, never renumbered.
- If a flat `RXXX` scheme is later mandated, a mechanical rename plus a
  mapping table will be required; a test asserting code stability
  (`test_diagnostic_codes_are_stable`) will flag the change.
