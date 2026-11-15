# TASK-077: Build and serve the playground

## Task Header

- **Task ID**: TASK-077
- **Title**: Zero-dependency build and static serve scripts for the playground
- **Milestone**: M12 — WASM playground (editor + viewer)
- **Priority**: Critical
- **Dependencies**: TASK-075, TASK-076
- **Status**: Complete

---

## Description

Two scripts make the playground runnable without npm, bundlers, or
`wasm-pack`:

- `playground/build.sh` — ensures the `wasm32-unknown-unknown` target is
  installed, builds `rupa-wasm` in release mode with
  `CARGO_INCREMENTAL=0` (required on this UNC-mounted workspace), and copies
  the module to `playground/rupa_wasm.wasm`.
- `playground/serve.mjs` — Node static server for the `playground/` directory
  on `127.0.0.1:5173` (override with `HOST`/`PORT`). It serves
  `application/wasm` for the module, refuses paths outside the root, answers
  `HEAD`, and exits cleanly on SIGINT/SIGTERM.

The built module is ignored by git.

Browsers refuse `WebAssembly.instantiateStreaming` for `file://`, so the page
must be served over HTTP with the correct module MIME type.

---

## Files Created/Modified

| File | Action | Notes |
|------|--------|-------|
| `playground/build.sh` | Create | Target setup, release build, copy artifact |
| `playground/serve.mjs` | Create | Static server with MIME map and traversal guard |
| `.gitignore` | Modify | Ignore `playground/*.wasm` |
| `agent/tasks/TASK-077.md` | Create | This document |
| `agent/STATUS.md`, `agent/CURRENT.md`, `agent/BACKLOG.md` | Modify | Progress tracking |

---

## Acceptance Criteria

- [x] `bash playground/build.sh` produces `playground/rupa_wasm.wasm`
      (1,258,052 bytes)
- [x] Server serves `/` as `text/html; charset=utf-8`
- [x] Server serves `/rupa_wasm.wasm` as `application/wasm`
- [x] `app.js` and `style.css` return 200 with correct MIME types
- [x] Missing files return 404
- [x] Path traversal (`/../Cargo.toml`) returns 404
- [x] `HEAD` requests return 200
- [x] Built wasm is not tracked by git
- [x] `node --check playground/serve.mjs` passes

---

## Notes

- This environment exports `PORT=0`; the server only honours an explicit
  positive `PORT` and otherwise binds 5173, logging the actual bound port.
- No renderer wiring exists yet (TASK-078); the page still shows its
  placeholder until the module is loaded.
