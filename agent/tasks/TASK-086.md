# TASK-086: Format button and Milestone 14 close-out

## Task Header

- **Task ID**: TASK-086
- **Title**: Wire the playground Format button to `rupa_format`, verify in the browser, close M14
- **Milestone**: M14 — Playground editing experience
- **Priority**: Critical
- **Dependencies**: TASK-084, TASK-085
- **Status**: Complete

---

## Description

`playground/rupa_wasm.wasm` was rebuilt so the module exports `rupa_format`
(1,277,458 bytes). The editor header now has a `Format` button next to `Render`.
Clicking it:

1. calls `formatSourceInModule(editor.value)` → `rupa_format` through the ABI,
2. on `status: "ok"` replaces the buffer with the canonical text, clamps the
   selection (formatting changes offsets), refreshes gutter + highlight, and
   re-renders immediately,
3. on `status: "error"` leaves the buffer untouched and shows the returned
   diagnostics with status `Cannot format`.

The button is disabled until the module loads, like `Render`.

### ABI refactor

`renderSourceInModule` and `formatSourceInModule` now share `callWithSource`,
which allocates the source buffer, calls the export, copies the frame out
before any later call can grow memory, and frees both buffers. Render and format
return the same frame header, so one reader serves both. The caller passes the
export function itself (`moduleExports.rupa_render` / `.rupa_format`) rather than
a name string.

### Gutter bug fixed during verification

The line-number column had no `white-space: pre`, so the newlines between numbers
collapsed and each row wrapped into two numbers (`1 2`, `3 4`, …). The gutter
now declares `white-space: pre`; numbers render one per line, top-aligned with
the editor text (both 110 px from the pane top) and right-aligned inside the
48 px column. This was a pre-existing M12 styling gap, only visible once the
editor was inspected closely.

### Review pass (same task)

- `formatEditor` no longer duplicates its tail: the buffer is replaced only
  when the text actually changed, then one `clearTimeout` + `render()` runs for
  both outcomes.
- Replacing `value` scrolls a textarea back to the top, so `scrollTop` and
  `scrollLeft` are restored before `refreshEditor()` repaints the highlight
  (otherwise the coloured layer would sync to the wrong offset).
- Caret: a caret that sat at the end now follows to the end of the formatted
text; any other caret is clamped to the new length (formatting changes offsets,
so line/column re-mapping would be guesswork). Verified: caret at end → at end;
caret at column 24 of an inline header → still 24, inside the text.

---

## Files Created/Modified

| File | Action | Notes |
|------|--------|-------|
| `playground/index.html` | Modify | `#format` button in the editor controls |
| `playground/app.js` | Modify | `callWithSource`, `formatSourceInModule`, `formatEditor`, button wiring |
| `playground/rupa_wasm.wasm` | Rebuild | Now exports `rupa_format` (git-ignored) |
| `playground/README.md` | Modify | Format button behaviour |
| `agent/tasks/TASK-086.md` | Create | This document |
| `agent/ROADMAP.md`, `agent/BACKLOG.md`, `agent/STATUS.md`, `agent/CURRENT.md` | Modify | Close Milestone 14 |
| `agent/handoffs/HANDOFF-2026-11-13-M14.md` | Create | Milestone 14 handoff |

---

## Verification

| Check | Result |
|-------|--------|
| `bash playground/build.sh` | OK, module 1,277,458 bytes |
| `node --check playground/app.js` | exit 0 |
| Format enabled after load | `#format.disabled === false` |
| Format on inline source | buffer became `size 200px x 100px` + blank line + `rectangle at (10px, 10px)` / `    size 80px x 40px` / `    fill red`; `// shapes` kept; gutter 7 lines; render `200×100 in 1 ms` |
| Highlight after format | 13 spans, no console errors |
| Format on tab-indented source | buffer unchanged (`\tsize 20px x 20px`), status `Cannot format`, diagnostic `L002` "Tab characters are not allowed" at `1:1` with help |
| Console | none |
| Caret at end after format | `selectionStart === value.length`, text ends with `fill red` |
| Caret mid-document | stays at 24 (clamped, never past the text) |
| Scroll preserved | `editor.scrollTop` 220 kept, highlight layer synced to 220 |
| Error path | buffer and caret unchanged (`\tsize 20px x 20px`, caret 5) |
| Gutter after fix | `white-space: pre`, 12 numbers, one per line, text top 110 px == editor text top 110 px, right edge 48 px == editor left edge |

Verification was done through the DOM and real button clicks; no screenshot was
taken. Rust is untouched in this task, so the workspace validation from TASK-085
still applies (fmt/clippy clean, 535 tests passed).

---

## Acceptance Criteria

- [x] Format button calls the module formatter, not JavaScript re-layout
- [x] Canonical output matches `rupa fmt` (inline expanded, comments preserved)
- [x] A buffer that cannot be formatted is not modified and its diagnostics are shown
- [x] Editor decorations (gutter, highlighting) refresh after formatting
- [x] Render happens immediately after a successful format
- [x] Milestone 14 recorded in ROADMAP/BACKLOG/STATUS/CURRENT + handoff
