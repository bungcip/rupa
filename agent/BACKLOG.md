# Rupa Implementation Backlog

Prioritized list of independently executable implementation tasks. Each task has a unique ID, dependencies, and acceptance criteria.

---

## Task Index

### Milestone 1 — Minimal Vertical Slice
- [x] **TASK-001**: Workspace and crate structure
- [x] **TASK-002**: Lexer with indentation tracking
- [x] **TASK-003**: Parser for minimal grammar
- [x] **TASK-004**: AST and basic diagnostics
- [x] **TASK-005**: Document declaration and validation
- [x] **TASK-006**: Rectangle and circle primitives
- [x] **TASK-007**: Solid fill paint representation
- [x] **TASK-008**: Basic renderer with tiny-skia
- [x] **TASK-009**: PNG encoding
- [x] **TASK-010**: CLI check and render commands
- [x] **TASK-011**: Integration test with golden image

### Milestone 2 — Core Vector Geometry
- [x] **TASK-012**: Ellipse, square, triangle primitives
- [x] **TASK-013**: Polygon with points list
- [x] **TASK-014**: Line and polyline primitives
- [x] **TASK-015**: Path geometry (cubic/quadratic Bézier)
- [x] **TASK-016**: Fill and stroke IR separation
- [x] **TASK-017**: Transform matrix composition
- [x] **TASK-018**: Stroke style properties (width, cap, join)

### Milestone 3 — Layers and Composition
- [x] **TASK-019**: Layer definition AST and validation
- [x] **TASK-020**: Layer instance placement and transforms
- [x] **TASK-021**: Local coordinate system handling
- [x] **TASK-022**: Z-index stable sorting
- [x] **TASK-023**: Nested layer composition
- [x] **TASK-024**: Cycle detection for layer references
- [x] **TASK-025**: Layer bounds clipping

### Milestone 4 — Resources and Text
- [x] **TASK-026**: Image resource loading and decoding
- [x] **TASK-027**: Picture placement and fit modes
- [x] **TASK-028**: Text primitive and font loading
- [x] **TASK-029**: Text shaping and glyph rasterization
- [x] **TASK-030**: Resource diagnostics

### Milestone 5 — Raster Drawing
- [x] **TASK-031**: Raster surface IR and allocation
- [x] **TASK-032**: Clear and pixel operations
- [x] **TASK-033**: Raster rectangle fill
- [x] **TASK-034**: Basic brush stroke
- [x] **TASK-035**: Vector/raster composition

### Milestone 6 — Procedural Language Features
- [x] **TASK-036**: Variable declarations and scoping
- [x] **TASK-037**: Expression parsing and evaluation
- [x] **TASK-038**: Repeat loops (counted)
- [x] **TASK-039**: Indexed loops
- [x] **TASK-040**: Deterministic PRNG implementation
- [x] **TASK-041**: Random function integration
- [x] **TASK-042**: Function grammar and basics

### Milestone 7 — Advanced Static Graphics
- [x] **TASK-043**: Linear gradient paint
- [x] **TASK-044**: Radial gradient paint
- [x] **TASK-045**: Pattern fills
- [x] **TASK-046**: Clipping and mask system
- [x] **TASK-047**: Blend mode implementations
- [x] **TASK-048**: Filter pipeline
- [x] **TASK-049**: Color space and linear compositing

### Milestone 8 — LLM Authoring Experience
- [x] **TASK-050**: Canonical formatter
- [x] **TASK-051**: Structured language reference (JSON)
- [x] **TASK-052**: Inspect command JSON output
- [x] **TASK-053**: Diagnostic codes and suggestions
- [x] **TASK-054**: CI for executable examples
- [x] **TASK-055**: LLM benchmark suite

### Milestone 9 — Document Model & Root Layer Refactor
- [x] **TASK-056**: Document model and root layer specification
- [x] **TASK-057**: Migrate AST and parser to file-as-document
- [x] **TASK-058**: Implicit root layer in compiler, IR, and renderer
- [x] **TASK-059**: Unify primitive parsing
- [x] **TASK-060**: Unify shape validation and lowering
- [x] **TASK-061**: Migrate examples, benchmarks, reference JSON, and fixtures
- [x] **TASK-062**: Update CLI inspect/formatter output and docs
- [x] **TASK-063**: End-to-end migration verification

### Maintenance
- [x] **TASK-064**: Reduce lowering duplication
- [x] **TASK-065**: Simplify compiler unit tests
- [x] **TASK-066**: Simplify DiagnosticBag error handling

### Milestone 10 — Groups, defaults, and CLI output (complete)
- [x] **TASK-067**: Specification revision: groups, defaults, CLI output (spec only, committed as 430e2fd)
- [x] **TASK-068**: Optional document size (256x256 default) and background (transparent) in parser, compiler, renderer
- [x] **TASK-069**: Remove `export` from the language; CLI-determined output with `input.png` default and migration diagnostic
- [x] **TASK-070**: Introduce group definitions and group placement (`place` targets groups only); reject placing layers — complete (`group`, `S030`–`S035`, plane-based layers)
- [x] **TASK-071**: Layer semantics update: optional size defaulting to canvas, background fill shorthand, empty layers, single-use planes — complete (`S036`, canvas-default planes)
- [x] **TASK-072**: Migrate examples, fixtures, formatter, language-reference JSON, inspect, and goldens to the group model; end-to-end verification

### Milestone 11 — Browser renderer core (WASM) (complete)
- [x] **TASK-073**: `rupa-wasm` crate with a safe API and a documented C ABI (frame wire format, diagnostics JSON)
- [x] **TASK-074**: wasm-safe font resolution (`fontdb` native-only, in-memory registry for browsers)
- [x] **TASK-075**: renderer hardening (pattern-tile panic removed, `picture` fails with `R001`) and golden parity tests

### Milestone 12 — WASM playground (editor + viewer)
- [x] **TASK-076**: `playground/index.html` + `style.css` — editor pane, canvas viewer, diagnostics panel
- [x] **TASK-077**: `playground/build.sh` and zero-dependency `playground/serve.mjs` (correct `application/wasm` MIME)
- [x] **TASK-078**: module loading, debounced live render, examples dropdown, PNG download
- [x] **TASK-079**: end-to-end browser verification, playground README, docs and handoff

### Milestone 13 — Inline primitive properties
- [x] **TASK-080**: specification, `DECISION-035`, and language-reference annotations for inline properties
- [x] **TASK-081**: shared body state machine so every drawing primitive accepts inline, block, or inline-then-block properties
- [x] **TASK-082**: formatter expands inline properties and preserves `//` comments (lexer records comments)
- [x] **TASK-083**: parser/formatter tests, `examples/inline-shapes.rupa`, byte-identical render parity, docs and handoff

### Milestone 14 — Playground editing experience
- [x] **TASK-084**: syntax highlighting for the editor (token layer, no dependencies)
- [x] **TASK-085**: `rupa_format` export in `rupa-wasm` and its C ABI entry
- [x] **TASK-086**: Format button in the playground and documentation/verification

### Post-M14 follow-ups
- [x] **TASK-087**: `edits` (span + replacement) on diagnostics with an exact fix, exposed via `rupa check --json`
- [x] **TASK-088**: canonical language metadata (`rupa metadata`, `rupa_metadata()` wasm export); playground highlighter and reference validated from it
- [x] **TASK-089**: `rupa fmt --check` (CI/pre-commit gate) and `--stdin` (editor filter)
- [x] **TASK-090**: `rupa check` source quote + carets + span label (up to three lines)
- [x] **TASK-091**: vendored, hash-pinned Rupa Sans default font; deterministic text on native and wasm
- [x] **TASK-092**: clean all cargo/clippy warnings (workspace lint inheritance, unused dependencies)
- [x] **TASK-093**: consolidate primitive-body parsing, wasm frame encoding, and lowering checks
- [x] **TASK-094**: reduce duplication in IR accessors, image caches, renderer filters, and CLI input handling
- [x] **TASK-095**: remove legacy `document`/`export` wrapper handling and catalog-only diagnostic codes
- [x] **TASK-096**: prune redundant unit tests and consolidate table-driven coverage
- [x] **TASK-106**: simplify parser dispatch, diagnostics, and expression helpers
- [x] **TASK-107**: simplify lexer emission, diagnostics, and test assertions
- [x] **TASK-108**: simplify lowering state, shape construction, and helpers
- [x] **TASK-109**: simplify layer validation diagnostics and bag handling
- [x] **TASK-110**: simplify front-end bag merging and lowering control flow
- [x] **TASK-111**: simplify group, document, and paint validation
- [x] **TASK-112**: simplify shape validation diagnostics and helpers
- [x] **TASK-113**: simplify remaining validation, error, and metadata modules
- [x] **TASK-114**: encapsulate `DiagnosticBag` storage behind accessors and traits
- [x] **TASK-115**: use `stats_alloc` for the parse allocation baseline
- [x] **TASK-116**: simplify `rupa-syntax` unit and integration tests
- [ ] Publish the playground as a static site
- [ ] Font upload control for `text` (`rupa_set_font` is already exported)
- [x] **TASK-105**: evaluate and replace the interner; lean pointer interner
      removes `ustr`'s ~5.6 ms cold CLI startup cost (M19)

### Milestone 15 — Baseline and compact spans
- [x] **TASK-097**: measure AST sizes and allocations; select the `IStr` backend
- [x] **TASK-098**: compact `Span`/`Position`/`FileId`

### Milestone 16 — Interned identifiers
- [x] **TASK-099**: add `IStr`, migrate lexer tokens, keep payloads uninterned
- [x] **TASK-100**: migrate AST string fields and remove duplicate `Text.content`

### Milestone 17 — Enum layout and verification
- [x] **TASK-101**: box large enum variants based on measurements
- [x] **TASK-102**: verify memory/perf/wasm, update docs and close the milestone

### Milestone 18 — Packed source locations
- [x] **TASK-103**: pack `FileId`/offset into 8-byte spans; derive line/column
- [x] **TASK-104**: verify memory/perf/wasm and close M18

### Milestone 19 — Lean interner
- [x] **TASK-105**: replace `ustr` with a lean pointer interner, verify output
      parity, and close the cold-start miss

Dependency notes: TASK-068 and TASK-069 are independent of each other.
TASK-070 depends on TASK-068/069 only for clean fixtures; TASK-071 depends on
TASK-070. TASK-072 depends on TASK-068 … TASK-071 and closes Milestone 10.

Milestone 11 depends on M10. TASK-074 depends on TASK-073; TASK-075 depends on
TASK-073 and TASK-074 and closes Milestone 11.

Milestone 12 depends on M11. TASK-076 and TASK-077 are independent; TASK-078
depends on both; TASK-079 depends on TASK-076 … TASK-078.

Milestone 15 starts after M14 and TASK-097/TASK-098 are sequential. Milestone 16
depends on M15; TASK-099 must land before TASK-100. Milestone 17 depends on M16;
TASK-101 must land before TASK-102. TASK-103 must land before TASK-104.
TASK-105 depends on TASK-104 and closes Milestone 19.

---

## Detailed Task Specifications (Milestone 1)

### TASK-001: Workspace and crate structure
- **Priority**: Critical
- **Dependencies**: None
- **Description**: Initialize Rust workspace with crate structure per specification §20
- **Files to Create**:
  - `Cargo.toml` (workspace root)
  - `crates/rupa-syntax/Cargo.toml`
  - `crates/rupa-compiler/Cargo.toml`
  - `crates/rupa-ir/Cargo.toml`
  - `crates/rupa-geometry/Cargo.toml`
  - `crates/rupa-renderer/Cargo.toml`
  - `crates/rupa-image/Cargo.toml`
  - `crates/rupa-cli/Cargo.toml`
  - Minimal `lib.rs` for each crate
- **Acceptance Criteria**:
  - [ ] `cargo build --workspace` succeeds
  - [ ] `cargo test --workspace` passes (empty test suites OK)
  - [ ] `cargo fmt -- --check` passes
  - [ ] `cargo clippy -- -D warnings` passes
  - [ ] No cyclic dependencies between crates

### TASK-002: Lexer with indentation tracking
- **Priority**: Critical
- **Dependencies**: TASK-001
- **Description**: Implement indentation-aware lexer for Rupa syntax (§3, §4)
- **Files to Create/Modify**:
  - `crates/rupa-syntax/src/lexer.rs`
  - `crates/rupa-syntax/src/token.rs`
  - `crates/rupa-syntax/src/error.rs`
- **Features**:
  - Track indentation levels (4 spaces = 1 level)
  - Reject tabs
  - Emit `Indent`, `Dedent` tokens
  - Keywords: `document`, `size`, `background`, `export`, `rectangle`, `circle`, `at`, `center`, `radius`, `fill`, `x`
  - Units: `px`, `deg`, `%`, `pt`
  - Colors: named, hex strings
  - Numbers: integer, float
  - Strings: double-quoted with escapes
  - Comments: `//`
  - Coordinate pairs: `(x, y)`
- **Acceptance Criteria**:
  - [ ] Valid minimal example tokenizes correctly
  - [ ] Invalid indentation produces clear error with location
  - [ ] Tabs rejected with error
  - [ ] Unknown keywords produce error
  - [ ] Unit tests for all token types
  - [ ] `cargo test -p rupa-syntax` passes

### TASK-003: Parser for minimal grammar
- **Priority**: Critical
- **Dependencies**: TASK-002
- **Description**: Build parser producing AST from lexer tokens
- **Files to Create/Modify**:
  - `crates/rupa-syntax/src/parser.rs`
  - `crates/rupa-syntax/src/ast.rs`
- **Grammar Subset**:
  - Document declaration with size, background, export
  - Rectangle: `rectangle at (x, y)` + `size` + `fill`
  - Circle: `circle center at (x, y)` + `radius` + `fill`
  - Expressions for coordinates, dimensions, colors
- **Acceptance Criteria**:
  - [x] Valid minimal example parses to AST
  - [x] Invalid syntax produces parse error with location
  - [x] Missing required properties detected (syntax-level)
  - [x] AST structure matches specification
  - [x] Parser tests for valid and invalid inputs (60 tests)
  - [x] `cargo test -p rupa-syntax` passes

### TASK-004: AST and basic diagnostics
- **Priority**: Critical
- **Dependencies**: TASK-003
- **Description**: Define AST node types and diagnostic reporting
- **Files to Create/Modify**:
  - `crates/rupa-syntax/src/ast.rs` (complete)
  - `crates/rupa-syntax/src/diagnostic.rs`
  - `crates/rupa-syntax/src/span.rs`
- **Features**:
  - Source span tracking (file, line, column)
  - Diagnostic codes (e.g., `R001`, `R002`)
  - Error, warning, help severity
  - Suggested corrections where applicable
- **Acceptance Criteria**:
  - [ ] All AST nodes have span information
  - [ ] Diagnostics include code, location, message, suggestion
  - [ ] Error codes are stable and documented
  - [ ] Unit tests for diagnostic formatting
  - [ ] `cargo test -p rupa-syntax` passes

### TASK-005: Document declaration and validation
- **Priority**: Critical
- **Dependencies**: TASK-004
- **Description**: Semantic validation of document-level constructs
- **Files to Create/Modify**:
  - `crates/rupa-compiler/src/document.rs`
  - `crates/rupa-compiler/src/validation.rs`
- **Validation Rules**:
  - Exactly one `document` declaration
  - `size` required, positive dimensions
  - `background` optional (transparent default)
  - `export` path validated (CLI may override)
  - Coordinate system: top-left origin, Y down
- **Acceptance Criteria**:
  - [ ] Valid document passes validation
  - [ ] Missing size → error with suggestion
  - [ ] Negative/zero dimensions → error
  - [ ] Multiple documents → error
  - [ ] Unit tests for all validation rules
  - [ ] `cargo test -p rupa-compiler` passes

### TASK-006: Rectangle and circle primitives
- **Priority**: Critical
- **Dependencies**: TASK-005
- **Description**: AST and validation for rectangle and circle shapes
- **Files to Create/Modify**:
  - `crates/rupa-compiler/src/shapes.rs`
  - `crates/rupa-ir/src/geometry.rs`
- **Rectangle**:
  - Position: top-left via `at (x, y)`
  - Size: `size <w> x <h>` required, positive
- **Circle**:
  - Position: center via `center at (x, y)`
  - Radius: `radius <r>` required, positive
- **Acceptance Criteria**:
  - [ ] Valid rectangles and circles pass validation
  - [ ] Missing position/size/radius → errors
  - [ ] Negative dimensions/radius → errors
  - [ ] Position semantics documented and tested
  - [ ] `cargo test -p rupa-compiler` passes

### TASK-007: Solid fill paint representation
- **Priority**: Critical
- **Dependencies**: TASK-006
- **Description**: Paint IR for solid colors (named, hex)
- **Files to Create/Modify**:
  - `crates/rupa-ir/src/paint.rs`
  - `crates/rupa-compiler/src/paint.rs`
- **Color Formats**:
  - Named: `white`, `black`, `red`, `green`, `blue`, `yellow`, `orange`, `purple`, `gray`, `transparent`
  - Hex: `"#RRGGBB"`, `"#RRGGBBAA"` (strings)
- **Acceptance Criteria**:
  - [ ] All named colors parse to correct RGBA
  - [ ] Hex strings parse correctly (with/without alpha)
  - [ ] Invalid hex → error with suggestion
  - [ ] Paint IR separates geometry from appearance
  - [ ] Unit tests for color parsing
  - [ ] `cargo test -p rupa-ir -p rupa-compiler` passes

### TASK-008: Basic renderer with tiny-skia
- **Priority**: Critical
- **Dependencies**: TASK-007
- **Description**: CPU renderer using `tiny-skia` for vector rasterization
- **Files to Create/Modify**:
  - `crates/rupa-renderer/src/lib.rs`
  - `crates/rupa-renderer/src/vector.rs`
  - `crates/rupa-geometry/src/transform.rs`
- **Features**:
  - Convert IR geometry to `tiny-skia` paths
  - Apply fills
  - Handle transforms (translate, rotate, scale)
  - Premultiplied alpha compositing
- **Acceptance Criteria**:
  - [ ] Rectangle renders at correct position/size
  - [ ] Circle renders at correct center/radius
  - [ ] Fills match expected colors
  - [ ] Transforms compose correctly
  - [ ] Unit tests for geometry conversion
  - [ ] `cargo test -p rupa-renderer` passes

### TASK-009: PNG encoding
- **Priority**: Critical
- **Dependencies**: TASK-008
- **Description**: Encode rendered surface to PNG file
- **Files to Create/Modify**:
  - `crates/rupa-renderer/src/png.rs`
  - `crates/rupa-image/src/encode.rs`
- **Features**:
  - Use `png` crate for encoding
  - Output dimensions = document dimensions
  - sRGB color space conversion
- **Acceptance Criteria**:
  - [ ] PNG file written successfully
  - [ ] Dimensions match document size
  - [ ] Pixel values match expected output
  - [ ] Transparent backgrounds work
  - [ ] Integration test with known output
  - [ ] `cargo test -p rupa-renderer -p rupa-image` passes

### TASK-010: CLI check and render commands
- **Priority**: Critical
- **Dependencies**: TASK-009
- **Description**: CLI entry points for `check` and `render`
- **Files to Create/Modify**:
  - `crates/rupa-cli/src/main.rs`
  - `crates/rupa-cli/src/commands.rs`
- **Commands**:
  - `rupa check <input.rupa>` — parse, validate, no render
  - `rupa render <input.rupa> --output <output.png>` — compile and render
  - Non-zero exit on failure
  - Diagnostics to stderr
- **Acceptance Criteria**:
  - [ ] `check` validates syntax and semantics
  - [ ] `render` produces PNG file
  - [ ] Invalid input → non-zero exit, error to stderr
  - [ ] Help text for both commands
  - [ ] Integration tests for CLI
  - [ ] `cargo test -p rupa-cli` passes

### TASK-011: Integration test with golden image
- **Priority**: Critical
- **Dependencies**: TASK-010
- **Description**: End-to-end test compiling example and comparing PNG
- **Files to Create/Modify**:
  - `tests/integration_test.rs`
  - `examples/minimal.rupa`
  - `tests/golden/minimal.png` (reference)
- **Example Source**:
  ```rupa
  document "Test"
      size 200px x 100px
      background white

      rectangle at (10px, 10px)
          size 80px x 40px
          fill red

      circle center at (150px, 50px)
          radius 30px
          fill blue

  export "test.png"
  ```
- **Acceptance Criteria**:
  - [ ] Example compiles without errors
  - [ ] Output PNG matches golden reference (exact or tolerated)
  - [ ] Test runs in `cargo test --workspace`
  - [ ] Golden image committed to repo
  - [ ] Deterministic output verified

---

## Detailed Task Specifications (Milestone 9)

### TASK-056: Document model and root layer specification
- **Priority**: Critical
- **Dependencies**: TASK-055
- **Status**: Complete
- **Description**: Update the specification and record decisions for one file =
  one document, the implicit root layer, and the redundancy-refactor strategy.
- **Files**:
  - `rupa-specification.md`
  - `docs/decisions/DECISION-026.md`
  - `docs/decisions/DECISION-027.md`
  - `docs/decisions/DECISION-028.md`
  - `agent/tasks/TASK-056.md` … `TASK-063.md`
- **Acceptance Criteria**:
  - [x] Spec defines one `.rupa` file as exactly one document
  - [x] Spec defines the implicit root layer and reserved `root` name
  - [x] Spec defines old-document migration diagnostics
  - [x] Decision records 026–028 exist
  - [x] Implementation tasks TASK-057…TASK-063 are defined

### TASK-057: Migrate AST and parser to file-as-document
- **Priority**: Critical
- **Dependencies**: TASK-056
- **Status**: Complete
- **Description**: Parse file-level document settings with no `document` wrapper,
  reject old syntax with a migration diagnostic, remove `Document.name`, and
  update the formatter.
- **Diagnostics**:
  - `P004` removed `document` declaration
  - `C009` duplicate document size
  - `C010` duplicate background
  - `C011` duplicate seed
  - `C012` duplicate export
- **Files**:
  - `crates/rupa-syntax/src/{parser,ast,token,diagnostic,formatter}.rs`
  - Parser/formatter tests
  - Test fixtures and source strings migrated early to keep CI green
- **Acceptance Criteria**:
  - [x] New file-level syntax parses in any document-setting order
  - [x] `document "Name"` produces `P004` with unindent help
  - [x] Duplicate settings produce their stable codes
  - [x] Formatter emits canonical wrapper-free output and is idempotent
  - [x] `cargo test -p rupa-syntax` passes
  - [x] Full workspace test suite passes

### TASK-058: Implicit root layer in compiler, IR, and renderer
- **Priority**: Critical
- **Dependencies**: TASK-057
- **Status**: Complete
- **Description**: Create the implicit root layer, lower top-level primitives and
  placements into it, reserve `layer "root"`, and preserve pixel output.
- **Files**:
  - `crates/rupa-ir/src/document.rs`
  - `crates/rupa-compiler/src/{lower,layer,validation}.rs`
  - `crates/rupa-renderer/src/vector.rs`
  - `crates/rupa-cli/src/commands.rs`
- **Acceptance Criteria**:
  - [x] Every document has exactly one root layer with canvas bounds
  - [x] Top-level primitives and placements render as root children
  - [x] Root z-index/order matches equivalent pre-migration documents
  - [x] Nested explicit layers and clipping are unchanged
  - [x] `layer "root"` is rejected with `S029`
  - [x] IR/compiler/renderer tests pass

### TASK-059: Unify primitive parsing
- **Priority**: High
- **Dependencies**: TASK-057
- **Status**: Complete
- **Description**: Replace repeated per-shape property loops with shared/table-
  driven helpers for position, size, fill, stroke, transform, opacity, and
  unexpected-property diagnostics.
- **Files**:
  - `crates/rupa-syntax/src/parser.rs`
- **Acceptance Criteria**:
  - [x] Common property parsing exists once
  - [x] Every primitive keeps required/optional behavior
  - [x] Diagnostic codes, spans, and suggestions do not regress
  - [x] `cargo test -p rupa-syntax` passes
  - [x] Table-driven tests cover all primitives

### TASK-060: Unify shape validation and lowering
- **Priority**: High
- **Dependencies**: TASK-058, TASK-059
- **Status**: Complete
- **Description**: Share common shape validation and lowering for paint, stroke,
  transform, opacity, and diagnostics while keeping geometry-specific rules
  explicit.
- **Files**:
  - `crates/rupa-compiler/src/{shapes,lower,validation}.rs`
- **Acceptance Criteria**:
  - [x] Common validation implemented once
  - [x] Common lowering implemented once
  - [x] IR output and golden images are unchanged
  - [x] Diagnostics retain stable codes and helpful messages
  - [x] Compiler/renderer tests pass

### TASK-061: Migrate examples, benchmarks, reference JSON, and fixtures
- **Priority**: Critical
- **Dependencies**: TASK-057, TASK-058
- **Status**: Complete
- **Description**: Final migration audit after TASK-057 already moved most
  fixtures. Confirmed all checked-in `.rupa` sources, test source strings,
  benchmark prompts, and the structured reference are consistent.
- **Files**:
  - `examples/`
  - `benchmarks/`
  - `docs/language-reference.json`
  - `crates/rupa-cli/tests/executable_examples.rs`
- **Acceptance Criteria**:
  - [x] No checked-in source uses `document "Name"`
  - [x] Executable-example tests pass
  - [x] Benchmark reference solutions score perfectly
  - [x] Golden images remain pixel-identical
  - [x] Language-reference example compiles

### TASK-062: Update CLI inspect/formatter output and docs
- **Priority**: High
- **Dependencies**: TASK-057, TASK-058
- **Status**: Complete
- **Description**: Ensure `fmt` emits wrapper-free canonical output and `inspect`
  reports the implicit root layer plus document settings.
- **Files**:
  - `crates/rupa-syntax/src/formatter.rs`
  - `crates/rupa-cli/src/commands.rs`
  - CLI tests/help text
- **Acceptance Criteria**:
  - [x] `rupa fmt` is canonical and idempotent for new syntax
  - [x] `rupa inspect` exposes the root layer
  - [x] CLI docs/help do not mention a document declaration
  - [x] `cargo test -p rupa-cli` passes

### TASK-063: End-to-end migration verification
- **Priority**: Critical
- **Dependencies**: TASK-057 … TASK-062
- **Status**: Complete
- **Description**: Audited for leftovers, ran all validation, verified examples/
  benchmarks/goldens/CLI, and closed Milestone 9.
- **Acceptance Criteria**:
  - [x] `cargo fmt -- --check` passes
  - [x] `cargo clippy --all-targets --all-features -- -D warnings` passes
  - [x] `cargo build --workspace` passes
  - [x] `cargo test --workspace` passes
  - [x] No checked-in `document "Name"` syntax remains
  - [x] Status/backlog/handoff are accurate

---

## Task Dependencies Summary

```
TASK-001
  └─ TASK-002
       └─ TASK-003
            ├─ TASK-004
            │    ├─ TASK-005
            │    │    └─ TASK-006
            │    │         └─ TASK-007
            │    │              └─ TASK-008
            │    │                   └─ TASK-009
            │    │                        └─ TASK-010
            │    │                             └─ TASK-011
```

---

## Notes

- Tasks must be completed in dependency order
- Each task updates `agent/CURRENT.md` when started
- Each task updates `agent/STATUS.md` and `agent/BACKLOG.md` when complete
- Handoff note required when stopping mid-task
- Validation commands must pass before marking complete