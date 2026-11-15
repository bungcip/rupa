# DECISION-022: Clipping layer content to declared bounds

**Date**: 2026-11-10
**Task**: TASK-025 (Layer bounds clipping)

## Context

Specification §6.4 defines a layer's finite local viewport as its declared
`size` and recommends clipping content to those bounds by default. It also
advises against adding a configurable `clip` property until clipping behavior
is fully implemented and tested. §13 requires clipping to have a clearly
defined coordinate space and evaluation order.

Layers can be scaled, rotated, and nested, so a layer's bounds are not
generally an axis-aligned rectangle in device space.

## Decision

- **Always enabled**: every layer instance clips its content to the layer's
  local rectangle `(0, 0)` to `(width, height)`. No `clip` property is added.
- **Device-space masks**: for each instance, the renderer builds a
  canvas-sized `tiny_skia::Mask` by filling the layer rectangle transformed by
  the composed local-to-parent transform. Masks are in device space, so scale
  and rotation clip the transformed rectangle rather than its AABB.
- **Nested intersection**: a nested instance's mask is intersected with the
  enclosing layer's mask by multiplying mask alpha values. Content therefore
  stays inside every enclosing layer, not only its immediate parent.
- **Fills and strokes**: the mask is passed to both `fill_path` and
  `stroke_path`, so stroke width extending past the bounds is clipped too.
- **Antialiased edges**: the mask rectangle is filled with antialiasing, so the
  clip boundary is smooth at subpixel positions.
- **Root composition is not clipped by a layer mask**: root-level shapes and
  instances are bounded only by the document canvas, as before.
- **Independent per instance**: each instance allocates its own mask; no clip
  state is shared between instances of one definition.
- **Opacity and ordering unchanged**: instance opacity still multiplies down
  the hierarchy and z-index ordering is untouched. Clipping is a coverage
  operation only.
- **Vector quality preserved**: content is still rasterized at final device
  resolution; there is no offscreen layer raster, so upscaled instances stay
  sharp.

## Consequences

- A mask the size of the document is allocated per layer instance. This is
  simple, deterministic, and correct for arbitrary transforms; very large
  documents with many instances will trade memory for that simplicity.
- A layer cannot paint outside its declared bounds, matching the finite
  viewport model.
- Configurable clipping and shape-based masks remain future work (§13,
  TASK-046).
