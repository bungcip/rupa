# TASK-083: Tests, example, and Milestone 13 close-out

## Task Header

- **Task ID**: TASK-083
- **Title**: Prove inline and indented sources are equivalent, document, and close M13
- **Milestone**: M13 — Inline primitive properties
- **Priority**: Critical
- **Dependencies**: TASK-080, TASK-081, TASK-082
- **Status**: Complete

---

## Description

Added regression tests for the new syntax, an inline playground example, a
render-parity test, and closed the milestone in the agent documents.

### Render parity

`crates/rupa-wasm/tests/render.rs::inline_and_indented_sources_render_identically`
renders the same two shapes from an inline source and from the canonical
indented source and asserts the PNG bytes are equal — the inline form is a
spelling, not a different drawing.

### Tests added

| File | Tests | Covers |
|------|-------|--------|
| `crates/rupa-syntax/tests/inline_properties.rs` | 7 | inline per primitive, sibling lines, inline+block continuation, missing body (`P001`), inline inside `layer`/`repeat`, `place` without properties, block-only `fill gradient` |
| `crates/rupa-syntax/tests/format_inline.rs` | 5 | expansion, header geometry, multi-line expansion, comment-only + trailing comments, comments inside blocks; every case asserts idempotency |
| `crates/rupa-wasm/tests/render.rs` | 1 | inline vs indented PNG byte equality |

### Existing example migration

`examples/raster.rupa` wrote `brush size 8px` on the header line. That is now a
normal inline property, so the canonical formatter expands it to `brush` plus an
indented `size 8px`; the example was updated to keep
`checked_in_examples_are_fmt_stable` green. The rendered result is unchanged
(the brush body already accepted `size`).

### Example

`playground/examples/inline-shapes.rupa` is the inline showcase and is listed in
the playground's examples dropdown. It is intentionally **not** in `examples/`:
`checked_in_examples_are_fmt_stable` requires every checked-in example to equal
its canonical (expanded) formatting, which an inline showcase by definition
does not.

---

## Files Created/Modified

| File | Action | Notes |
|------|--------|-------|
| `crates/rupa-syntax/tests/inline_properties.rs` | Create | Parser regression tests |
| `crates/rupa-syntax/tests/format_inline.rs` | Create | Formatter regression tests |
| `crates/rupa-wasm/tests/render.rs` | Modify | Inline/indented byte parity |
| `playground/examples/inline-shapes.rupa` | Create | Inline example |
| `examples/raster.rupa` | Modify | `brush size 8px` expanded to the canonical block form (fmt-stable test) |
| `playground/app.js` | Modify | "Inline properties" dropdown entry |
| `agent/tasks/TASK-080…083.md`, `agent/ROADMAP.md`, `agent/BACKLOG.md`, `agent/STATUS.md`, `agent/CURRENT.md` | Create/Modify | Milestone close-out |
| `agent/handoffs/HANDOFF-2026-11-13-M13.md` | Create | Milestone 13 handoff |

---

## Acceptance Criteria

- [x] Inline and indented sources render byte-identical PNGs
- [x] Parser and formatter regressions covered by new tests
- [x] Formatting is idempotent and keeps comments
- [x] Inline example available in the playground dropdown
- [x] `cargo fmt -- --check`, clippy, build, and workspace tests pass
  (527 passed, 0 failed, 22 suites)
- [x] Milestone 13 recorded in ROADMAP/BACKLOG/STATUS/CURRENT
