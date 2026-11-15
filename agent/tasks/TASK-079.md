# TASK-079: End-to-end verification and Milestone 12 close-out

## Task Header

- **Task ID**: TASK-079
- **Title**: Verify the playground in a browser, document it, and close Milestone 12
- **Milestone**: M12 — WASM playground (editor + viewer)
- **Priority**: Critical
- **Dependencies**: TASK-076, TASK-077, TASK-078
- **Status**: Complete

---

## Description

Ran the real page in Chromium against the served build and recorded the
behaviour, then wrote `playground/README.md` and updated the agent documents.

Server: `node playground/serve.mjs` (background), preview registered at
`http://127.0.0.1:5173/`, PID from `netstat` (5096).

---

## Verification performed

| Check | Result |
|-------|--------|
| Page loads, no console errors | `console: []`; only network request is `rupa_wasm.wasm → 200` |
| Starter source renders on load | status `320×240 in 38 ms`, canvas-size `320 × 240`, diagnostics "No problems." |
| Renderer exports usable via page | canvas drawn, download enabled |
| Invalid source (missing radius) | status `1 problem`, dot `error`, one diagnostic: `S005`, "circle is missing required 'radius' property", location `2:1`, help "add 'radius <r>px'" |
| Last valid preview preserved on error | canvas stayed 320×240 while source was broken |
| Fixed source recovers | label switched to `200 × 100`, status ok |
| Examples dropdown | `shapes.rupa` loaded into the editor and rendered `320×220 in 10 ms`; canvas has visible pixels; 0 diagnostics |
| Download PNG click | no console/network errors after click (file itself not inspected) |
| Console warnings | only a Canvas2D `willReadFrequently` warning caused by the verification probe's `getImageData`, not by `app.js` |

Workspace validation is recorded in `agent/STATUS.md` (fmt, clippy, build, and
all tests must pass with `CARGO_INCREMENTAL=0`).

---

## Files Created/Modified

| File | Action | Notes |
|------|--------|-------|
| `playground/README.md` | Create | Run instructions, usage, limits, verification notes |
| `agent/tasks/TASK-079.md` | Create | This document |
| `agent/handoffs/HANDOFF-2026-11-13-M12.md` | Create | Milestone 12 handoff |
| `agent/ROADMAP.md`, `agent/BACKLOG.md`, `agent/STATUS.md`, `agent/CURRENT.md` | Modify | Close Milestone 12 |

---

## Acceptance Criteria

- [x] Page loads with no console errors and renders the starter source
- [x] Typing re-renders the preview (250 ms debounce)
- [x] Compile errors show code, message, position, and help
- [x] The last successful preview survives a failed render
- [x] Examples load from the dropdown and render
- [x] Download button is enabled only after a successful render
- [x] `playground/README.md` documents build, serve, usage, and limits
- [x] Workspace validation commands pass
- [x] Milestone 12 marked complete in ROADMAP/BACKLOG/STATUS

---

## Notes / future work

- No font upload control: `text` reports `R003` unless the host installs bytes
  via `rupa_set_font`.
- No `picture` support in the browser (`R001`).
- Optional follow-ups: deploy the static page to Freebuff Sites, font drop-in
  control, share-by-URL, formatter button.
