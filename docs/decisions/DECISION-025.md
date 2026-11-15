# DECISION-025: Color space and compositing

**Date**: 2026-11-11
**Task**: TASK-049 (Color space and linear compositing)

## Context

Specification §18.1 asks for a documented internal color representation,
premultiplied alpha for compositing, and linear-light compositing where the
renderer supports it. The renderer is built on `tiny-skia` 0.12, whose
rasterization and blend operations operate on 8-bit sRGB-encoded bytes.

## Decision

- **Internal representation**: the renderer stores premultiplied RGBA8 in
  sRGB (gamma) space, matching `tiny-skia`'s native pixel format. No
  conversion to linear light is performed during rasterization or
  compositing; blend modes therefore compose in sRGB space.
- **IR colors**: `Color` components are straight-alpha sRGB in `0.0..=1.0`.
  Linear-light helpers (`Color::to_linear` / `Color::from_linear` and the
  underlying `srgb_to_linear` / `linear_to_srgb` transfer functions) are
  provided for callers and future backends that want linear-space math.
- **PNG output**: the encoder emits 8-bit straight-alpha RGBA in sRGB with
  an sRGB chunk, unpremultiplying at the boundary (DECISION-006). No
  additional color-space conversion is applied.
- **Pictures and resources**: decoded images remain straight-alpha RGBA8 in
  sRGB with no gamma conversion (DECISION-023).

## Consequences

- Compositing results match `tiny-skia`'s sRGB behavior and remain
  deterministic across platforms.
- Full linear-light compositing is deferred until a backend or conversion
  pass supports it end-to-end; the provided helpers keep that option open
  without changing current output.
- Existing golden images remain valid.
