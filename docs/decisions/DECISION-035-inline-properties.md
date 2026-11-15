# DECISION-035: Inline drawing-primitive properties

**Date**: 2026-11-13
**Task**: TASK-080 … TASK-083 (Milestone 13)

## Context

Rupa described drawing primitives with an indented property block:

```rupa
rectangle at (10px, 40px)
    size 80px x 40px
    fill red
```

That is readable for multi-property shapes, but verbose for one-off shapes and
awkward for LLM output and quick playground edits, where a single line is
clearer. Authors reasonably expected to be able to write:

```rupa
rectangle at (10px, 40px) size 80px x 40px fill red
```

Indentation should mean "this block continues the previous primitive", not
"properties are mandatory on their own lines".

## Options considered

1. **Keep block-only properties.** Simple, but no compact form and indentation
   keeps carrying two meanings.
2. **Inline-only properties.** Breaks every existing example, golden image
   source, benchmark, and the whole documentation set.
3. **Inline, block, or both.** Inline properties end at the end of the line; an
   indented block continues the same primitive.

## Decision

Adopt option 3.

- A primitive's body is entered inline when the header line continues, otherwise
  through an indented block (as before). An inline body ends at `Newline`; if the
  next non-blank line is indented, that block continues the same primitive.
- Applies to drawing primitives (`rectangle`, `square`, `circle`, `ellipse`,
  `triangle`, `polygon`, `line`, `polyline`, `path`, `pixel`, `brush`), plus
  `text`, `picture`, and `place`.
- Properties whose value is a block (`fill gradient`, `fill pattern`, `points`,
  path `control`) still require their own indented block.
- Structural declarations (`layer`, `group`, `repeat`, `fn`) keep requiring an
  indented body, so composition semantics are untouched.
- Missing body: one `P001` diagnostic, message "Expected an indented block or
  inline properties".
- Canonical formatting expands inline properties to one property per indented
  line, so `rupa fmt` output is unchanged in layout for existing sources.
- `rupa fmt` now preserves `//` comments (the lexer keeps them as a side channel
  used only by the formatter). Previously formatting silently deleted them; that
  was a defect the playground's Format button would have exposed.

## Consequences

- The parser needs one shared body state machine instead of a bare
  `expect_indent()` per primitive; all primitives use it so none is left behind.
- Golden images, examples, and benchmarks are unaffected: existing sources keep
  their meaning and canonical form.
- The formatter's inline expansion is what makes the playground's Format button
  useful: compact input becomes the documented canonical style.
- Comments survive formatting; comment placement may shift to the end of the
  line that a reflowed property list produced.
- Diagnostics continue to use `P001`; no new stable code is introduced.
