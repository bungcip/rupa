# TASK-020: Layer instance placement and transforms

## Task Header
- **Task ID**: TASK-020
- **Title**: Layer instance placement and transforms
- **Milestone**: M3 — Layers and Composition
- **Priority**: Critical
- **Estimated Effort**: 8-10 hours
- **Dependencies**: TASK-019

## Description
Implement `place` layer instances per specification §6.2. An instance refers
to a layer definition by name, places it at a document position, and applies
independent scale/rotation/opacity. Definitions are not mutated by instances.

This task also introduces IR layer definitions/instances and renders multiple
instances independently. Z-index sorting is TASK-022; local-coordinate and
clipping details are TASK-021/TASK-025.

## Files to Create/Modify

| File | Action | Notes |
|------|--------|-------|
| `crates/rupa-syntax/src/ast.rs` | Modify | Add `LayerInstance` and `DocumentChild::Place` |
| `crates/rupa-syntax/src/parser.rs` | Modify | Parse `place "Name" at` blocks |
| `crates/rupa-syntax/src/lib.rs` | Modify | Re-export instance type |
| `crates/rupa-syntax/src/diagnostic.rs` | Modify | Add unknown-layer/z-index codes |
| `crates/rupa-compiler/src/layer.rs` | Modify | Validate instances and references |
| `crates/rupa-compiler/src/validation.rs` | Modify | Dispatch instance validation |
| `crates/rupa-compiler/src/lower.rs` | Modify | Lower definitions and instances |
| `crates/rupa-ir/src/document.rs` | Modify | Add IR layer definitions/instances |
| `crates/rupa-ir/src/lib.rs` | Modify | Re-export layer IR |
| `crates/rupa-renderer/src/vector.rs` | Modify | Render instances with group transforms |
| `crates/rupa-cli/tests/cli.rs` | Modify | End-to-end two-instance render |
| `docs/decisions/DECISION-017.md` | Create | Instance transform conventions |

## Syntax

```rupa
layer "Cloud"
    size 240px x 120px
    circle center at (120px, 60px)
        radius 40px
        fill white

place "Cloud" at (100px, 100px)
    scale 0.8
    rotate 5deg
    opacity 0.9
    z-index 10
```

## Semantics

- `place "<name>" at (x, y)` instantiates a layer definition at `(x, y)`.
- The instance position is the layer's top-left local origin.
- Instance scale/rotation apply around the layer's declared bounds center.
- Instance opacity multiplies each layer child's opacity.
- Instances do not mutate the definition or other instances.
- References resolve independent of declaration order; unknown names are
  semantic errors.
- `z-index` is parsed and validated as an integer, but stable sorting is
  implemented in TASK-022. For now instances render in source order after root
  shapes.
- Layer-local coordinates are used by definition children; instance placement
  translates them into document space.
- No clipping yet; TASK-025 adds layer bounds clipping.

## Acceptance Criteria

- [x] `LayerInstance` AST and `DocumentChild::Place`
- [x] Parser handles `place "Name" at (x, y)` plus transform and z-index
- [x] Missing `at` is `S003`; empty name `S018`; unknown layer `S019`
- [x] Non-integer `z-index` is `S020`
- [x] Instance transforms reuse `S014` validation
- [x] IR has layer definitions and instances
- [x] Lowering resolves references independent of declaration order
- [x] Renderer draws multiple instances independently at correct positions
- [x] Instance scale/rotation pivot on layer bounds center
- [x] Instance opacity multiplies child opacity
- [x] Unit tests for parser, validation, lowering, IR, and renderer
- [x] End-to-end CLI test with two instances of one definition
- [x] `cargo test --workspace` passes
- [x] `cargo fmt -- --check` passes
- [x] `cargo clippy --all-targets --all-features -- -D warnings` passes

## Specification References

- §6.2: Layer instances
- §6.6: Forward references
- §11: Transformations and opacity
- §17: Graphics IR

## Handoff Notes

**Completed Work**:
- Added AST `LayerInstance` and `DocumentChild::Place`.
- Parser handles `place "<name>" at (x, y)` with rotate/scale/opacity/z-index;
  missing position is retained for semantic validation.
- Added `S019` unknown layer and `S020` invalid z-index; instances reuse
  `S003`, `S018`, and `S014`.
- Added layer/instance IR types to `GraphicsDocument`, with definitions
  sharing the paint store and instances storing group transforms.
- Lowering resolves definitions and instances, including forward references.
- Renderer applies instance group transforms around layer bounds center,
  multiplies instance opacity into child opacity, and renders multiple
  instances independently.
- Recorded conventions and temporary source-order limitation in
  `docs/decisions/DECISION-017.md`.
- Tests: 2 parser, 5 layer/instance validation, 2 lowering, 1 IR, 4 renderer,
  and 2 CLI end-to-end instance tests.

**Remaining Work**:
- None for TASK-020.

**Test Results**:
- `cargo test --workspace`: passes (rupa-syntax 101, rupa-compiler 157,
  rupa-ir 18, rupa-geometry 9, rupa-image 7, rupa-renderer 44,
  rupa-cli 26 CLI + 5 integration).
- `cargo clippy --all-targets --all-features -- -D warnings`: passes.
- `cargo fmt -- --check`: passes.

**Known Issues**:
- Root shapes render before all instances; z-index interleaving is TASK-022.
- Layer clipping is TASK-025.
- Nested place instances are TASK-023.
- Instance local-coordinate details are formalized in TASK-021.

**Exact Next Action**:
- Create `agent/tasks/TASK-021.md` from the backlog and implement local
  coordinate system handling.
