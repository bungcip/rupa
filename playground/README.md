# Rupa Playground

Edit Rupa source on the left; the preview on the right re-renders as you type.
Compilation happens entirely in the browser through a WebAssembly build of the
same compiler and renderer the CLI uses, so the output matches `rupa render`.

## Run it

```bash
# 1. Build the wasm module (once, and after Rust changes)
bash playground/build.sh

# 2. Serve the page (no dependencies)
node playground/serve.mjs
# → http://127.0.0.1:5173/
```

`build.sh` installs the `wasm32-unknown-unknown` target if needed, builds
`rupa-wasm` in release mode, and copies the module to
`playground/rupa_wasm.wasm` (git-ignored).

`serve.mjs` binds `127.0.0.1:5173`; set `HOST` or a positive `PORT` to change
that. Serving over HTTP matters: browsers refuse
`WebAssembly.instantiateStreaming` for `file://`, and the module must be sent as
`application/wasm`.

## Using it

- **Syntax highlighting**: the editor colours keywords, properties, numbers
  and units, strings, comments, named colours, and booleans. The word classes,
  named colours, and unit suffixes come from the module's `rupa_metadata()`
  export, so the highlighter cannot drift from the lexer or compiler. A
  transparent textarea sits over a coloured `<pre>` layer, so editing behaves
  normally.
- **Format**: the `Format` button rewrites the buffer with the same formatter
  `rupa fmt` uses (inline properties become one per indented line, comments are
  kept) and re-renders. If the source cannot be formatted, the buffer is left
  untouched and the diagnostics explain why.
- **Live render**: edits render 250 ms after you stop typing.
- **Immediate render**: the `Render` button or Ctrl/Cmd+Enter.
- **Examples**: the dropdown loads sources from `playground/examples/`, including
  `text.rupa`, which renders with the vendored default font.
- **Diagnostics**: errors and warnings list their stable code (`S005`, `R001`,
  …), message, `line:column`, and help text. The last successful preview stays
  visible while the current source is broken.
- **Download PNG**: saves the current preview as `rupa-preview.png`.
- **Tab** inserts four spaces.

## Limits

- **Pictures**: `picture` needs file access, which a browser page does not have,
  so it reports `R001`. Put images in the document for the CLI, or omit them in
  the playground.
- **Text**: works offline with the vendored Rupa Sans regular and bold faces;
  omit `font` or use `font "Rupa Sans"`. Native builds do not read system fonts,
  so an unrecognised family reports `R003`. The module still exposes
  `rupa_set_font` (backed by `rupa_renderer::install_font`) for future uploads.
- **Nothing persists**: the source lives in the page only, so a reload returns
  to the starter source. The page makes no requests beyond its own assets.

## How it fits together

| File | Role |
|------|------|
| `index.html` | Two-pane layout: editor + diagnostics, preview + download |
| `style.css` | Dark theme, checkerboard preview stage, responsive layout |
| `app.js` | Module loading, ABI calls, debounce, canvas drawing, examples, download, syntax highlighting from `rupa_metadata()` |
| `serve.mjs` | Zero-dependency static server with correct MIME types |
| `build.sh` | wasm release build and artifact copy |

The browser contract is documented in `docs/decisions/DECISION-033-wasm-playground.md`
and `crates/rupa-wasm/src/abi.rs`:

| Export | Returns |
|--------|---------|
| `rupa_render(ptr, len)` | `[u32 total][u32 meta_len][meta JSON][PNG bytes]` |
| `rupa_format(ptr, len)` | same header, body is the canonical `rupa fmt` text (empty when `status` is `error`) |
| `rupa_metadata()` | same header, body is the canonical language metadata JSON (keyword classes, named colours, units, primitive properties) |
| `rupa_set_font(...)` | installs extra font bytes; the default Rupa Sans is embedded |

The host frees both the source buffer (`rupa_free`) and either result
(`rupa_free_frame`). `rupa_format` runs the same formatter as the CLI, so the
playground never reimplements formatting in JavaScript.

## Verifying

- `cargo test -p rupa-wasm` pins PNG output against `tests/golden/minimal.png`
  and checks determinism, diagnostics, frame layout, and the metadata export.
- `cargo test -p rupa-cli` compares `docs/language-reference.json` with
  `rupa metadata`, so keeping the reference in sync is a test failure rather
  than a manual review. Regenerate its canonical keys with
  `rupa metadata --reference docs/language-reference.json`; hand-authored prose
  such as `document_model` and `example` is preserved.
- The page itself was verified by loading it in Chromium: starter source renders
  (`320×240 in 38 ms`), a broken source shows `S005` at `2:1` while keeping the
  previous preview, an example loads and renders (`320×220 in 10 ms`), and the
  download button enables after a successful render.
