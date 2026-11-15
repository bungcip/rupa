# TASK-071: Layer semantics update

## Task Header

- **Task ID**: TASK-071
- **Title**: Optional layer size defaulting to the canvas, background `fill`
  shorthand, empty layers, single-use planes
- **Milestone**: M10 — Groups, defaults, and CLI output
- **Priority**: High
- **Estimated Effort**: 3–4 hours
- **Dependencies**: TASK-070 (groups and group-only placement)

---

## Description

Bring layer definitions in line with specification §6.4 and §6.6.

- `size` inside a layer is **optional**. When omitted, the layer's bounds equal
  the document canvas (which itself defaults to `256px x 256px`). `S015` is
  marked legacy and is never emitted.
- A `fill` written directly inside a `layer` block is the **background fill
  shorthand**: it paints the layer's full bounds before any other child and is
  not a primitive. It may appear at most once per layer; a repeat reports the
  new diagnostic `S036` (`DUPLICATE_LAYER_FILL`).
- Layers may be empty: an empty layer is still a valid composition plane and
  emits a plane child with no content.
- Layers remain single-use planes: `place` never targets a layer (`S034`), a
  layer block cannot contain another `layer` declaration, and duplicate layer
  names report `S016`.

Lowering implements the background fill by prepending a rectangle covering
`(0, 0)..(width, height)` with the parsed paint, so ordering, clipping, raster
mode, opacity, and blend behaviour stay inside the existing plane path.

---

## Files Modified

| File | Action | Notes |
|------|--------|-------|
| `crates/rupa-syntax/src/ast.rs` | Modify | `LayerDefinition.fill: Option<Fill>` |
| `crates/rupa-syntax/src/parser.rs` | Modify | Parse layer background `fill`; duplicate `S036` |
| `crates/rupa-syntax/src/diagnostic.rs` | Modify | `S036`; `S015` legacy; clearer `S003`/`S004` text |
| `crates/rupa-compiler/src/layer.rs` | Modify | Optional size; layer fill validation; new tests |
| `crates/rupa-compiler/src/validation.rs` | Modify | Validate layer background paint |
| `crates/rupa-compiler/src/lower.rs` | Modify | Canvas default size; prepend background rectangle |
| `crates/rupa-cli/tests/cli.rs` | Modify | End-to-end default-size + fill rendering test |
| `docs/language-reference.json` | Modify | Layer size/background-fill description |
| `examples/layer-fill.rupa` | Create | Gradient background fill, omitted size, empty layer |

---

## Acceptance Criteria

- [x] A layer without `size` lowers with the document canvas dimensions
- [x] A layer without `size` is valid source (no `S015`)
- [x] `fill` directly inside a layer paints its full bounds before children
- [x] A second layer background `fill` reports `S036`
- [x] Empty layers are valid and lower to an empty plane child
- [x] Layers remain single-use: `place` targets groups only (`S034`), and a
      nested `layer` declaration is rejected
- [x] `rupa check` / `rupa render` agree on the new behaviour end to end
- [x] `cargo fmt -- --check` passes
- [x] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [x] `cargo build --workspace` passes
- [x] `cargo test --workspace` passes

---

## Specification References

- §6.0 Semantic rules for groups and layers
- §6.4 Layer definitions (optional size, background `fill` shorthand)
- §6.6 Group and layer bounds, clipping
- §6.9 Complete example: one group, two layers

---

## Handoff Notes

**Completed Work**:
- Optional layer size, layer background `fill` shorthand, and empty layers are
  implemented and tested end to end.

**Remaining Work**:
- TASK-072: migrate formatter/inspect/reference JSON/goldens to the group
  model and close Milestone 10.

**Known Issues**:
- The formatter still does not emit `layer`/`group` declarations; that gap is
  tracked by TASK-072.
