# DECISION-030: Background defaults to transparent

## Context

Request: confirm that `background` is optional and defaults to transparent.

## Decision

`background` is optional and defaults to **transparent** (fully transparent
black, alpha 0). This formalizes behavior already documented in §5 of the
specification. Rendered PNGs with no `background` therefore have an alpha
channel with transparent pixels wherever nothing is drawn.

## Consequences

- No change to the parser or renderer; this decision records the normative
  default and the PNG contract for omitted backgrounds.
- Golden tests must keep covering transparent-edge output (§23.4).
