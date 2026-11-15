# DECISION-033: Browser (WebAssembly) renderer and playground

> **Update (TASK-091):** DECISION-040 vendors the OFL-licensed Rupa Sans font
> and removes native system-font discovery. Statements below about no vendored
> font and `R003` for all text are historical; text now renders by default in
> both the CLI and the playground.

## Context

The project needs a browser demo: an editor that re-renders Rupa source in a
viewer. The compiler and renderer are pure CPU Rust (`compile_source`,
`render_document`, PNG encoding to any `Write`), so the pipeline can run in
`wasm32-unknown-unknown`. Three constraints differ from the CLI:

1. No filesystem: `picture` resources and system font discovery are unavailable.
2. `fontdb`'s default features (`fs`, `memmap`) do not build for browsers.
3. The playground must not need a JavaScript toolchain, `wasm-bindgen` or
   `wasm-pack` to build.

## Options considered

- **`wasm-bindgen` + `wasm-pack`.** Ergonomic bindings, but adds a second CLI
  toolchain whose version must match the crate, plus generated JS glue.
- **`#[no_mangle] extern "C"` ABI with a length-prefixed buffer.** No extra
  tooling, one small documented `unsafe` boundary, output compared byte for byte
  against the CLI in native tests.
- **Deferred**: server-side rendering via HTTP. Rejected: the demo is meant to
  be static and offline-capable.

## Decision

- New crate `rupa-wasm` (`cdylib` + `rlib`) exposes:
  - safe Rust API `render_source(&str) -> Frame { meta, png }` and
    `encode_frame(&Frame) -> Vec<u8>`;
  - a C ABI in `abi.rs` only: `rupa_alloc`, `rupa_free`, `rupa_render`,
    `rupa_format`, `rupa_free_frame`, and (browser only) `rupa_set_font`.
- Formatting (DECISION-035 syntax) is exported rather than reimplemented in
  JavaScript: `rupa_format` returns the same `[u32 total][u32 meta_len][meta
  JSON][body]` header as `rupa_render`, with the canonical `rupa fmt` text as the
  body (`status`, `diagnostics`, `error` in the metadata). One reader and one
  `rupa_free_frame` serve both results.
- Wire format: `[u32 total][u32 meta_len][meta JSON][PNG bytes]`, one allocation
  per frame, freed by the host.
- Metadata reports `status`, `width`, `height`, `diagnostics[]`
  (`code`, `severity`, `message`, `line`, `column`, `endLine`, `endColumn`,
  `help`) and `renderError`. Resource and font render failures reuse the
  existing stable codes `R001`, `R002`, `R003`; other renderer failures have no
  language code and are reported as `renderError` instead of inventing one.
- Fonts: `rupa-renderer` keeps `fontdb` behind
  `cfg(not(target_arch = "wasm32"))`. Browser builds resolve fonts from an
  in-memory registry (`install_font`) and report `MissingFont` when text is
  rendered without one. No font binary is vendored: the design keeps the crate
  free of a large asset and lets a host supply its own licensed font.
- The renderer no longer panics on pattern-tile allocation failure; it returns
  `RenderError::PatternTile`. A browser trap would otherwise kill the demo.

## Consequences

- The browser path is deterministic and byte-identical to `rupa render` for the
  checked-in `minimal` golden image (covered by a native test).
- `picture` primitives cannot render in the playground; they produce `R001`
  with an explanation.
- `text` requires the host to install font bytes; without one it produces
  `R003` instead of failing silently.
- The ABI is a stable, documented interface; changing it requires updating
  `abi.rs`, the frame header, and the host.
