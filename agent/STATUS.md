# Project Status

*Last updated: M19 closed through TASK-105; TASK-106 through TASK-116 simplification complete*

## Completed Milestones

- **M1 — Minimal Vertical Slice** (TASK-001…TASK-011) ✅
- **M2 — Core Vector Geometry** (TASK-012…TASK-018) ✅
- **M3 — Layers and Composition** (TASK-019…TASK-025) ✅
- **M4 — Resources and Text** (TASK-026…TASK-030) ✅
- **M5 — Raster Drawing** (TASK-031…TASK-035) ✅
- **M6 — Procedural Language Features** (TASK-036…TASK-042) ✅
- **M7 — Advanced Static Graphics** (TASK-043…TASK-049) ✅
- **M8 — LLM Authoring Experience** (TASK-050…TASK-055) ✅
- **M9 — Document Model & Root Layer Refactor** (TASK-056…TASK-063) ✅
- Maintenance: TASK-064…TASK-066 ✅ (committed as 437d7ea)
- **M10 — Groups, defaults, and CLI output** (TASK-067…TASK-072) ✅
- **M11 — Browser renderer core (WASM)** (TASK-073…TASK-075) ✅
- **M12 — WASM playground (editor + viewer)** (TASK-076…TASK-079) ✅
- **M13 — Inline primitive properties** (TASK-080…TASK-083) ✅
- **M14 — Playground editing experience** (TASK-084…TASK-086) ✅
- **M15 — Baseline and compact spans** (TASK-097…TASK-098) ✅
- **M16 — Interned identifiers** (TASK-099…TASK-100) ✅
- **M17 — Enum layout and verification** (TASK-101…TASK-102) ✅
- **M18 — Packed source locations** (TASK-103…TASK-104) ✅
- **M19 — Lean interner** (TASK-105) ✅
- Maintenance: TASK-106 parser simplification ✅
- Maintenance: TASK-107 lexer simplification ✅
- Maintenance: TASK-108 lowering simplification ✅
- Maintenance: TASK-109 layer validation simplification ✅
- Maintenance: TASK-110 compile pipeline simplification ✅
- Maintenance: TASK-111 group/document/paint validation simplification ✅
- Maintenance: TASK-112 shape validation simplification ✅
- Maintenance: TASK-113 remaining compiler modules simplification ✅
- Maintenance: TASK-114 `DiagnosticBag` encapsulation ✅
- Maintenance: TASK-115 `stats_alloc` parse baseline ✅
- Maintenance: TASK-116 `rupa-syntax` test simplification ✅

## Current Milestone

Milestones 1–19 are complete. M19 replaced `ustr` with a lean pointer interner:
`check examples/minimal.rupa` fell from 8.35 ms to 2.09 ms (-75%), first intern
from 1.405 ms to 1.7 µs, peak RSS from 11.7 MB to 4.5 MB, and wasm from
1,377,325 to 1,360,720 bytes. All PNGs, `inspect` JSON, and invalid-fixture
`check --json` remain byte-identical. The earlier cold-start miss is closed;
see DECISION-044.

M18 packed source locations to 8-byte spans with a
1-byte `FileId` and offset-only `Position`, reducing `DocumentChild` from 720 to
304 bytes and fixture allocations from 101,203 to 50,387 bytes. TASK-104 verified
that all four release render medians are within 2.1% of the M17 build, all PNGs
are byte-identical, and `check --json`/`inspect` output matches the M17 build.
The only new diagnostic is `L010` for source files over 16 MiB - 1.
M15–M17 reduced AST memory and verified the result: `DocumentChild` 1456 -> 720 B,
index/name strings interned, unique payloads boxed, `Span` 40 B, and the fixture
parse 150,297 -> 101,203 bytes (-32.7%). M19 removed the remaining cold-start
miss by deleting `ustr`'s fixed global cache; output, diagnostics, and JSON are
unchanged. The M11–M14 work is committed (renderer `0031e2a`, syntax `2e46ed0`,
wasm `03b3a16`, playground `d7f3edc`, docs with this file).

## Known Blockers

- This workspace is mounted over UNC; incremental compilation fails with
  "could not create session directory lock file". Run cargo with
  `CARGO_INCREMENTAL=0`.
- The default font remains vendored inside the repository (Rupa Sans, OFL), so
  text rendering has no host-font dependency. Browser hosts can still install
  extra faces via `rupa_set_font` / `rupa_renderer::install_font`.

## Milestone 11 summary

- New `crates/rupa-wasm` (`cdylib` + `rlib`): safe `render_source`/
  `encode_frame` API plus a documented C ABI (`rupa_alloc`, `rupa_free`,
  `rupa_render`, `rupa_free_frame`, `rupa_set_font`).
- Wire format `[u32 total][u32 meta_len][meta JSON][PNG bytes]`; metadata
  carries status, canvas size, diagnostics (code, severity, message, 1-based
  line/column, help) and `renderError`.
- Font resolution moved to `rupa-renderer::fonts`; the original M11 design used
  native `fontdb` plus a browser registry. TASK-091 later replaced both with the
  embedded Rupa Sans default (DECISION-040).
- Renderer hardening: pattern tiles return `RenderError::PatternTile` instead of
  panicking; browser `picture` fails with `R001` instead of touching a
  filesystem.
- Native tests pin output to `tests/golden/minimal.png` and verify determinism,
  diagnostics, frame layout, and the ABI. The built module was also driven from
  Node against its real ABI.
- Recorded in `docs/decisions/DECISION-033-wasm-playground.md`.

## Test Status

`cargo fmt --all -- --check` clean, `cargo clippy --all-targets --all-features
-- -D warnings` clean, `cargo build --workspace` OK, `cargo test --workspace`
passes (514 tests, 0 failures) with `CARGO_INCREMENTAL=0`.
`cargo build -p rupa-wasm --target wasm32-unknown-unknown --release` produces
`rupa_wasm.wasm` (1,258,052 bytes). Re-verified at M12 close-out: fmt/clippy/
build clean, 514 tests passed, 0 failed (20 suites, `CARGO_INCREMENTAL=0`).
Re-verified at M13 close-out: fmt/clippy/build clean, 527 tests passed, 0 failed
(22 suites, `CARGO_INCREMENTAL=0`). Re-verified at TASK-085: clippy/build clean,
535 tests passed, 0 failed (23 suites), then `cargo fmt --all` + `--check` clean
and `cargo test -p rupa-wasm` green (18 tests). Re-verified at TASK-088:
fmt check and clippy clean, workspace build OK, 551 tests passed, 0 failed
(25 suites, `CARGO_INCREMENTAL=0`), and the wasm target built with the new
`rupa_metadata()` export. Re-verified at TASK-089: fmt check, clippy, and
workspace build clean; 558 tests passed, 0 failed (25 suites). Re-verified at
TASK-090: fmt check, clippy, and workspace build clean; 560 tests passed, 0
failed (25 suites). Re-verified at TASK-091: fmt check, clippy, and workspace
build clean; 565 tests passed, 0 failed (25 suites). The wasm release build is
now `1,393,856` bytes and a Node ABI run rendered `examples/text.rupa` byte-
identically to `tests/golden/text.png`. Re-verified at TASK-095: fmt check,
clippy, and workspace build clean; 562 tests passed, 0 failed (25 suites).
TASK-095 and TASK-096 were committed together as `1191c64` after TASK-096:
fmt check, clippy, and workspace build clean; 531 tests passed, 0 failed (25
suites). The wasm release build is `1,388,056` bytes. TASK-096 also removed the
unnecessary `size` declarations from `examples/invalid/*.rupa`.
Re-verified at TASK-097: fmt check and clippy clean, workspace build OK, 535
tests passed, 0 failed (27 suites across the workspace), and
`cargo build -p rupa-syntax --target wasm32-unknown-unknown` issued no warnings.
The new size baseline is recorded in DECISION-042: `Span` 56, `Position` 24,
`FileId` 8, `Token` 88, `DocumentChild` 1456, `Text` 1432; parsing the synthetic
fixture takes 381 allocations / 150,297 bytes.
Re-verified at TASK-098: fmt check and clippy clean, workspace build OK, 541
tests passed, 0 failed, and the wasm release build is 1,396,128 bytes. Compact
source locations are `FileId` 4 B, `Position` 16 B, `Span` 40 B; AST sizes fell
correspondingly (`DocumentChild` 1456 -> 1168, `Text` 1432 -> 1160). The fixture
parse is now 381 allocations / 123,097 bytes. All diagnostic/JSON values and
golden images are unchanged.
Re-verified at TASK-099: fmt check and clippy clean, workspace build OK, 545
tests passed, 0 failed, and the wasm release build is 1,391,884 bytes. `IStr` is
8 B native (4 B on wasm32), `TokenKind` fell from 32 to 24 B and `Token` from 72
to 64 B. After intern-table warm-up the fixture parse is 304 allocations /
114,327 bytes. Diagnostic spans, JSON, and goldens are unchanged.
Re-verified at TASK-100: fmt check and clippy clean, workspace build OK, 547
tests passed, 0 failed, and the wasm release build is 1,388,244 bytes. AST sizes
fell to `DocumentChild` 1056 B, `Text` 1016 B, `GroupDefinition` 224 B, and
`LayerDefinition` 704 B. The fixture parse is 278 allocations / 109,651 bytes.
`Text` now stores only `text` (no duplicate `content`), and all AST identifiers
use `IStr`. Diagnostic values, JSON, and goldens are unchanged.
Re-verified at TASK-101: fmt check and clippy clean, workspace build OK, 551
tests passed, 0 failed, and the wasm release build is 1,400,685 bytes. Boxed
`Fill.gradient`/`Fill.pattern` and `Expression::Position`; `DocumentChild` is
720 B, `Fill` 72 B, `Expression` 72 B, `LayerDefinition` 368 B. The fixture parse
is 271 allocations / 101,203 bytes. Alternating release render runs on
`examples/paints.rupa` showed no regression (median 20.97 -> 20.94 ms) with a
byte-identical PNG.
Re-verified at TASK-102 (final): fmt check and clippy clean, workspace build OK,
551 tests passed, 0 failed, and the wasm release build is 1,400,685 bytes
(+0.91% vs TASK-097). Final sizes: `Span` 40, `Token` 64, `Text` 680,
`DocumentChild` 720, `Fill` 72, `Expression` 72, `GroupDefinition` 224,
`LayerDefinition` 368. The fixture parse is 271 allocations / 101,203 bytes
(-32.7% bytes). All goldens/diagnostics/JSON are unchanged. Cold CLI render
timings regressed by a fixed ~5.6 ms (`minimal` 4.78 -> 11.27 ms, `layers`
8.98 -> 15.36 ms, `text` 5.27 -> 12.29 ms) because of `ustr` initialization;
the target miss and keep decision are recorded in DECISION-042.
Re-verified at TASK-104 (final M18): fmt, clippy, workspace build, 556 tests,
and the wasm release build pass. Final sizes: `Span` 8, `Token` 32, `Document`
200, `DocumentChild` 304, `Text` 296, `Fill` 40, `Expression` 40,
`GroupDefinition` 96, `LayerDefinition` 208. Fixture parse is 271 allocations /
50,387 bytes; wasm is 1,377,325 bytes. Release render medians are +2.1%, +1.3%,
+0.4%, and +0.3% versus `7bc951f`, with byte-identical PNGs. `check --json` for
every invalid fixture and `inspect` JSON for every valid example are identical
to the M17 build.
Re-verified at TASK-105 (final M19): fmt check, clippy, workspace build, and
557 tests pass, 0 failures. The wasm release build is 1,360,720 bytes and a Node
ABI run rendered `examples/minimal.rupa` byte-identically to the golden. `IStr`
is still 8 bytes native (4 on `wasm32`); fixture parse stays 271 allocations /
50,387 bytes after warm-up. Release medians: `check minimal` 8.35 -> 2.09 ms,
`render minimal` 9.14 -> 3.73 ms, `layers` 12.51 -> 7.26 ms, `text` 9.54 ->
4.34 ms, `paints` 21.43 -> 16.38 ms, large-source `check` 730.77 -> 707.78 ms.
Peak RSS for minimal render is 11,668 -> 4,480 KB. All PNGs, `inspect` JSON, and
invalid-fixture `check --json` are byte-identical to the M18 build.
TASK-106 then simplified `parser.rs` from 3,811 to 3,351 lines (-12.1%) by sharing
child dispatch, replacing 30 raw diagnostics with `DiagnosticBag` helpers, and
extracting expression/identifier/unit/transform helpers. All 557 tests pass,
gates are clean, and PNG/JSON outputs are byte-identical to `df99783`; release
timings showed no regression. Test helper macros then removed repeated
parse/assert/match boilerplate (test module 1,159 -> 987 lines).
TASK-107 simplified `lexer.rs` from 1,074 to 852 lines (-20.7%) with token
emission helpers, guarded operator dispatch, shared string recovery, one-pass
unit spans, and test macros. All 557 tests pass, gates are clean, outputs and
the wasm render match `82c9ccf`, and wasm is 1,350,685 bytes.

## Next Actions

1. Commit the TASK-107 lexer simplification only after explicit user
   authorization.
2. Remaining product follow-ups: deploy the playground as a static site, add a
   font upload control (`rupa_set_font`), and share-by-URL.

## Milestone 12 summary

- `playground/` is a static page: editor pane with gutter and diagnostics,
  checkerboard preview stage, examples dropdown, Render, and Download PNG.
- `playground/build.sh` builds `rupa-wasm` for `wasm32-unknown-unknown` and copies
  the module (1,258,052 bytes) next to the page; `playground/serve.mjs` serves it
  on `127.0.0.1:5173` with `application/wasm` and a traversal guard.
- `playground/app.js` debounces edits by 250 ms, guards against stale renders,
  frees both wasm buffers, keeps the last good preview on errors, and shows code
  + message + `line:column` + help for each diagnostic.
- Verified in Chromium: starter render `320×240 in 38 ms`, `S005` at `2:1` on a
  broken source with the previous preview retained, `shapes.rupa` example
  `320×220 in 10 ms`, download enabled after success, no console errors.
- Limits documented: `picture` → `R001` (no filesystem), `text` without an
  installed font → `R003` (no bundled font).
- `playground/README.md` documents build, serve, usage, and limits.

## Milestone 13 summary

- A drawing primitive may write its properties inline:
  `rectangle at (10px, 40px) size 80px x 40px fill red`. An indented block is
  now only needed to continue the same primitive or for a block-valued
  property (`fill gradient`, `fill pattern`, `points`, path `control`).
- The parser uses one body state machine (`BodyState { Inline, Block, Empty }`)
  for `rectangle`, `circle`, `ellipse`, `square`, `triangle`, `polygon`, `line`,
  `polyline`, `path`, `text`, `picture`, `pixel`, `brush`, and `place`.
  Inline lists end at end of line, so a sibling primitive is never swallowed;
  a primitive with no properties reports one `P001`.
- `rupa fmt` expands inline properties to the canonical one-per-line block and
  now preserves `//` comments (the lexer records them via
  `tokenize_with_comments`; `tokenize()` is unchanged for other callers).
  Formatting is idempotent, including commented and nested sources.
- Two parser bugs were fixed: a `Block` body treated its closing `Dedent` as a
  property, and a bodyless primitive could leave `pos` past `Eof` and spin
  (`check_eof` now also treats a missing token as end of file).
- Inline and indented spellings render byte-identical PNGs
  (`crates/rupa-wasm/tests/render.rs`).
- Recorded in `docs/decisions/DECISION-035-inline-properties.md`; specification §7.0 and
  `docs/language-reference.json` (0.3.0) describe the inline form.
- `playground/examples/inline-shapes.rupa` shows the compact syntax and is in
  the playground dropdown. It is deliberately not under `examples/`, where
  `checked_in_examples_are_fmt_stable` requires canonical expanded formatting.

## Milestone 14 summary

- The playground editor highlights Rupa source: a coloured `<pre>` layer sits
  behind a transparent-textarea editor, sharing one typography rule. Tokens are
  comments, strings, numbers/units and hex colours, keywords, properties, the
  ten named colours, and booleans, all driven by a single `WORD_CLASSES` map.
  Repaints run on edits, Tab-indent, and example loads; the layer follows scroll.
- `crates/rupa-wasm` exports the CLI formatter: `format_source_text` +
  `encode_format_frame` (safe API) and `rupa_format(src_ptr, src_len)` (C ABI),
  reusing the render frame header so one reader and `rupa_free_frame` serve both.
  Failures return diagnostics with empty text. `rupa_render` now shares
  `read_source`/`copy_out` with it. 8 tests in `crates/rupa-wasm/tests/format.rs`.
- The editor's **Format** button rewrites the buffer with canonical output
  (inline properties expanded one per line, comments preserved), keeps the caret
  clamped, refreshes gutter/highlight, and re-renders. A source that cannot be
  formatted is left untouched with status `Cannot format` and its diagnostics.
- Verified in the browser: inline source became the expanded block, `// shapes`
  survived, gutter grew to 7 lines, render `200×100 in 1 ms`, 13 highlight spans,
  no console errors; a tab-indented source kept its buffer and showed `L002` at
  `1:1` with help.
- `playground/README.md` documents highlighting, Format, and the ABI table;
  `agent/handoffs/HANDOFF-2026-11-13-M14.md` records the handoff.

## Post-M14 follow-ups

- **TASK-087** added structured fix-its: `Diagnostic.edits: Vec<SuggestedEdit>`
  (`span` + `replacement`) is populated where the fix is exact —
  `with_suggestion` (unknown keyword) and unknown named colour `T002` — and is
  skipped when empty, so diagnostics without a fix serialize as before. Codes,
  messages, and help text are untouched.
- `rupa check --json` prints `{path, status, diagnostics[]}` with code,
  severity, message, positions, help, and edits (1-based lines/columns plus byte
  offsets). Human output and the exit status are unchanged.
- Tests: `diagnostic.rs` (2), parser keyword test, `paint.rs` (2), and 3 CLI
  integration tests (`check --json` valid / `L006` / `T002`).
- Recorded in `docs/decisions/DECISION-036-diagnostic-edits.md`; specification
  §22.3 documents the JSON shape.

- **TASK-088** removed the duplicated keyword/colour/unit/primitive data. The
  lexer, diagnostic suggestions, CLI, reference, and playground highlighter now
  share `rupa-syntax::metadata` plus `rupa-compiler::metadata_json`.
  `rupa metadata` prints the canonical JSON (`--reference FILE` merges it into
  the human-readable reference), and the wasm export `rupa_metadata()` returns
  the same payload with the existing frame header. The playground builds its
  word map and unit regex at load time from that export; the reference's
  canonical keys are checked against the compiler metadata in
  `crates/rupa-cli/tests/language_metadata.rs`.
- Removed spellings (`document`, `export`) stay lexed for migration diagnostics
  but are excluded from tooling metadata. Recorded in
  `docs/decisions/DECISION-037-language-metadata.md`.

- **TASK-089** made the formatter usable outside its in-place rewrite.
  `rupa fmt <file> --check` writes nothing and exits 1 when the file is not
  canonical, so CI and pre-commit hooks can gate formatting. `rupa fmt --stdin`
  reads source from stdin and writes canonical source to stdout for editor
  filters; `--stdin --check` reports through the exit status only. Supplying a
  path with `--stdin` is rejected by clap, and in-place formatting is unchanged.
  Recorded in `docs/decisions/DECISION-038-fmt-check-stdin.md`.

- **TASK-090** improved human diagnostic context. `render_diagnostic` now quotes
  up to three source lines covered by the primary span, draws carets per line,
  and labels the first caret run with the diagnostic message. Same-line spans
  quote one line; long spans use character columns so UTF-8 source stays
  aligned. `rupa check --json`, edits, and wasm metadata are unchanged.
  Recorded in `docs/decisions/DECISION-039-diagnostic-span-labels.md`.

- **TASK-091** closed the last determinism hole for `text`. The renderer embeds
  the OFL-licensed Rupa Sans regular and bold faces, pins family and SHA-256
  digests in `EMBEDDED_FONTS`, and verifies them in unit tests. Native builds no
  longer read system fonts; an unknown explicit family reports `R003`. Browser
  builds use the embedded faces by default, so the playground renders text
  without uploads, while `rupa_set_font` can still add host-supplied faces.
  `examples/text.rupa` matches `tests/golden/text.png` byte-for-byte from wasm.
  Recorded in `docs/decisions/DECISION-040-vendored-font.md`.

- **TASK-092** cleaned Cargo/clippy warnings: every member manifest inherits
  `[workspace.lints]`, unused `anyhow`/`pretty_assertions` workspace entries and
  per-crate unused dependencies were removed, and the invalid
  `clippy.performance` key became `clippy.perf`. `cargo clippy --all-targets
  --all-features -- -D warnings` now exits 0 with zero warning lines; workspace
  tests remain 565 passed.

- **TASK-093** removed internal duplication. `Parser::parse_shape_body` now owns
  the primitive body loop, shared property dispatch, unknown-property diagnostic,
  and dedent handling for all shape-like primitives; the wasm frame encoders
  share `encode_payload`; shape validation dropped unused stroke parameters; and
  lowering reuses `required` for pixel, brush, picture, and placement checks.
  No grammar, diagnostic code, help text, or wire format changed. Workspace tests
  remain 566 passed.

- **TASK-094** removed cross-crate duplication. IR root accessors share
  `root_nodes`; geometry pivot transforms share `about_pivot`; `ResourceStore`
  and `FontStore` share a private `PathCache<T>`; renderer RGB filters share
  `for_each_rgb` and blend-mode mapping moved to `to_skia_blend_mode`; CLI
  commands share `compile_input` and `input_base_dir`. Public APIs, filter math,
  render order, and diagnostic output are unchanged. Workspace tests were 570
  passed at that point.

- **TASK-095** removed the pre-release compatibility layer for `document` and
  `export`. Those words are no longer keywords or tokens, the parser/formatter
  have no migration branches, `KeywordSpec.documented` is gone, and catalog-only
  codes `P004`, `P005`, `C001`, `C002`, `C004`, `C005`, `C006`, `C012`, `S015`,
  `S019`, and `S021` were deleted. Old sources now report `L006` like any
  unknown keyword. Workspace tests are 562 passed and the wasm target builds.

- **TASK-096** pruned redundant unit tests: per-primitive shape validity and
  missing-property cases are now table-driven, duplicate omitted-size and
  non-PNG-output tests were merged, and tautological metadata/parser assertions
  were removed. The seven `examples/invalid/*.rupa` fixtures no longer declare
  `size`, so they exercise the 256×256 default. Workspace tests are 531 passed.
  TASK-095 + TASK-096 are committed as `1191c64`.

## Milestones 15–18 summary

- **M15 — Baseline and compact spans** (TASK-097…TASK-098): complete. Size and
  allocation guardrails landed in TASK-097; TASK-098 shrank `Span` to 40 bytes.
- **M16 — Interned identifiers** (TASK-099…TASK-100): complete. `IStr` and
  `Box<str>` now cover tokens and AST fields; duplicate `Text.content` is gone.
- **M17 — Enum layout and verification** (TASK-101…TASK-102): TASK-101 boxed
  the measured large payloads; TASK-102 verifies size, allocations, render time,
  wasm size, diagnostics, JSON, and goldens, then closes the milestone.
- **M18 — Packed source locations** (TASK-103…TASK-104): TASK-103 packs file id
  and offset into 8-byte spans, adds 256-file / 16 MiB - 1 limits and on-demand
  line/column; TASK-104 verifies render time, JSON, wasm size, and goldens.
