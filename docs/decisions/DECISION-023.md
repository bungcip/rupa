# DECISION-023: Image decoding and resource loading

**Date**: 2026-11-10
**Task**: TASK-026 (Image resource loading and decoding)

## Context

Specification §10 makes `picture` the only external-image construct and says
the first implementation should support PNG input, with missing resources
producing a clear diagnostic. Several later features (pictures, pattern fills,
raster operations, filters) need a single canonical pixel layout and a way to
load a resource once per document.

## Decision

- **Canonical layout**: decoded images are 8-bit straight-alpha RGBA8,
  row-major, top-left origin. Every input color type is converted during
  decoding so downstream stages handle one layout.
- **PNG only for now**: PNG is decoded with the `png` crate already used for
  output. Other formats are reported as `UnsupportedFormat`; adding them is a
  later decoding-dependency change.
- **Signature detection**: format is decided by the PNG signature, not by the
  file extension. A `.jpg` file containing PNG data decodes, and a `.png` file
  containing other data gets a format error rather than a misleading decode
  error.
- **Normalization**: palettes, grayscale, gray+alpha, and RGB are expanded to
  RGBA with opaque alpha where absent. 16-bit samples are reduced to their most
  significant byte. No gamma or color-space conversion is applied, which keeps
  decoding deterministic and independent of metadata.
- **Path-carrying errors**: `ImageError` names the failing path; a separate
  path-independent `ImageErrorKind` lets callers branch on
  unsupported/decoding/dimension failures.
- **Resolution and caching**: `ResourceStore` resolves relative references
  against a base directory, keeps absolute references as-is, and caches decoded
  buffers by resolved path. Repeated instantiations of one picture therefore
  decode the file once.
- **No canonicalization**: paths are used as written so diagnostics match the
  author's source. The trade-off is that two spellings of one file cache twice.
- **Validation at the boundary**: `ImageBuffer::new` rejects zero dimensions and
  mismatched buffer lengths, so invalid buffers cannot enter the pipeline.

## Consequences

- Picture rendering (TASK-027) can assume RGBA8 and needs no per-format logic.
- Determinism holds for identical files; there is no environment-dependent
  font/color management in the decode path.
- Non-PNG resources are a clear, actionable error rather than silent failure.
- Cache size is unbounded per document; documents with many distinct large
  images hold them all for the render duration.
