# TASK-080: Specify inline primitive properties

## Task Header

- **Task ID**: TASK-080
- **Title**: Document and decide inline drawing-primitive properties
- **Milestone**: M13 — Inline primitive properties
- **Priority**: Critical
- **Dependencies**: M12
- **Status**: Complete

---

## Description

Added the language change and its rationale: a drawing primitive may carry its
properties on the header line, in an indented block, or both.

```rupa
rectangle at (10px, 40px) size 80px x 40px fill red
circle center at (150px, 50px) radius 30px fill blue
```

Indentation now means "this block continues the preceding primitive" rather than
"properties are mandatory on their own lines".

---

## Files Created/Modified

| File | Action | Notes |
|------|--------|-------|
| `docs/decisions/DECISION-035-inline-properties.md` | Create | Options, decision, consequences |
| `rupa-specification.md` | Modify | §3 bullet, new §7.0, §7.1 compact example, §2.5 formatter note |
| `docs/language-reference.json` | Modify | Version `0.3.0`, `property_style` annotations |

---

## Acceptance Criteria

- [x] Specification states the inline form, the continuation rule, and the block-only exceptions
- [x] Decision record explains why inline wins without breaking existing sources
- [x] `docs/language-reference.json` advertises inline properties
- [x] Nested block-valued properties (`fill gradient`, `fill pattern`, `points`, path `control`) remain block-only
