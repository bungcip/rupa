# TASK-070: Group definitions and group-only placement

## Task Header

- **Task ID**: TASK-070
- **Title**: Introduce `group` declarations; `place` targets groups only, so
  placing a layer is a semantic error
- **Milestone**: M10 — Groups, defaults, and CLI output
- **Priority**: High
- **Estimated Effort**: 4–6 hours
- **Dependencies**: TASK-067 (specification), TASK-068 (defaults),
  TASK-069 (`export` removal)

---

## Description

Implement the group/layer split from specification §6.0–§6.9.

- `group "Name"` declares a reusable template that owns primitives, nested
  `place`, `let`, `repeat`, and an optional `size`.
  - Declared `size` anchors the group's bounds at its local origin and clips
    content to those bounds.
  - Without `size`, the bounds are measured from the children
    (`composition_bounds`); measured bounds never clip and never re-anchor
    content — they are only the scale/rotation pivot and the offscreen surface
    size.
- `place "Name" at (x, y)` instantiates a group. Placement properties
  (`z-index`, `opacity`, `scale`, `rotate`, `blend`) are unchanged.
- Placing a layer is a semantic error (`S034`): a layer is a single-use plane
  composited once at the document origin.
- New diagnostics: `S030` duplicate group name, `S031` invalid group name,
  `S032` invalid group size, `S033` unknown group (with suggestion), `S034`
  place targets layer, `S035` recursive group reference. `S019` unknown layer
  and `S021` recursive layer reference remain defined but are legacy.
- Since layers can no longer be placed, the compiler composites each layer once
  as `CompositionChild::Plane` at its `z-index`. Layer declarations gained
  `z-index`, `opacity`, and `blend` so a plane keeps its composition properties.

---

## Files Created/Modified

| File | Action | Notes |
|------|--------|-------|
| `crates/rupa-syntax/src/{token,lexer,ast,parser}.rs` | Modify | `group` keyword, `GroupDefinition`, `GroupInstance` |
| `crates/rupa-syntax/src/parser.rs` | Modify | `parse_group`, optional placement body |
| `crates/rupa-compiler/src/group.rs` | Create | Group, placement, instance and cycle validation |
| `crates/rupa-compiler/src/layer.rs` | Modify | Layer definition/plane validation only |
| `crates/rupa-compiler/src/{validation,lower,lib}.rs` | Modify | Wire group validation and lowering |
| `crates/rupa-ir/src/document.rs` | Modify | `groups`, `Bounds`, `GroupDefinition`, `GroupInstance`, `LayerPlane`, `composition_bounds` |
| `crates/rupa-renderer/src/vector.rs` | Modify | Group instance and layer plane compositing |
| `docs/language-reference.json` | Modify | `group` keyword, group/layer model |
| `examples/**`, `benchmarks/**` | Modify | Group-based fixtures |
| `crates/rupa-*/tests/**`, inline fixtures | Modify | Migrated to groups |

---

## Acceptance Criteria

- [x] `group "Name"` parses with optional `size`, nested `place`, `let`,
      `repeat`, and primitives
- [x] `place` resolves group definitions; an unknown name reports `S033`
- [x] Placing a layer reports `S034` with a suggestion to define a group
- [x] Duplicate group names (`S030`), empty names (`S031`), invalid sizes
      (`S032`), and recursive references (`S035`) are reported
- [x] Groups without `size` use measured child bounds as the transform pivot
      and never clip; groups with `size` clip to the declared bounds
- [x] Layers composite exactly once as planes, honoring `z-index`, `opacity`,
      and `blend`
- [x] `place` without properties is valid (no spurious `P001`)
- [x] Examples, benchmarks, inline fixtures, and the language reference are
      migrated; executable-example and benchmark tests pass
- [x] `cargo fmt -- --check` passes
- [x] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [x] `cargo build --workspace` passes
- [x] `cargo test --workspace` passes

---

## Implementation Notes

- Automatic bounds measurement lives in `composition_bounds` in `rupa-ir`:
  stroke widths are ignored, text is approximated as
  `0.6 * size * chars` wide and `size` tall, pictures use their frame size
  (or zero when unspecified), and raster clear/pixel/brush content is
  excluded.
- Lowering is two-pass: all group children are lowered first, then bounds are
  measured for groups without a declared size, so forward references work.
- `place` bodies are optional, matching the specification's placement form
  `place "Name" at (x, y)` with zero or more indented properties.

---

## Specification References

- §6.0–§6.9 Group and layer model
- §6.2 Group bounds and automatic bounding box
- §6.5 `z-index`
- §15 Diagnostic codes (`S030`–`S035`)

---

## Handoff Notes

**Completed Work**:
- Group declarations, group-only placement, and plane-based layer compositing
  are implemented end to end.

**Remaining Work**:
- TASK-071 (optional layer size, layer `fill` shorthand) and TASK-072
  (inspect output, goldens, milestone close-out).

**Known Issues**:
- Layer declarations already accept `size`, `z-index`, `opacity`, and `blend`;
  the optional-size default and the background `fill` shorthand remain
  TASK-071.
