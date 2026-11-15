# TASK-018: Stroke style properties

## Task Header
- **Task ID**: TASK-018
- **Title**: Stroke style properties (width, cap, join)
- **Milestone**: M2 — Core Vector Geometry
- **Priority**: Critical
- **Estimated Effort**: 5-7 hours
- **Dependencies**: TASK-017

## Description
Implement the core stroke style properties from specification §7.6: width
(already parsed), line cap, and line join. Dash patterns and configurable miter
limits remain future work because the specification says to add dash after
basic stroke rendering is stable.

## Files to Create/Modify

| File | Action | Notes |
|------|--------|-------|
| `crates/rupa-syntax/src/ast.rs` | Modify | Add cap/join to stroke declaration |
| `crates/rupa-syntax/src/parser.rs` | Modify | Parse cap/join on all primitives |
| `crates/rupa-syntax/src/lib.rs` | Modify | Re-export stroke style enums |
| `crates/rupa-compiler/src/lower.rs` | Modify | Apply cap/join to IR stroke style |
| `crates/rupa-renderer/src/vector.rs` | Modify | Verify cap/join mapping/rendering |
| `crates/rupa-cli/tests/cli.rs` | Modify | End-to-end cap/join render |
| `docs/decisions/DECISION-015.md` | Create | Stroke style conventions |

## Syntax

```rupa
line from (100px, 100px) to (500px, 300px)
    stroke black
    width 8px
    cap round
    join round
```

- `cap butt|round|square`
- `join miter|round|bevel`

## Semantics

- Cap/join apply to any shape with a stroke (closed shapes, lines, polylines,
  paths).
- Defaults are `butt` cap and `miter` join, matching the IR defaults.
- `cap`/`join` may appear before or after `stroke`; they configure the stroke
  declaration.
- Cap/join without a stroke are parsed and retained but have no visible effect.
- Width remains positive pixels (`S011`/`S007`).
- Dash and miter-limit syntax are explicitly deferred.
- IR already exposes `LineCap`/`LineJoin`; lowering maps AST enums to IR enums,
  and the renderer maps IR enums to tiny-skia.

## Acceptance Criteria

- [x] AST stroke declaration includes optional cap/join enums
- [x] Parser accepts `cap` and `join` on all stroked primitive blocks
- [x] Canonical values are validated `butt|round|square` and
      `miter|round|bevel`; invalid values are parser errors
- [x] Lowering maps cap/join into IR `StrokeStyle`
- [x] Renderer maps IR cap/join to tiny-skia and renders them
- [x] Unit tests for parser, lowering, and renderer mapping/pixels
- [x] End-to-end CLI test with cap/join
- [x] `cargo test --workspace` passes
- [x] `cargo fmt -- --check` passes
- [x] `cargo clippy --all-targets --all-features -- -D warnings` passes

## Specification References

- §7.6: Stroke properties
- §17: Graphics IR
- §2.1: Canonical syntax

## Handoff Notes

**Completed Work**:
- Added AST `StrokeCap`/`StrokeJoin` enums and optional `cap`/`join` fields on
  the shared `Stroke` declaration.
- Parser accepts `cap butt|round|square` and `join miter|round|bevel` on all
  nine primitive types, merging them with a later `stroke` declaration
  regardless of property order.
- Invalid cap/join values are parser errors (`P003`) with valid options.
- Lowering maps AST cap/join to IR `LineCap`/`LineJoin`; defaults remain
  `butt`/`miter`.
- Renderer already maps IR enums to tiny-skia; added mapping and square-cap
  pixel tests.
- Recorded conventions in `docs/decisions/DECISION-015.md`.
- Tests: 3 parser, 2 lowering, 2 renderer, and 1 CLI end-to-end cap/join test.

**Remaining Work**:
- None for TASK-018. Milestone 2 is complete.

**Test Results**:
- `cargo test --workspace`: passes (rupa-syntax 97, rupa-compiler 141,
  rupa-ir 17, rupa-geometry 9, rupa-image 7, rupa-renderer 40,
  rupa-cli 22 CLI + 5 integration).
- `cargo clippy --all-targets --all-features -- -D warnings`: passes.
- `cargo fmt -- --check`: passes.

**Known Issues**:
- `dash` and `miter-limit` are deferred; IR fields remain at defaults.
- Cap/join without a stroke are parsed but have no visible effect.

**Exact Next Action**:
- Milestone 2 is complete. Create `agent/tasks/TASK-019.md` from the backlog
  and begin Milestone 3 with layer definition AST and validation.
