# DECISION-032: Groups as reusable templates; layers as composition planes

## Context

Request: introduce a new `group` construct with responsibilities distinct from
`layer`, and define the semantic rules between the two. Previously the reusable
template was the layer definition and `place` instantiated layers. The example
sketch in the request mistakenly included a `document` wrapper; the user
confirmed it stays removed (one file = one document, DECISION-026).

## Proposed semantic rules (adopted)

| Feature | Group | Layer |
|---|---|---|
| Contains drawing primitives | Yes | Yes |
| Has its own size | Yes (optional, see below) | Yes (optional, defaults to canvas) |
| Has a local coordinate system | Yes | Yes |
| Can be used multiple times | Yes | No |
| Can be placed with `place` | Yes | No |
| Can have `z-index` (as definition) | No | Yes |
| Can have `opacity` (as definition) | No | Yes |
| Can have blend mode (as definition) | No | Yes |
| Can contain child groups | Via `place` | Via `place` |
| May be empty | Yes | Yes |

## Decisions on the open questions

- **Groups may contain other groups only through `place`.** A `group` block
  cannot nest another `group` block directly; composition is done with
  instances. This reuses one concept instead of adding a new one.
- **Group size is automatic by default.** A group without `size` has a
  bounding box computed from its children in group-local coordinates. An
  explicit `size` overrides the automatic box, defines the local viewport, and
  clips content to it (same clipping rule as layers, §6). This keeps group use
  practical and avoids needless declarations while keeping explicit control
  available.
- **Layers may be empty.** An empty layer is a valid composition plane.
- **Layers may contain primitives directly.** Authors are not forced to wrap
  single-use shapes in a group.
- **Layer-level `fill`.** A `fill` written directly inside a `layer` block is
  shorthand for painting the layer's full bounds with that paint, before all
  other children. It is a background fill, not a primitive.
- **Layer size defaults to the document canvas.** A layer without `size` gets
  the document canvas size (defaulting to 256x256 when the document has no
  `size`; DECISION-029). Explicit sizes still allow independent dimensions.
- **`place` targets group definitions only.** Placing a layer name is a
  semantic error. This replaces the former place-a-layer semantics
  (DECISION-017/020); those decisions are superseded for the placement target.
- **Placed group instances** support `at`, `scale`, `rotate`, `opacity`, and
  `z-index` (z-index scoped to the containing layer, stable order preserved).
- **Composition model.** The implicit root layer and every explicit layer are
  composited planes over the document canvas, ordered by z-index (default
  source order, stable). Top-level primitives and top-level `place` belong to
  the implicit root layer as before (DECISION-027).

## Consequences

- This is a breaking semantic change: existing sources that place layers must
  be migrated to groups. Implementation migration is a follow-up milestone
  (M10 in the specification, backlog TASK-067+).
- The IR gains a group definition/instance pair mirroring layers.
- Cycle detection applies to group references (already static, analogous to
  DECISION-021).

## Clarifications recorded during the specification revision

- **Automatic size never re-anchors content.** The group's local origin is
  always the authored `(0px, 0px)`. The automatic bounding box is a measured
  extent in local coordinates, used for reporting and as the rotation/scale
  origin; `place ... at` always positions the local origin. An explicit `size`
  is a viewport anchored at the local origin and clips content outside
  `(0, 0)-(w, h)`.
- **Omitted `z-index` is `0`.** Ties keep source declaration order, and the
  implicit root plane counts as declared before any explicit layer. Top-level
  content therefore sits behind a default-z-index layer; going behind it needs
  a negative z-index.
- **Group and layer names are separate namespaces**, because `place` resolves
  groups only.
- **`fill` inside a layer** is the layer background shorthand; `fill` at file
  top level is invalid, since the document uses `background`.
