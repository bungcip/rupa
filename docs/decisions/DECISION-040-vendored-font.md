# DECISION-040: Vendored, hash-pinned default font

- **Status**: Accepted
- **Date**: 2026-11-14
- **Related**: TASK-091, DECISION-033, specification §9/§18

## Context

Rupa's core promise is deterministic output for the same source, resources, and
settings. Text broke that promise in two ways: native resolution read whatever
font `fontdb` found on the host, and browser builds had no font at all, so every
`text` primitive reported `R003`. Explicit `font` families were also resolved
against host state rather than a pinned asset.

The environment has no outbound network and tool access to system font
directories/caches was not available, so the repository needed a self-contained
font of its own.

## Decision

1. Vendor **Rupa Sans** regular and bold under the SIL Open Font License 1.1.
   The faces are generated from a 5x7 ASCII outline table by
   `crates/rupa-renderer/assets/generate_rupa_sans.py`; `OFL.txt` is checked in
   next to them.
2. Embed both faces with `include_bytes!` in `rupa-renderer::fonts` and pin the
   family name plus each SHA-256 digest in `EMBEDDED_FONTS`. A unit test parses
   both faces and verifies the digests.
3. Resolution is:
   - empty family or `"Rupa Sans"` -> embedded regular/bold;
   - browser host-installed face, when present -> that face (explicit host
     resource);
   - any other family -> `R003` (`MissingFont`), never a system fallback.
4. Remove the native `fontdb` dependency. Native rendering no longer reads
   system fonts.
5. Keep `rupa_set_font` for browser hosts that want to install an additional
   explicit face; it is no longer required for ordinary text.

## Alternatives considered

- **Keep system fonts for explicit families**: preserves old behavior but leaves
  the determinism claim false for `font "Inter"` and similar sources.
- **Download and vendor a larger OFL face**: not possible without outbound
  network; a generated face keeps the repository self-contained.
- **Bitmap-only text path**: would not reuse `fontdue` or support installed
  browser faces, and would split text handling into two renderers.

## Consequences

- Default `text` output is identical across native hosts and the playground; the
  wasm text golden matches the CLI byte for byte.
- `font "Inter"` and other non-embedded families now fail with `R003` unless a
  browser host explicitly installs that family. Specification examples use
  `"Rupa Sans"`.
- The vendored faces add about 67 KB to the repository and wasm module.
- The face is intentionally small and blocky; improving glyph coverage means
  regenerating the assets and updating the pinned hashes.
- Removing `fontdb` removes a system-dependent dependency from the renderer.
