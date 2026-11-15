# DECISION-006: PNG output color contract

**Date**: 2026-11-10
**Task**: TASK-009 (PNG encoding)

## Context

The renderer produces premultiplied RGBA8 for compositing (DECISION-005).
PNG stores straight (non-premultiplied) alpha. Specification §18.1 requires a
documented conversion to the output color representation and deterministic
platform-independent behavior. TASK-009 requires exact dimensions, correct
pixels, transparent backgrounds, and sRGB output.

## Decision

- **Encoded color contract**: PNG files contain 8-bit **straight-alpha RGBA**
  in sRGB. For fully opaque images this is byte-identical to the renderer's
  RGB values.
- **Premultiplied → straight conversion** happens at the PNG boundary.
  `rupa_image::encode::unpremultiply_rgba` maps
  `c_straight = round(c_premultiplied * 255 / alpha)` with clamping; fully
  transparent pixels are normalized to `(0, 0, 0, 0)`.
- **Color space**: the encoder emits an sRGB chunk using the perceptual
  rendering intent. No platform-specific color conversion is performed; the
  sRGB bytes from the renderer are written unchanged (aside from alpha
  unpremultiplication).
- **Dimensions**: PNG width/height are exactly the `RenderOutput`/document
  dimensions. Zero dimensions and buffers whose length is not
  `width * height * 4` are rejected before any file is created.
- **Layering**: `rupa-image` owns generic straight-alpha PNG encoding
  (`encode_rgba_png`, `encode_png` for `ImageBuffer`). `rupa-renderer` owns the
  premultiplied renderer boundary (`encode_png`, `encode_render_output`).
- `png = "0.18"` is used in both crates.

## Consequences

- Semi-transparent renders viewed in standard PNG viewers show the intended
  straight-alpha color; renderer bytes for semi-transparent fills differ from
  PNG bytes by design.
- Tests assert both the premultiplied renderer pixel and the unpremultiplied
  PNG pixel for a 50% red fill.
- TASK-010 can call `rupa_renderer::encode_render_output` after rendering.
- Future non-sRGB or 16-bit output requires extending this contract.
