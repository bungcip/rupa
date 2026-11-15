# TASK-058: Implicit root layer in compiler, IR, and renderer

**Milestone**: 9 — Document Model & Root Layer Refactor
**Dependencies**: TASK-057
**Status**: Complete

## Goal

Give every document an implicit root layer and lower all top-level drawing
primitives and placements into it, without changing rendered output for
equivalent documents.

## Scope

- Add the compiler-created root layer:
  - local coordinates = document coordinates
  - bounds = document canvas
  - no clipping beyond the output canvas
  - not placeable or referenceable
- Lower top-level shapes, text, pictures, raster operations, and `place`
  instances into the root layer in source order.
- Preserve root `z-index` and stable source-order semantics within the root layer.
- Reject explicit `layer "root"` definitions with `S029`.
- Update IR document APIs and renderer composition to consume the root layer.
- Update `inspect` data preparation so the root layer is visible to tooling.
- Keep `GraphicsDocument` construction APIs ergonomic for tests and callers.

## Files to Modify

- `crates/rupa-ir/src/document.rs`
- `crates/rupa-compiler/src/lower.rs`
- `crates/rupa-compiler/src/layer.rs`
- `crates/rupa-compiler/src/validation.rs`
- `crates/rupa-renderer/src/vector.rs`
- CLI inspect/benchmark paths that assume direct root children

## Acceptance Criteria

- [x] Every document has exactly one implicit root layer
- [x] Top-level primitives render as root-layer children
- [x] Top-level placements and z-index ordering remain correct
- [x] Nested explicit layers and clipping are unchanged
- [x] `layer "root"` is rejected with `S029`
- [x] Equivalent old/new documents produce pixel-identical output
- [x] `cargo test -p rupa-ir -p rupa-compiler -p rupa-renderer` passes

## Testing

- Root layer existence, bounds, and ownership
- Root primitive and placement ordering
- Mixed root primitives and placements with equal/unequal z-index
- Reserved root-name validation
- Golden/render regression tests

## Implementation Notes

- `GraphicsDocument` now has a public implicit `root: LayerDefinition` with
  reserved name `root`, canvas bounds, vector mode, and `clip: false`.
- `children` moved to `root.children`; accessor methods (`children`,
  `children_mut`, `push_*`, `shapes`, `instances`, render order) delegate to the
  root layer.
- The compiler lowers top-level primitives and placements into the root layer.
- Explicit `layer "root"` definitions are rejected with `S029`; names remain
  case-sensitive, so `"Root"` is allowed.
- The renderer consumes `document.root.children`.
- `rupa inspect` now serializes compiled graphics IR, including the root layer.

## Verification

- `cargo fmt -- --check` — pass
- `cargo clippy --all-targets --all-features -- -D warnings` — pass
- `cargo build --workspace` — pass
- `cargo test --workspace` — pass
- Golden image output remains pixel-identical
- Inspect smoke test reports `root` with canvas bounds and root children
