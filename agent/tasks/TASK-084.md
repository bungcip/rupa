# TASK-084: Syntax highlighting in the playground editor

## Task Header

- **Task ID**: TASK-084
- **Title**: Colour Rupa source in the playground editor
- **Milestone**: M14 — Playground editing experience
- **Priority**: High
- **Dependencies**: TASK-083
- **Status**: Complete

---

## Description

The editor was a plain `<textarea>` with a line-number gutter. It now has syntax
highlighting, implemented without dependencies and without changing the build:

- `index.html` adds an `<pre id="highlight">` layer behind the textarea.
- `style.css` makes the textarea's text transparent (caret and selection still
  visible) and paints the `<pre>` at the same padding, font, line height and
  tab size, inset past the 48 px gutter.
- `app.js` tokenizes the source and writes escaped, classed spans into the layer.

Token classes: `tok-comment`, `tok-string`, `tok-number` (numbers with units and
bare `#rrggbb` colours), `tok-keyword` (primitives and structural keywords),
`tok-property`, `tok-color` (named colours), `tok-literal` (`true`/`false`).
The word lists mirror the language reference and `KNOWN_KEYWORDS` in
`crates/rupa-syntax/src/diagnostic.rs`; the colour list mirrors `NAMED_COLORS` in
`crates/rupa-compiler/src/paint.rs`, so only real colours are coloured.

Highlighting is repainted on every edit, on Tab-indent, and when an example is
loaded; the layer follows the textarea's vertical and horizontal scroll.

---

## Implementation notes

- Word data is one `WORD_CLASSES` map built from four space-separated lists
  (keyword, property, colour, literal), so there are no parallel sets or a
  classifier function; `WORD_CLASSES.get(word) ?? ""` is the whole lookup.
- `refreshEditor()` refreshes both decorations (gutter + highlight) in one call,
  used by the initial paint, edits, Tab-indent, and example loading.
- `style.css` declares the shared typography once for `.editor__highlight,
  .editor__input`, so the layers cannot drift apart.
- All source text goes through `escapeHtml` (one lookup table), so a `<` in the
  source cannot inject markup.
- A trailing newline is appended to the layer so a final empty line keeps its
  height and the highlight never trails the textarea.
- `TOKEN_PATTERN` is a single ordered regex (comment → string → number/hex →
  word), so `//` inside a string and `#7950F2` are classified correctly.
- Only presentation changed: the textarea remains the source of truth, so
  typing, Tab, Ctrl/Cmd+Enter, diagnostics and rendering behave as before.

---

## Files Created/Modified

| File | Action | Notes |
|------|--------|-------|
| `playground/index.html` | Modify | Highlight layer behind the textarea |
| `playground/style.css` | Modify | Layer geometry, transparent text, token colours |
| `playground/app.js` | Modify | Tokenizer, `updateHighlight`, scroll sync |
| `playground/README.md` | Modify | Document highlighting |
| `agent/tasks/TASK-084.md` | Create | This document |

---

## Verification

| Check | Result |
|-------|--------|
| `node --check playground/app.js` | exit 0 |
| Page loads and renders | status `320×240 in 33 ms` |
| Token counts on the starter source | 5 keyword, 7 property, 10 number, 2 colour, 1 string |
| Textarea text is transparent | `rgba(0, 0, 0, 0)` |
| Comment token | `// a note` → `tok-comment`, `rgb(111, 111, 133)` |
| Layer text equals the source | `true` |
| Console errors | none |
| Re-check after simplification | same token counts (5 keyword, 7 property, 10 number, 2 colour, 1 string); layer text, line height, and padding match the textarea; gutter stays in sync; `pink` (not a Rupa colour) is no longer highlighted |

The page was inspected through the DOM (token classes, computed colours, layer
text). No screenshot was taken.

---

## Acceptance Criteria

- [x] Keywords, properties, numbers/units, strings, comments, named colours, and booleans are coloured
- [x] The textarea stays editable; caret and selection remain visible
- [x] Highlighting keeps alignment with the gutter, line height, and scroll
- [x] No new dependencies, APIs, or build steps
- [x] No console errors; rendering still works
