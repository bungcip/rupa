# DECISION-033: Fold decision clarifications into the specification

## Context

Request: incorporate the substance of the DECISION-*.md documents into the
specification wherever it makes the specification clearer and less ambiguous,
instead of leaving the clarifications only in decision documents.

## Options considered

1. Keep decisions separate; the spec stays high-level.
2. Fold clarifying content into the specification and reference decisions from
   it where useful.

## Decision

Fold clarifying content into the specification. Decision documents remain the
record of context and alternatives (per AGENTS.md), but the specification is
the authoritative, self-contained statement of language behavior. Ongoing rule:

- Whenever a decision resolves ambiguity (defaults, conventions, exact
  semantics), the specification must state it explicitly in the relevant
  section; a decision doc alone is not sufficient.
- Decision docs may be referenced for context, but the spec must not defer to
  them for normative content.

## Consequences

- Each spec revision must sweep affected decision docs and fold their
  substance into the spec.
- This run folds DECISION-026–028 and the new DECISION-029–032 substance into
  the specification (defaults table, document model, groups, output contract).
- Reviewers validate spec changes against decision docs.
