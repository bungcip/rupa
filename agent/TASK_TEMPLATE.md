# Task Template

Use this template for every new implementation task. Copy to `agent/tasks/TASK-XXX.md` and fill in all sections.

---

## Task Header

- **Task ID**: TASK-XXX
- **Title**: <short descriptive title>
- **Milestone**: M<#> — <Milestone Title>
- **Priority**: Critical | High | Medium | Low
- **Estimated Effort**: <hours/days>
- **Dependencies**: TASK-XXX, TASK-YYY (or "None")

---

## Description

<Clear description of what this task accomplishes. Reference specification sections.>

---

## Files to Create/Modify

| File | Action | Notes |
|------|--------|-------|
| `crates/xxx/src/file.rs` | Create | |
| `crates/xxx/src/file.rs` | Modify | |

---

## Acceptance Criteria

**All must pass to mark task complete:**

- [ ] Criterion 1 (specific, measurable)
- [ ] Criterion 2
- [ ] Criterion 3
- [ ] `cargo fmt -- --check` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo build --workspace` passes
- [ ] `cargo test --workspace` passes

---

## Implementation Steps

1. Step 1
2. Step 2
3. Step 3
4. Run validation commands
5. Update documentation

---

## Testing Requirements

- Unit tests: <what to test>
- Integration tests: <what to test>
- Golden images: <if applicable>

---

## Specification References

- Section X.Y: <title>
- Section Z.W: <title>

---

## Handoff Notes

**Completed Work**:
- 

**Remaining Work**:
- 

**Test Results**:
- 

**Known Issues**:
- 

**Exact Next Action**:
-