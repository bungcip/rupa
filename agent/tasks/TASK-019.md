# TASK-019: Layer definition AST and validation

## Task Header
- **Task ID**: TASK-019
- **Title**: Layer definition AST and validation
- **Milestone**: M3 — Layers and Composition
- **Priority**: Critical
- **Estimated Effort**: 5-7 hours
- **Dependencies**: TASK-018

## Description
Add reusable layer definitions per specification §6.1. A layer definition has
a unique name, its own declared size, and shape children using layer-local
coordinates. Layer instances (`place`) are TASK-020; this task stores and
validates definitions only.

## Files to Create/Modify

| File | Action | Notes |
|------|--------|-------|
| `crates/rupa-syntax/src/ast.rs` | Modify | Add `LayerDefinition`; store on Document |
| `crates/rupa-syntax/src/parser.rs` | Modify | Parse `layer "Name"` blocks |
| `crates/rupa-syntax/src/lib.rs` | Modify | Re-export `LayerDefinition` |
| `crates/rupa-syntax/src/diagnostic.rs` | Modify | Add layer validation codes |
| `crates/rupa-compiler/src/layer.rs` | Create | Layer validation |
| `crates/rupa-compiler/src/lib.rs` | Modify | Export layer module |
| `crates/rupa-compiler/src/validation.rs` | Modify | Validate layers and their children |
| `crates/rupa-compiler/src/lower.rs` | Modify | Ignore definitions until instances exist |
| `crates/rupa-cli/tests/cli.rs` | Modify | End-to-end check of a layer-only document |
| `docs/decisions/DECISION-016.md` | Create | Layer definition conventions |

## Syntax

```rupa
layer "Cloud"
    size 240px x 120px

    circle center at (60px, 60px)
        radius 40px
        fill white

    rectangle at (60px, 60px)
        size 120px x 40px
        fill white
```

## Semantics

- `layer "<name>"` declares a reusable definition; it does not draw by itself.
- Layer names are strings, case-sensitive, and must be non-empty.
- Every layer requires a positive pixel `size`; missing size is an error.
- Layer children use layer-local coordinates (0,0 at the layer's top-left).
- Definitions are stored separately from root document children.
- Duplicate layer names are errors.
- Definitions do not render until instantiated. Rendering layer instances is
  TASK-020.
- Layer definitions may contain the existing primitive types. Nested layer
  definitions are not part of this task.

## Acceptance Criteria

- [x] `LayerDefinition` AST node with name, optional size, child shapes
- [x] `Document` stores layer definitions separately from root children
- [x] Parser handles `layer "Name"` with size and shape children
- [x] Missing layer size produces `S015`
- [x] Non-positive/non-finite layer size produces `S017`
- [x] Duplicate layer names produce `S016`
- [x] Empty layer name produces `S018`
- [x] Non-pixel layer size produces `S007`
- [x] Layer child shapes are semantically validated
- [x] Lowering ignores definitions until instances exist (no phantom shapes)
- [x] CLI `check` accepts valid layer definitions and reports invalid ones
- [x] Unit tests for parsing, validation, lowering, and CLI
- [x] `cargo test --workspace` passes
- [x] `cargo fmt -- --check` passes
- [x] `cargo clippy --all-targets --all-features -- -D warnings` passes

## Specification References

- §6.1: Layer definition
- §4.1: Identifiers and strings (case-sensitive names)
- §17: Graphics IR
- §2.2: Explicit semantics

## Handoff Notes

**Completed Work**:
- Added `LayerDefinition { name, size, children, span }` and stored
  `layers: Vec<LayerDefinition>` separately on `Document`.
- Parser handles `layer "<name>"` blocks with `size` and shape children; shape
  dispatch was factored into `parse_child_shape` for reuse.
- Added layer validation in `rupa-compiler/src/layer.rs`:
  - `S015` missing size, `S016` duplicate name, `S017` invalid size,
    `S018` empty name, `S007` non-pixel size
  - case-sensitive unique names
  - layer children validated through the normal shape/paint path
- Lowering intentionally ignores definitions (no phantom root shapes).
- Recorded storage/validation conventions in
  `docs/decisions/DECISION-016.md`.
- Tests: 2 parser, 8 layer-validation, 1 lowering, and 2 CLI end-to-end tests.

**Remaining Work**:
- None for TASK-019.

**Test Results**:
- `cargo test --workspace`: passes (rupa-syntax 99, rupa-compiler 150,
  rupa-ir 17, rupa-geometry 9, rupa-image 7, rupa-renderer 40,
  rupa-cli 24 CLI + 5 integration).
- `cargo clippy --all-targets --all-features -- -D warnings`: passes.
- `cargo fmt -- --check`: passes.

**Known Issues**:
- Definitions do not render until TASK-020 adds instances.
- Nested layer definitions and clipping are deferred (TASK-023/TASK-025).
- No IR layer store exists yet.

**Exact Next Action**:
- Create `agent/tasks/TASK-020.md` from the backlog and implement layer
  instance placement and transforms.
