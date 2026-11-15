# TASK-081: Parse inline primitive property bodies

## Task Header

- **Task ID**: TASK-081
- **Title**: Shared body state machine for every drawing primitive
- **Milestone**: M13 — Inline primitive properties
- **Priority**: Critical
- **Dependencies**: TASK-080
- **Status**: Complete

---

## Description

Replaced the per-primitive `expect_indent()` with a shared body state machine so
that primitives accept an inline property list, an indented block, or an inline
list continued by an indented block.

`BodyState { Inline, Block, Empty }` plus `at_body_boundary`, `enter_body`,
`enter_optional_body`, and `body_continue` are now used by `rectangle`, `circle`,
`ellipse`, `square`, `triangle`, `polygon`, `line`, `polyline`, `path`, `text`,
`picture`, `pixel`, `brush`, and `place`.

- Inline bodies end at the end of their line; the next primitive is never swallowed.
- An indented block after an inline body continues the same primitive.
- A primitive with no properties reports one `P001` ("Expected an indented block or inline properties").
- `place` keeps its optional body; `place "Dot" at (0px, 0px)` alone stays valid.

---

## Bugs found and fixed

| Bug | Cause | Fix |
|-----|-------|-----|
| `P001 "Unexpected token in rectangle: Dedent"` | `Block` state did not stop after skipping newlines, so the closing `Dedent` was treated as a property | `body_continue` now advances past newlines and then stops on `Dedent`/`Eof` |
| Parser spun forever on a primitive with no body | `check_eof` only matched a literal `Eof` token, and a failed body could leave `pos` one past it | `check_eof` also treats a missing current token as end of file |

---

## Files Created/Modified

| File | Action | Notes |
|------|--------|-------|
| `crates/rupa-syntax/src/parser.rs` | Modify | `BodyState`, body helpers, 14 primitives, `check_eof` |
| `crates/rupa-syntax/tests/inline_properties.rs` | Create | 7 parser tests for the new forms |

---

## Acceptance Criteria

- [x] Inline properties parse for every drawing primitive, `text`, `picture`, and `place`
- [x] Inline properties work inside `layer`, `group`, and `repeat` bodies
- [x] Inline then indented continues one primitive; a sibling line starts a new one
- [x] A bodyless primitive yields exactly one `P001` and parsing terminates
- [x] Block-valued properties still require an indented block
