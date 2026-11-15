# TASK-078: Live preview wiring

## Task Header

- **Task ID**: TASK-078
- **Title**: Load the wasm renderer, re-render debounced edits, show diagnostics, examples, and PNG download
- **Milestone**: M12 — WASM playground (editor + viewer)
- **Priority**: Critical
- **Dependencies**: TASK-076, TASK-077
- **Status**: Complete (browser interaction verified in TASK-079)

---

## Description

`playground/app.js` now drives the whole loop:

- **Loading**: `fetch("rupa_wasm.wasm")` +
  `WebAssembly.instantiateStreaming`, falling back to
  `WebAssembly.instantiate` when streaming is unavailable or the MIME type is
  rejected. Failures are surfaced in the placeholder and diagnostics panel with
  a pointer to `build.sh`.
- **Rendering**: source bytes go through `rupa_alloc`/`rupa_render`; the frame
  header `[u32 total][u32 meta_len]` is read and the bytes are **copied out
  immediately** (any later call can grow memory and detach views), then
  `rupa_free`/`rupa_free_frame` release both buffers in `finally` blocks.
- **Debounce**: typing schedules a render 250 ms later; `Render` and
  Ctrl/Cmd+Enter render immediately. A monotonically increasing generation
  counter discards results and image decodes from superseded renders.
- **Preview**: PNG is decoded with `createImageBitmap`, canvas is resized to the
  document dimensions, and `#canvas-size` shows `W × H`.
- **Diagnostics**: rows show severity, stable code, message, `line:column`, and
  help. `renderError` entries appear with the `render` label. The last
  successful preview stays on screen when a render fails.
- **Status**: busy/ok/error dot plus `W×H in N ms` (with warning count when
  present).
- **Examples**: dropdown loads `playground/examples/*.rupa` (minimal, shapes,
  paints, layers, layer-fill, procedural, raster — copied from `examples/`).
- **Download**: button writes the last successful PNG as `rupa-preview.png`.

---

## Files Created/Modified

| File | Action | Notes |
|------|--------|-------|
| `playground/app.js` | Modify | Module loading, render loop, diagnostics, examples, download |
| `playground/index.html` | Modify | Examples select, Render button, Download PNG button |
| `playground/style.css` | Modify | `.controls`, `.control`, `.visually-hidden` |
| `playground/examples/*.rupa` | Create | Seven example sources for the dropdown |
| `agent/tasks/TASK-078.md` | Create | This document |
| `agent/STATUS.md`, `agent/CURRENT.md`, `agent/BACKLOG.md` | Modify | Progress tracking |

---

## Acceptance Criteria

- [x] Module is fetched and instantiated with a non-streaming fallback
- [x] Every render frees the source buffer and the frame buffer
- [x] Edits debounce at 250 ms; Render button and Ctrl/Cmd+Enter are immediate
- [x] Stale results are discarded by generation guard
- [x] Diagnostics show code, message, position, and help; `renderError` shown
- [x] Last successful preview is kept on failure
- [x] Examples dropdown loads served sources; all seven pass `rupa check`
- [x] Download PNG writes the last successful render
- [x] `node --check playground/app.js` passes; new element ids exist
- [ ] Browser interaction (typing, error recovery, download) — TASK-079

---

## Notes

- The canvas keeps the previous bitmap when a render fails; only diagnostics and
  status change.
- `createImageBitmap` failure would leave the previous preview; the generation
  guard prevents an out-of-order decode from overwriting a newer frame.
