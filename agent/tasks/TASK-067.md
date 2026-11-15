# TASK-067: Specification revision — groups, defaults, CLI output

## Status
Complete (specification only; implementation is TASK-068 … TASK-072, Milestone 10).

## Description
Revise `rupa-specification.md` per the user's design request:
- Document `size` optional, default `256px x 256px` (DECISION-029).
- `background` optional, default transparent (DECISION-030, formalized).
- Remove `export` from the language; output determined by the CLI
  (DECISION-031): `--output` overrides the `input.png` default.
- Introduce `group` as a reusable visual template distinct from `layer`
  (DECISION-032): semantic rules table, `place` targets groups only, group
  automatic bounding box with optional explicit size and clipping, layer-level
  background `fill` shorthand, empty layers valid, layers contain primitives
  directly, group nesting via `place` only.
- Fold clarifying decision content into the specification (DECISION-033).

## Notes
- The user's example sketch accidentally included a `document` wrapper; the
  user confirmed it stays removed (one file = one document, DECISION-026).
- The design supersedes the place-a-layer semantics of M3 (DECISION-017/020
  superseded for the placement target). This is a breaking change; all
  implementation migration is backlog work (TASK-068 … TASK-072).

## Acceptance criteria
- [x] Specification updated (defaults table §2.3, §3, §5, §6 rewritten, §7,
      §11, §16, §17, §21, §22, §23, §24 M10, §25)
- [x] Defaults table lists every new default: canvas 256x256, transparent
      background, layer size = canvas, group size = automatic bounding box,
      clipping rules, z-index = 0 with stable source order
- [x] Group local origin vs automatic bounding box resolved and stated (§6.2,
      §6.6, §11, §17)
- [x] Complete worked example with one group and two z-indexed layers (§6.9)
- [x] Layer background `fill` shorthand stated, and `fill` at file top level
      rejected (§3, §6.4, §23.2)
- [x] Group/layer namespaces stated (§4.1)
- [x] Decisions DECISION-029 … DECISION-033 recorded; DECISION-032 carries the
      clarification addendum
- [x] No code changes in this task (spec revision only)
- [x] Backlog updated with implementation follow-ups
