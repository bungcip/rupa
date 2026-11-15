# AGENTS.md — General Instructions for AI Coding Agents

## Project: Rupa — Compiled Graphics Programming Language

This document provides mandatory instructions for any AI agent working on the Rupa compiler and renderer. Treat the [rupa-specification.md](rupa-specification.md) as the authoritative source for language behavior and design decisions.

---

## 1. Project Principles

| Principle | Description |
|-----------|-------------|
| **LLM-friendly first** | Predictable syntax, few alternatives, explicit semantics, easy error correction |
| **Human-readable** | Concise English keywords, consistent visual structure |
| **Machine-verifiable** | Strict grammar, static validation, deterministic rendering, useful diagnostics |
| **Composable** | Reusable group definitions instantiated many times |
| **Hybrid graphics** | Vector geometry and raster operations coexist |
| **Deterministic** | Same source + resources + settings = same output |
| **Incremental** | Implement in small, testable milestones |
| **File-scoped documents** | One `.rupa` file is exactly one document; drawing belongs to layers |

### Explicit Non-Goals (Do Not Implement)
- Animation, timelines, keyframes, easing
- Interactive event handlers (mouse, keyboard, pointer)
- Real-time rendering, physics simulation
- SVG import/export (not part of this specification)
- Multiple output formats beyond PNG (initially)

---

## 2. Coding Conventions

### Rust Workspace Structure
```
rupa/
├── Cargo.toml
├── crates/
│   ├── rupa-syntax/       # lexer, parser, AST
│   ├── rupa-compiler/     # name resolution, semantic analysis, lowering
│   ├── rupa-ir/           # Graphics IR and validation
│   ├── rupa-geometry/     # geometry, paths, transforms
│   ├── rupa-renderer/     # CPU rendering and compositing
│   ├── rupa-image/        # image resources and pixel buffers
│   └── rupa-cli/          # check, fmt, render, inspect, bench
├── examples/
├── benchmarks/
├── tests/
└── docs/
```

### Code Style
- **Edition**: Rust 2024 or later
- **Formatting**: `cargo fmt` (standard rustfmt)
- **Linting**: `cargo clippy -- -D warnings`
- **Testing**: `cargo test` (unit + integration)
- **No `unsafe`** unless absolutely necessary and documented
- **No cyclic dependencies** between crates
- **Public APIs narrow** — prefer internal visibility

### Error Handling
- Use `Result<T, E>` with custom error types
- Errors must include: stable code, file/line/column, short explanation, suggested correction
- Never silently ignore misspelled properties or unknown keywords

### Testing Requirements
- Every milestone must compile and pass tests
- Golden image tests for renderer output
- Lexer/parser tests for valid and invalid syntax
- Semantic tests for name resolution, cycles, invalid properties
- Determinism tests for PRNG and layer ordering

---

## 3. Required Validation Commands

Before marking any task complete, run:

```bash
# Format check
cargo fmt -- --check

# Lint check
cargo clippy --all-targets --all-features -- -D warnings

# Build all crates
cargo build --workspace

# Run all tests
cargo test --workspace
```

**All must pass.** Do not commit if any fail.

---

## 4. Rules for Modifying Specifications and Architecture

1. **Never silently change an established language design.** If the specification is ambiguous or wrong, document the decision in `docs/decisions/` and update the specification.
2. **Record significant architectural decisions** in `docs/decisions/DECISION-XXX.md` with:
   - Context
   - Options considered
   - Decision
   - Consequences
3. **Do not duplicate the entire language specification** in code comments. Reference `rupa-specification.md` sections instead.
4. **Avoid unnecessary abstractions and premature optimization.** Implement the minimal vertical slice first.

---

## 5. Rules for Handling Incomplete Work and Handoffs

### Before Starting Work
- Inspect Git status: `git status`
- Read `agent/STATUS.md`, `agent/CURRENT.md`, `agent/BACKLOG.md`
- Read the relevant task document in `agent/tasks/`
- Check `agent/handoffs/` for notes from previous agents
- **Never overwrite another agent's active work**

### During Work
- Update `agent/CURRENT.md` with active task, status, files modified, blockers
- Keep task scope small — each task must be independently verifiable
- Declare dependencies explicitly in task documents
- Run validation commands frequently

### Before Stopping
- Update `agent/STATUS.md` with completed milestones, current milestone, blockers
- Update `agent/CURRENT.md` with progress, remaining work, exact next action
- Write a handoff note in `agent/handoffs/HANDOFF-<timestamp>.md` containing:
  - Completed work
  - Remaining work
  - Test results
  - Known issues
  - Exact next action
- Leave the repository in a consistent state (no broken builds)
- Commit completed work when appropriate (see below)

### Commit Guidelines
- **Commit when**: A task's acceptance criteria are fully verified
- **Do not commit**: Incomplete work, broken tests, speculative changes
- **Commit message format**: `<crate>: <short description>` (e.g., `rupa-syntax: add indentation-aware lexer`)
- Stage only intended files; never commit secrets or generated artifacts

---

## 6. Agent Continuity Requirements

| Requirement | Enforcement |
|-------------|-------------|
| Git is single source of truth | All state in repo; no reliance on conversation history |
| Tasks independently verifiable | Acceptance criteria + passing tests |
| Dependencies declared | In task documents and `BACKLOG.md` |
| No silent completion | Acceptance criteria must be verified |
| Docs synchronized | Update `STATUS.md`, `CURRENT.md`, task docs on changes |
| No design drift | Decisions recorded in `docs/decisions/` |
| Handoffs mandatory | Every stop requires a handoff note |

---

## 7. Workflow Summary

```
1. Read AGENTS.md, agent/README.md, agent/STATUS.md, agent/CURRENT.md
2. Pick next task from agent/BACKLOG.md (respecting dependencies)
3. Read task document in agent/tasks/TASK-XXX.md
4. Update agent/CURRENT.md with task assignment
5. Implement, test, verify acceptance criteria
6. Update agent/STATUS.md, agent/BACKLOG.md, task document
7. Write handoff in agent/handoffs/
8. Commit if criteria met
9. Repeat
```

---

## 8. Current Milestone Priority

**Milestones 1–10 are complete.** The project is closed through **Milestone 10 —
Groups, defaults, and CLI output** (see `agent/ROADMAP.md` and
`agent/BACKLOG.md`).

M10 changes the source model:
- Optional document `size`/`background` with defaults; `export` removed; output
  path from the CLI.
- `group` as a reusable template; `place` targets groups only; layers are
  single-use planes composited once at their `z-index`.
- Layer `size` optional (canvas default); layer-level `fill` is a background
  shorthand; empty layers are valid planes.
- TASK-067 through TASK-072 are complete. Milestone 10 is closed.

**Next task:** none scheduled. Propose follow-up work as new backlog entries.