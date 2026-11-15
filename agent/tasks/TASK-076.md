# TASK-076: Playground page (editor + viewer + diagnostics)

## Task Header

- **Task ID**: TASK-076
- **Title**: Create the playground layout: source editor, canvas preview, diagnostics panel
- **Milestone**: M12 — WASM playground (editor + viewer)
- **Priority**: Critical
- **Dependencies**: TASK-075
- **Status**: Complete

---

## Description

Static page shell for the playground. No renderer wiring yet (that is TASK-078);
this task establishes the markup, styling, and editor surface so the next tasks
only add behaviour.

Layout:

- Top bar with product name and a status indicator
  (`#status-dot`/`#status-text`) for render state and timing.
- Left pane: `Source` header, monospace editor with a synced line-number gutter,
  and a `Diagnostics` list whose rows carry severity, code, message, location,
  and help.
- Right pane: `Preview` header with canvas dimensions (`#canvas-size`), a
  checkerboard stage (so transparent backgrounds are visible), the `#preview`
  canvas at 320×240, and a placeholder shown until the renderer runs.
- Responsive: two columns above 900px, stacked rows below.

`app.js` owns the editor surface only: starter source, line numbers, Tab
insertion of four spaces, and gutter scroll sync.

---

## Files Created/Modified

| File | Action | Notes |
|------|--------|-------|
| `playground/index.html` | Create | Page shell, panes, diagnostics list, canvas |
| `playground/style.css` | Create | Dark theme, grid layout, checkerboard stage, diagnostic rows |
| `playground/app.js` | Create | Starter source, gutter, tab indentation |
| `agent/tasks/TASK-076.md` | Create | This document |
| `agent/CURRENT.md`, `agent/BACKLOG.md`, `agent/STATUS.md` | Modify | Progress tracking |

---

## Acceptance Criteria

- [x] Page contains an editor, a canvas viewer, and a diagnostics panel
- [x] Editor is monospace, disables wrapping, and inserts four spaces on Tab
- [x] Gutter line numbers follow the source and scroll with the editor
- [x] Canvas sits on a checkerboard stage; placeholder explains it is not yet live
- [x] Layout is responsive at ≤900px
- [x] `node --check playground/app.js` passes; referenced element ids exist
- [ ] Visual/browser verification — deferred to TASK-079 (no server until TASK-077)

---

## Notes

- The placeholder text ("Waiting for the renderer.") is intentional until
  TASK-078 loads the wasm module.
- `playground/rupa_wasm.wasm` does not exist yet; `app.js` makes no fetch, so the
  page loads cleanly without it.
