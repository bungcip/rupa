# Rupa Implementation Roadmap

This document breaks the Rupa specification into sequential implementation milestones. Each milestone builds on the previous one and must be independently verifiable.

---

## Milestone Overview

| Milestone | Title | Status | Depends On |
|-----------|-------|--------|------------|
| M1 | Minimal Vertical Slice | ✅ Complete | — |
| M2 | Core Vector Geometry | ✅ Complete | M1 |
| M3 | Layers and Composition | ✅ Complete | M2 |
| M4 | Resources and Text | ✅ Complete | M3 |
| M5 | Raster Drawing | ✅ Complete | M3 |
| M6 | Procedural Language Features | ✅ Complete | M4 |
| M7 | Advanced Static Graphics | ✅ Complete | M5, M6 |
| M8 | LLM Authoring Experience | ✅ Complete | M1–M7 |
| M9 | Document Model & Root Layer Refactor | ✅ Complete | M1–M8 |
| M10 | Groups, defaults, and CLI output | ✅ Complete | M1–M9 |
| M11 | Browser renderer core (WASM) | ✅ Complete | M10 |
| M12 | WASM playground (editor + viewer) | ✅ Complete | M11 |
| M13 | Inline primitive properties | ✅ Complete | M12 |
| M14 | Playground editing experience | ✅ Complete | M13 |
| M15 | Baseline and compact spans | ✅ Complete | M14 |
| M16 | Interned identifiers | ✅ Complete | M15 |
| M17 | Enum layout and verification | ✅ Complete | M16 |
| M18 | Packed source locations | ✅ Complete | M17 |
| M19 | Lean interner | ✅ Complete | M18 |

M1–M19 are complete and validated. M17 recorded a cold-CLI render-time miss
because `ustr` initialization added a fixed ~5.6 ms to one-shot runs
(DECISION-042). M18 packed source locations, and M19 replaced the interner:
`check minimal.rupa` fell from 8.35 ms to 2.09 ms and peak RSS from 11.7 MB to
4.5 MB, closing the miss (DECISION-044).

The detailed Milestone 1–8 sections below record the scope under which those
milestones were completed. M9 supersedes the old `document` wrapper and root
composition model; M10 adds groups, defaults, and CLI output. Use the
specification and M9/M10 sections for current language behavior.

---

## Milestone 1 — Minimal Vertical Slice
**Target**: Compile a small `.rupa` file and produce a valid PNG with expected dimensions and pixels.

### Scope
- Rust workspace setup (`Cargo.toml`, crate structure)
- Lexer with indentation-aware tokenization
- Parser producing AST for minimal grammar
- Document declaration (`document`, `size`, `background`, `export`)
- Rectangle primitive (`rectangle at`, `size`, `fill`)
- Circle primitive (`circle center at`, `radius`, `fill`)
- Solid color fills (named colors, hex strings)
- Basic CPU renderer (using `tiny-skia` or similar)
- PNG output (using `png` crate)
- CLI: `check` and `render` commands

### Grammar Subset
```
document "name"
    size <width> x <height>
    background <color>
    rectangle at (x, y)
        size <w> x <h>
        fill <color>
    circle center at (x, y)
        radius <r>
        fill <color>
export "file.png"
```

### Acceptance Criteria
- [ ] `cargo build --workspace` succeeds
- [ ] `cargo test --workspace` passes
- [ ] `cargo fmt -- --check` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Example `.rupa` file compiles and renders correct PNG
- [ ] CLI `check` validates syntax/semantics without rendering
- [ ] CLI `render` produces PNG matching expected pixels

### Estimated Tasks
- TASK-001: Workspace and crate structure
- TASK-002: Lexer with indentation tracking
- TASK-003: Parser for minimal grammar
- TASK-004: AST and basic diagnostics
- TASK-005: Document declaration and validation
- TASK-006: Rectangle and circle primitives
- TASK-007: Solid fill paint representation
- TASK-008: Basic renderer with tiny-skia
- TASK-009: PNG encoding
- TASK-010: CLI check and render commands
- TASK-011: Integration test with golden image

---

## Milestone 2 — Core Vector Geometry
**Target**: Full primitive set with fills, strokes, and transforms.

### Scope
- Ellipse, square, triangle, polygon primitives
- Line and polyline primitives
- Path primitive (start, line to, curve to, quad to, close)
- Fill and stroke properties
- Transform support (position, rotation, scale, opacity)
- Stroke properties (width, cap, join)

### Acceptance Criteria
- [ ] All primitives render correctly
- [ ] Fill + stroke combinations work
- [ ] Transforms compose correctly (nested, rotation/scale)
- [ ] Stroke cap/join styles render
- [ ] Golden image tests for each primitive

### Estimated Tasks
- TASK-012: Ellipse, square, triangle
- TASK-013: Polygon with points list
- TASK-014: Line and polyline
- TASK-015: Path geometry (cubic/quadratic Bézier)
- TASK-016: Fill and stroke IR separation
- TASK-017: Transform matrix composition
- TASK-018: Stroke style properties

---

## Milestone 3 — Layers and Composition
**Target**: Reusable layer definitions with independent coordinate systems and stable composition.

### Scope
- Layer definitions (`layer "name"`, `size`, children)
- Layer instances (`place "name" at`, transform, opacity, z-index)
- Local coordinate systems per layer
- Stable z-index ordering (source order tiebreaker)
- Nested placements (layers containing other layer instances)
- Cycle detection for recursive layer references
- Forward references (resolve independent of declaration order)
- Default clipping to layer bounds

### Acceptance Criteria
- [ ] Layer definitions parse and validate
- [ ] Multiple instances of same layer render independently
- [ ] Transforms on instances don't affect definition or other instances
- [ ] Z-index ordering stable and correct
- [ ] Nested compositions work
- [ ] Cycle detection rejects recursive references
- [ ] Content clipped to layer bounds

### Estimated Tasks
- TASK-019: Layer definition AST and validation
- TASK-020: Layer instance placement and transforms
- TASK-021: Local coordinate system handling
- TASK-022: Z-index stable sorting
- TASK-023: Nested layer composition
- TASK-024: Cycle detection
- TASK-025: Layer bounds clipping

---

## Milestone 4 — Resources and Text
**Target**: External images and text rendering with deterministic fonts.

### Scope
- External PNG resource loading (`picture`)
- Picture placement with fit modes (contain, cover, fill, none)
- Text primitive (`text`, `at`/`center at`, font, size, weight, fill, align)
- Font loading (bundled test font for determinism)
- Missing resource diagnostics
- Font fallback and shaping (using `rustybuzz` + `fontdue`/`swash`)

### Acceptance Criteria
- [ ] PNG images load and render with fit modes
- [ ] Text renders at correct position with baseline anchor
- [ ] Bundled font produces deterministic output
- [ ] Missing resources produce clear diagnostics
- [ ] Golden tests for text and images

### Estimated Tasks
- TASK-026: Image resource loading and decoding
- TASK-027: Picture placement and fit modes
- TASK-028: Text primitive and font loading
- TASK-029: Text shaping and glyph rasterization
- TASK-030: Resource diagnostics

---

## Milestone 5 — Raster Drawing
**Target**: Raster surfaces with pixel operations and vector/raster composition.

### Scope
- Raster layer mode (`mode raster`)
- Clear operation
- Pixel set operations
- Filled rectangle raster operations
- Basic brush strokes
- Vector/raster layer composition
- Raster operations execute in source order

### Acceptance Criteria
- [ ] Raster layers create independent pixel buffers
- [ ] Clear, pixel, rect, brush operations work
- [ ] Raster layers compose with vector layers correctly
- [ ] Source order execution within raster layer
- [ ] Golden tests for raster operations

### Estimated Tasks
- TASK-031: Raster surface IR and allocation
- TASK-032: Clear and pixel operations
- TASK-033: Raster rectangle fill
- TASK-034: Basic brush stroke
- TASK-035: Vector/raster composition

---

## Milestone 6 — Procedural Language Features
**Target**: Variables, loops, deterministic randomness for generative graphics.

### Scope
- Variable declarations (`let`)
- Arithmetic expressions (+, -, *, /, %, precedence)
- Comparison and boolean operators
- Repeat loops (`repeat N times`)
- Indexed loops (`repeat N times as i`)
- Deterministic PRNG (document-level seed)
- `random(min, max)` function
- Basic function definitions (after grammar specified)

### Acceptance Criteria
- [ ] Variables work in expressions
- [ ] Loops generate correct repetition
- [ ] Indexed loops expose loop variable
- [ ] Same seed produces identical output
- [ ] PRNG documented and tested
- [ ] Procedural examples render deterministically

### Estimated Tasks
- TASK-036: Variable declarations and scoping
- TASK-037: Expression parsing and evaluation
- TASK-038: Repeat loops (counted)
- TASK-039: Indexed loops
- TASK-040: Deterministic PRNG implementation
- TASK-041: Random function integration
- TASK-042: Function grammar and basics (deferred)

---

## Milestone 7 — Advanced Static Graphics
**Target**: Gradients, patterns, compositing, filters.

### Scope
- Linear gradients (from/to, angle, stops)
- Radial gradients
- Pattern fills (dots, grid, stripes, checkerboard)
- Clipping and masks (alpha, luminance, shape-based)
- Blend modes (normal, multiply, screen, overlay, etc.)
- Filters (blur, brightness, contrast, etc.)
- Improved color management (linear-light compositing)

### Acceptance Criteria
- [ ] Gradients render correctly
- [ ] Patterns tile correctly
- [ ] Clipping/masks work with correct coordinate spaces
- [ ] Blend modes match reference
- [ ] Filters apply to scoped targets
- [ ] Premultiplied alpha compositing
- [ ] Golden tests for all features

### Estimated Tasks
- TASK-043: Linear gradient paint
- TASK-044: Radial gradient paint
- TASK-045: Pattern fills
- TASK-046: Clipping and mask system
- TASK-047: Blend mode implementations
- TASK-048: Filter pipeline
- TASK-049: Color space and linear compositing

---

## Milestone 8 — LLM Authoring Experience
**Target**: Tooling for reliable LLM-generated code.

### Scope
- Canonical formatter (`rupa fmt`)
- Structured language reference (JSON)
- `inspect` command with JSON output
- Stable diagnostic codes with suggestions
- Executable documentation (examples compile in CI)
- LLM benchmark suite

### Acceptance Criteria
- [ ] `fmt` produces canonical output
- [ ] Language reference JSON generated/validated
- [ ] `inspect` shows document structure
- [ ] Diagnostics have stable codes and suggestions
- [ ] All examples in docs compile in CI
- [ ] Benchmark suite runs and measures metrics

### Estimated Tasks
- TASK-050: Canonical formatter
- TASK-051: Structured language reference
- TASK-052: Inspect command JSON output
- TASK-053: Diagnostic codes and suggestions
- TASK-054: CI for executable examples
- TASK-055: LLM benchmark suite

---

## Milestone 9 — Document Model & Root Layer Refactor
**Target**: Make one `.rupa` file exactly one document, move every drawing
primitive under an implicit root layer, and reduce redundant primitive code.

### Current State
- Specification updated in TASK-056.
- Decision records 026–028 added.
- Parser/AST file-as-document migration complete in TASK-057.
- Checked-in fixtures migrated early in TASK-057 to keep CI green.
- Implicit root layer implemented across compiler, IR, renderer, and inspect in
  TASK-058.
- Common primitive parsing unified in TASK-059.
- Shared shape validation/lowering appearance pipeline completed in TASK-060.
- Example/benchmark/reference fixture audit completed in TASK-061.
- Canonical formatter ordering and CLI tests completed in TASK-062.
- End-to-end migration verification completed in TASK-063; Milestone 9 is closed.

### Scope
- Remove the `document "Name"` wrapper; one file is one document
- Parse file-level settings (`size`, `background`, `seed`, `export`)
- Add migration diagnostics for old sources
- Compiler-created implicit root layer with document bounds
- Lower top-level primitives and placements into the root layer
- Reserve the layer name `root`
- Shared/table-driven primitive parsing
- Shared shape validation and lowering
- Migrate examples, benchmarks, structured reference, fixtures, and CLI tooling

### Acceptance Criteria
- [ ] One file is exactly one document with no wrapper syntax
- [ ] Old `document` sources produce a clear migration diagnostic
- [ ] Every drawing primitive is owned by a layer
- [ ] Root-layer rendering and ordering match equivalent pre-migration documents
- [ ] `layer "root"` is rejected
- [ ] Primitive parser/validator/lowering duplication is materially reduced
- [ ] Examples, benchmarks, and goldens pass with the new syntax
- [ ] `cargo fmt`, clippy, build, and workspace tests pass

### Estimated Tasks
- TASK-056: Document model and root layer specification ✅
- TASK-057: Migrate AST and parser to file-as-document ✅
- TASK-058: Implicit root layer in compiler, IR, and renderer ✅
- TASK-059: Unify primitive parsing ✅
- TASK-060: Unify shape validation and lowering ✅
- TASK-061: Migrate examples, benchmarks, reference JSON, and fixtures ✅
- TASK-062: Update CLI inspect/formatter output and docs ✅
- TASK-063: End-to-end migration verification ✅

---

## Milestone 10 — Groups, defaults, and CLI output
**Target**: Introduce reusable `group` templates distinct from single-use
`layer` planes, make document `size`/`background` optional with defaults, remove
`export` from source, and make output CLI-determined.

### Current State
- Specification updated in TASK-064–066.
- Decision records 029–032 added.
- Group declarations and group-only placement implemented in TASK-070.
- Layer size optional and layer background fill added in TASK-071.
- Final migration and end-to-end verification completed in TASK-072; Milestone 10
  is closed.

### Scope
- Optional document `size` (default 256x256) and `background` (DECISION-029)
- Remove `export` from the language; output path from the CLI (DECISION-031)
- `group` as a reusable template with automatic or explicit size
- `place` targets group definitions only; layers are single-use planes
  composited once at their `z-index` (DECISION-032)
- Layer `size` optional (canvas default); layer-level `fill` is a background
  shorthand; empty layers are valid planes
- Reserve the name `root` for the implicit root layer
- Update formatter, inspect, diagnostics, examples, benchmarks, and reference

### Acceptance Criteria
- [ ] Document `size`/`background` optional with documented defaults
- [ ] `export` rejected with a migration diagnostic
- [ ] `group` declarations parse, validate, and render
- [ ] `place` targets groups only; placing a layer is `S034`
- [ ] Recursive group references rejected (`S035`)
- [ ] Layer background `fill` shorthand works; duplicate is `S036`
- [ ] Examples, benchmarks, and goldens pass with the new syntax
- [ ] `cargo fmt`, clippy, build, and workspace tests pass

### Estimated Tasks
- TASK-064: Document defaults specification ✅
- TASK-065: Remove `export` from the language ✅
- TASK-066: Group construct specification ✅
- TASK-067: Parser/AST for groups and optional settings ✅
- TASK-068: Compiler name resolution for groups ✅
- TASK-069: IR and lowering for group instances ✅
- TASK-070: Group declarations and group-only placement ✅
- TASK-071: Optional layer size and layer background fill ✅
- TASK-072: Final migration and end-to-end verification ✅

---

---

## Milestone 11 — Browser renderer core (WASM)
**Target**: Run the existing compile-and-render pipeline inside a browser module
and prove its output equals the CLI's.

### Scope
- `rupa-wasm` crate (`cdylib` + `rlib`) with a safe API and a documented C ABI
- Frame wire format `[u32 total][u32 meta_len][meta JSON][PNG bytes]`
- Diagnostics JSON with stable codes and 1-based source positions
- wasm-safe font resolution: `fontdb` native-only, in-memory registry in browsers
- Renderer hardening: no panic/trap on pattern tiles or missing resources

### Acceptance Criteria
- [x] `render_source` returns PNG + metadata for valid sources
- [x] Invalid sources return coded diagnostics with line/column
- [x] Browser render of `examples/minimal.rupa` equals `tests/golden/minimal.png`
- [x] wasm32 release build succeeds and instantiates in Node
- [x] `cargo fmt`, clippy, build, and workspace tests pass

### Tasks
- TASK-073: `rupa-wasm` crate and C ABI ✅
- TASK-074: wasm-safe font resolution ✅
- TASK-075: renderer hardening and golden parity ✅

---

## Milestone 12 — WASM playground (editor + viewer)
**Target**: A static page where editing Rupa source updates a rendered preview.

### Scope
- Editor pane, canvas viewer, diagnostics panel, status/timing indicator
- Zero-dependency build (`build.sh`) and static server (`serve.mjs`)
- Debounced live rendering, examples dropdown, PNG download
- No backend, no accounts, no storage

### Acceptance Criteria
- [x] Editing source re-renders the preview within ~1 second
- [x] Compile errors show codes, messages, and positions without killing the last preview
- [x] Examples can be loaded; PNG can be downloaded
- [x] `build.sh` produces the module and `serve.mjs` serves it with `application/wasm`
- [x] Browser verification documented; validation commands pass

### Tasks
- TASK-076: playground page shell ✅
- TASK-077: build and serve scripts ✅
- TASK-078: live preview wiring ✅
- TASK-079: end-to-end verification and close-out ✅

---

## Milestone 13 — Inline primitive properties
**Target**: Let a drawing primitive carry its properties on its own line, so
compact sources read naturally and formatting can produce the canonical block
form.

### Scope
- Parser body state machine shared by all drawing primitives, `text`, `picture`, and `place`
- Indentation means "continues the previous primitive"; inline lists end at end of line
- Block-valued properties (`fill gradient`, `fill pattern`, `points`, `control`) stay block-only
- Formatter expands inline properties to one per indented line and preserves comments
- Specification, decision record, and language reference updated

### Acceptance Criteria
- [x] `rectangle at (10px, 40px) size 80px x 40px fill red` parses and renders
- [x] Inline properties work inside `layer`, `group`, and `repeat` bodies
- [x] Inline and indented sources render byte-identical PNGs
- [x] `rupa fmt` expands inline properties, is idempotent, and keeps comments
- [x] `cargo fmt`, clippy, build, and workspace tests pass

### Tasks
- TASK-080: specification and decision record ✅
- TASK-081: parser body state machine ✅
- TASK-082: formatter expansion and comment preservation ✅
- TASK-083: tests, example, verification, and close-out ✅

---

## Milestone 14 — Playground editing experience
**Target**: Make the playground feel like an editor: coloured source and a
Format button that produces the canonical layout.

### Scope
- Syntax highlighting over a transparent-textarea editor, without dependencies
- `rupa_format` in `rupa-wasm` (safe API + C ABI) so formatting is the CLI's
  formatter, not a JavaScript reimplementation
- Format button that rewrites the buffer, keeps diagnostics, and re-renders
- Documentation and browser verification

### Acceptance Criteria
- [x] Keywords, properties, numbers, strings, comments, colours, and booleans are coloured
- [x] Highlighting stays aligned with the gutter and scrolls with the textarea
- [x] `rupa_format` returns `rupa fmt` output and reports failures without partial text
- [x] Format rewrites the editor, preserves comments, and re-renders
- [x] A source that cannot be formatted leaves the buffer untouched and shows diagnostics

### Tasks
- TASK-084: editor syntax highlighting ✅
- TASK-085: `rupa_format` export and its C ABI entry ✅
- TASK-086: Format button, browser verification, and close-out ✅

---

## Milestone 15 — Baseline and compact spans
**Target**: Establish a measured memory baseline and shrink source locations,
the largest fixed cost in every AST node.

### Scope
- Struct-size and allocation guardrails in `rupa-syntax`
- Interning backend spike (`ustr` preferred, `Arc<str>` fallback)
- Compact `FileId`, `Position`, and `Span` while preserving diagnostic values

### Acceptance Criteria
- [x] Baseline sizes and allocation counts are recorded
- [x] `size_of::<Span>() <= 40`
- [x] Diagnostic line/column/offset values and JSON are unchanged
- [x] `cargo fmt`, clippy, build, and workspace tests pass

### Tasks
- TASK-097: baseline sizes, allocations, and interning spike ✅
- TASK-098: compact `Span`/`Position`/`FileId` ✅

---

## Milestone 16 — Interned identifiers
**Target**: Stop allocating a `String` per identifier and remove duplicated
text payloads.

### Scope
- `IStr` interned handle with transparent string serialization
- Interned lexer identifiers and AST name fields
- `Box<str>` for unique payloads (`text`, picture paths, string literals)
- Remove the duplicate `Text.content` field when confirmed redundant

### Acceptance Criteria
- [x] `IStr` is pointer-sized on native and smaller on wasm32
- [x] No AST identifier/name field uses `String`
- [x] Diagnostic/JSON values and golden PNGs are unchanged
- [x] `cargo fmt`, clippy, build, and workspace tests pass

### Tasks
- TASK-099: `IStr`, backend, and lexer token migration ✅
- TASK-100: AST field migration and duplicate text removal ✅

---

## Milestone 17 — Enum layout and verification
**Target**: Reduce enum width only where measurements justify boxing, then close
the memory milestone with evidence.

### Scope
- Measure `DocumentChild` and `Fill` variant sizes
- Box selected large variants without excessive allocation overhead
- Final size/allocation/render/wasm comparison
- Decision record, roadmap, status, and handoff close-out

### Acceptance Criteria
- [x] `size_of::<DocumentChild>()` falls at least 25% from baseline, or a
      measured stop decision is documented (50.5% reduction)
- [x] AST heap allocation for the repetitive fixture falls at least 30%
      (32.7% fewer bytes)
- [x] Render regression measured and documented: hot renderer path unchanged,
      wasm growth 0.91%; cold CLI target missed because of `ustr` init
      (see DECISION-042; closed by TASK-105/DECISION-044)
- [x] All goldens, diagnostics, JSON, and validation commands pass

### Tasks
- TASK-101: measured enum-layout optimization ✅
- TASK-102: final verification, benchmarks, and milestone close-out ✅

---

## Milestone 18 — Packed source locations
**Target**: Reduce source-location memory again by changing the language limits
and deriving line/column only when diagnostics need them.

### Scope
- `FileId(u8)` with a 256-file limit
- 16 MiB - 1 source limit with diagnostic `L010`
- Offset-only `Position`; file id + 24-bit offset packed into `Location`
- Eight-byte `Span`
- `LineIndex` for diagnostic line/column and formatter indentation
- Final verification and close-out

### Acceptance Criteria
- [x] `Span` is 8 bytes, `Position` 4 bytes, `FileId` 1 byte
- [x] Packed locations round-trip 256 files and offsets through 16,777,215
- [x] Oversized files report `L010` without scanning
- [x] Diagnostics, JSON, formatter comments, and golden PNGs are unchanged for
      valid files
- [x] TASK-104 re-runs the full verification and closes the milestone

### Tasks
- TASK-103: packed locations, source limits, and on-demand line/column ✅
- TASK-104: final verification and M18 close-out ✅

---

## Milestone 19 — Lean interner
**Target**: Remove `ustr`'s fixed cold-start cost from one-shot CLI runs while
keeping the `IStr` contract and every output byte-identical.

### Scope
- Measure the `ustr` first-intern cost and resident memory
- Evaluate patched `ustr`, `Arc<str>`, indexed handles, and a lean pointer
  interner
- Replace the backend behind `IStr`; keep 8-byte native / 4-byte wasm handles
- Precompute identifier hashes so compiler maps keep pointer-speed lookups
- Remove the `ustr` dependency and re-verify sizes, outputs, wasm, and tests

### Acceptance Criteria
- [x] First intern falls from millisecond to microsecond scale
- [x] `check minimal.rupa` drops from 8.35 ms to 2.09 ms (-75%)
- [x] All example renders improve; large-source compile does not regress
- [x] Peak RSS falls from 11.7 MB to 4.5 MB
- [x] Golden PNGs, `inspect` JSON, and invalid-fixture `check --json` are
      byte-identical
- [x] wasm release shrinks to 1,360,720 bytes and renders the native golden

### Tasks
- TASK-105: evaluate, implement, and verify the lean interner ✅

---

## Dependency Graph

```
M1 → M2 → M3 → M4 → M6
           ↘ M5 ↗
M3 → M7 ← M5, M6
M1–M7 → M8 → M9 → M10 → M11 → M12 → M13 → M14 → M15 → M16 → M17 → M18 → M19
```

## Notes

- Milestones are **sequential where dependencies require it**
- M4 and M5 can proceed in parallel after M3
- M6 depends on M4 (text uses expressions)
- M7 depends on M5 (raster filters) and M6 (procedural filters)
- M8 depends on all previous milestones
- M9 depends on M8 and is a coordinated source-model migration; implementation
  tasks must land in dependency order
- M10 depends on M9 and adds groups, defaults, and CLI output; implementation
  tasks must land in dependency order
- M15 establishes the memory baseline; M16 depends on the compact span layout;
  M17 depends on both and closes with measured evidence
- M18 depends on M17; TASK-103 must land before TASK-104 closes the milestone
- M19 depends on M18; TASK-105 replaces the interning backend and closes the
  cold-start miss recorded by M17
- Each milestone must have **passing tests, golden images, and working CLI** before next starts