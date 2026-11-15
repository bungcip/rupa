# Handoff Notes

This directory contains handoff notes from agents when they stop work.

## Naming Convention

`HANDOFF-<timestamp>.md` — e.g., `HANDOFF-2026-11-10T14-30-00Z.md`

## Format

Each handoff note must contain:

1. **Completed Work** — What was finished
2. **Remaining Work** — What's left in the current task
3. **Test Results** — What passes/fails
4. **Known Issues** — Blockers or problems
5. **Exact Next Action** — Specific next step for the next agent

## Example

```markdown
# Handoff - 2026-11-10T14-30-00Z

## Completed Work
- Created workspace Cargo.toml
- Created all 7 crate directories with minimal Cargo.toml and lib.rs

## Remaining Work
- Need to verify `cargo build --workspace` passes
- Need to run fmt/clippy/test checks

## Test Results
- Build not yet tested

## Known Issues
- None

## Exact Next Action
Run `cargo build --workspace` to verify workspace compiles
```