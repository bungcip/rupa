# DECISION-024: Picture placement and fit modes

**Date**: 2026-11-10
**Task**: TASK-027 (Picture placement and fit modes)

## Context

Specification §10 defines `picture "<path>"` with `at`, `size`, and the fit
modes `contain`, `cover`, `fill`, and `none`, but does not fix the anchor or
how fit interacts with transforms, clipping, and layers. TASK-026 produced
canonical straight-alpha RGBA8 images and a `ResourceStore`.

## Decision

- **Frame anchor**: `at (x, y)` is the top-left corner of the picture frame,
  matching `rectangle` and layer placement. `size` is the frame width and
  height. When `size` is omitted, the frame is the image's natural pixel size.
- **Default fit**: `contain`. An omitted `fit` letterboxes and centers inside
  the frame, which never crops content unexpectedly.
- **Fit semantics**:
  - `contain`: uniform scale `min(fw/iw, fh/ih)`, centered.
  - `cover`: uniform scale `max(fw/iw, fh/ih)`, centered, then cropped to the
    frame.
  - `fill`: independent x/y scales to the frame.
  - `none`: scale 1, drawn from the frame's top-left corner.
- **Frame clipping**: every picture is clipped to its frame in device space,
  intersected with enclosing layer clips. This makes `cover` cropping and
  rotated frames correct without compositing to an offscreen surface.
- **Transforms**: rotation and scale pivot on the frame center, consistent
  with primitives and layer instances. Opacity multiplies down the hierarchy
  and is applied to the image draw.
- **Ordering**: pictures participate at z-index `0` alongside direct shapes,
  keeping source order among equal z-index children.
- **Resource resolution at render time**: IR stores the path exactly as
  written. `render_document` refuses pictures (no resolver), and
  `render_document_with_resources` resolves paths through `ResourceStore`.
  The CLI roots the store at the input document's directory. Compilation and
  `check` remain filesystem-free.
- **Sampling**: images are drawn with `tiny_skia`'s `draw_pixmap`, i.e. simple
  sampling. Higher-quality resampling is deferred.

## Consequences

- Picture IR stays backend-independent and serializable; no pixel payloads in
  the IR.
- `check` cannot report missing image files; that diagnostic belongs to
  render-time errors now and to TASK-030 for richer resource validation.
- A picture path is relative to the document, so moving a document or sharing
  it requires keeping its resources alongside it.
- `cover`, `fill`, and rotated frames are clipped exactly, but scale quality is
  basic until resampling is added.
