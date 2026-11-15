# Agent Workflow — Rupa Development

This document explains how the agent-based development workflow operates for the Rupa project.

---

## How the Workflow Works

The Rupa project uses a **git-backed, agent-agnostic, resumable workflow**. All state lives in the repository — there is no reliance on conversation history or external session state.

### Core Concepts

| File | Purpose |
|------|---------|
| `agent/STATUS.md` | Overall project progress, completed milestones, current milestone, blockers |
| `agent/BACKLOG.md` | Prioritized list of tasks with IDs, dependencies, acceptance criteria |
| `agent/CURRENT.md` | Currently active task, owner, files modified, blockers |
| `agent/tasks/TASK-XXX.md` | Individual task specifications |
| `agent/handoffs/` | Handoff notes from agents stopping work |

### Principles

1. **Git is the single source of truth** — All work state is committed or documented in the repo
2. **Tasks are independently verifiable** — Each has explicit acceptance criteria and passing tests
3. **Dependencies are declared** — In task docs and BACKLOG.md
4. **No silent completion** — Acceptance criteria must be verified before marking done
5. **Docs stay synchronized** — Update STATUS, CURRENT, and task docs when things change
6. **Handoffs are mandatory** — Every stop requires a handoff note

---

## Starting Work as a New Agent

### 1. Inspect Repository State
```bash
git status
```

### 2. Read Core Documentation
```bash
cat AGENTS.md
cat agent/README.md
cat agent/STATUS.md
cat agent/CURRENT.md
cat agent/BACKLOG.md
```

### 3. Check for Active Work
- Read `agent/CURRENT.md` — if a task is active, **do not overwrite it**
- Check `agent/handoffs/` for recent handoff notes

### 4. Pick Next Task
- Read `agent/BACKLOG.md` for prioritized tasks
- Respect dependencies (a task's dependencies must be complete)
- Choose the highest-priority available task

### 5. Read Task Specification
```bash
cat agent/tasks/TASK-XXX.md
```

### 6. Claim the Task
Update `agent/CURRENT.md`:
```markdown
## Current Task
- **Task ID**: TASK-XXX
- **Title**: <task title>
- **Status**: in_progress
- **Owner**: <agent/session identifier>
- **Started**: <timestamp>
- **Files Modified**: (list as you work)
- **Blockers**: (none or description)
```

### 7. Implement
- Write code following AGENTS.md conventions
- Run validation commands frequently
- Update task document with progress

### 8. Verify Acceptance Criteria
- Run all tests
- Run `cargo fmt -- --check`
- Run `cargo clippy -- -D warnings`
- Run `cargo build --workspace`
- Run `cargo test --workspace`

### 9. Complete Task
- Update `agent/STATUS.md` (mark milestone progress)
- Update `agent/BACKLOG.md` (mark task complete)
- Update `agent/CURRENT.md` (clear or set to next task)
- Write handoff note in `agent/handoffs/`
- Commit if criteria fully met

---

## Resuming After Another Agent Stops

### 1. Read the Latest Handoff
```bash
ls -la agent/handoffs/
cat agent/handoffs/HANDOFF-<latest>.md
```

### 2. Check Current State
```bash
cat agent/STATUS.md
cat agent/CURRENT.md
cat agent/BACKLOG.md
```

### 3. Determine Next Action
The handoff note contains:
- **Completed work** — What was done
- **Remaining work** — What's left in the current task
- **Test results** — What passes/fails
- **Known issues** — Blockers or problems
- **Exact next action** — Specific next step to take

### 4. Continue from the Exact Next Action
Do not re-explore or re-plan. Start directly from the specified next action.

---

## Key Rules

| Rule | Enforcement |
|------|-------------|
| Never overwrite active work | Check CURRENT.md first |
| Tasks must be small | Independently verifiable in one session |
| Acceptance criteria are mandatory | Cannot mark complete without verification |
| Validation commands must pass | fmt, clippy, build, test all required |
| Handoffs are required | Every stop = handoff note |
| Commit only verified work | Never commit broken builds |

---

## Example Session Flow

```bash
# 1. New agent starts
git status
cat AGENTS.md agent/README.md agent/STATUS.md agent/CURRENT.md agent/BACKLOG.md

# 2. Picks TASK-003 from BACKLOG.md
cat agent/tasks/TASK-003.md

# 3. Claims task
# (edits CURRENT.md)

# 4. Implements, tests, verifies

# 5. Completes
# (updates STATUS.md, BACKLOG.md, CURRENT.md)
# (writes handoff)
# (commits if verified)

# 6. Stops — next agent reads handoff and continues
```