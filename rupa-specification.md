# Rupa Language Specification
## LLM-Friendly Static Graphics Programming Language

**Status:** Design specification / implementation brief  
**Implementation language:** Rust  
**Source extension:** `.rupa`  
**Compiled representation:** `.rbc` (planned)  
**Initial output format:** PNG only

---

## 1. Project overview

Rupa is a compiled, general-purpose graphics programming language for creating **static raster and vector-based artwork** through a combination of declarative drawing and procedural programming.

Rupa is designed primarily for source files authored by Large Language Models (LLMs), while remaining readable and understandable to humans.

Its design goals are:

1. **LLM-friendly:** predictable syntax, few alternatives, explicit semantics, and easy error correction.
2. **Human-readable:** concise English keywords and a consistent visual structure.
3. **Machine-verifiable:** strict grammar, static validation, deterministic rendering, and useful diagnostics.
4. **Composable:** reusable layer definitions can be instantiated many times.
5. **Hybrid graphics:** vector geometry and raster operations can coexist in one document.
6. **Deterministic:** the same source, resources, and settings produce the same output.
7. **Incremental:** the language and renderer can be implemented in small, testable milestones.

Rupa is a graphics programming language, not an SVG dialect. SVG may be considered as an external interchange format in a future project, but **SVG import/export is not part of this specification**.

### 1.1 Explicit non-goals

Rupa targets static image generation. The following are out of scope:

- Animation and timelines
- Keyframes and easing
- Interactive event handlers
- Mouse, keyboard, or pointer events
- Real-time interactive rendering
- Physics simulation
- SVG export or SVG as the internal representation

### 1.2 Initial output scope

The first implementation exports **PNG only**. Do not implement SVG, PDF, JPEG, WebP, TIFF, GIF, or video exporters as part of the initial implementation.

---

## 2. Design philosophy: human-readable, LLM-friendly, machine-verifiable

The language should optimize for reliable generation, not for accepting every possible way of expressing the same idea.

### 2.1 Canonical syntax

Every operation should have one canonical spelling and structure.

For example, use `fill` for the paint inside a shape. Do not add equivalent aliases such as `color`, `paint`, or `background` to shape properties.

The document-level `background` property is a distinct concept and is allowed.

A `.rupa` file is exactly one document. There is no document declaration or
wrapper block; file-level declarations describe the canvas and its contents
(§3, §5).

### 2.2 Explicit semantics

Avoid syntax whose meaning depends on guessing.

- `rectangle at (x, y)` means the rectangle's top-left corner.
- `circle center at (x, y)` means the circle's center.
- `ellipse center at (x, y)` means the ellipse's center.
- `line from (x1, y1) to (x2, y2)` specifies both endpoints.
- Coordinates inside a group or layer are local to that scope.
- A group's local origin `(0px, 0px)` is always its authored origin; an
  automatic bounding box never re-anchors group content (§6.2).
- A placed group instance has its own transform, opacity, and z-index.
- A layer without `size` uses the document canvas; a document without `size`
  uses `256px x 256px`; a document without `background` is transparent.

### 2.3 Predictable defaults

Defaults must be documented and independent of platform or renderer implementation.

Recommended defaults:

| Property | Default |
|---|---|
| Position | `(0px, 0px)` only where the construct explicitly permits omission |
| Scale | `1.0` |
| Rotation | `0deg` |
| Opacity | `1.0` |
| Blend mode | `normal` |
| Fill | none unless explicitly defined by a primitive shorthand |
| Stroke | none |
| Canvas size (document `size` omitted) | `256px x 256px` |
| Background (document `background` omitted) | transparent (alpha `0`) |
| Layer size (`size` omitted) | the document canvas |
| Group size (`size` omitted) | the automatic bounding box of the group's children |
| Clipping | enabled at the effective bounds of every layer and of every group with an explicit `size`; disabled for automatic group bounds |
| Z-index (omitted) | `0`; equal values keep source declaration order |
| Random seed | explicit document seed; otherwise a documented fixed default |

Do not silently invent different defaults in different backends.

### 2.4 Strict but helpful compiler

Unknown keywords and invalid properties should produce errors. Diagnostics should include:

- Stable error code
- File, line, and column
- A short explanation
- A suggested correction when possible

The compiler should not silently ignore misspelled properties.

### 2.5 Canonical formatting

Provide a formatter (`rupa fmt`) that emits the canonical representation of valid source. This reduces syntax variation in LLM-generated files and makes diffs easier to review. The canonical form puts each property on its own indented line, so inline properties are expanded, and it preserves `//` comments.

### 2.6 Source limits and locations

Source locations are compact by design. The following limits are part of the
language contract:

- A compilation may reference at most **256 source files**, identified by an
  8-bit `FileId` (`0..=255`).
- A single source file may be at most **16,777,215 bytes** (`16 MiB - 1`). The
  limit includes the file length because a span's end offset may equal it. A
  larger file is rejected with diagnostic `L010` before it is scanned.
- A span stores a file id and a 24-bit byte offset. Line and column numbers are
  not stored in the AST. They are derived from the source when a diagnostic or
  JSON report is produced.
- Byte offsets are authoritative. Reported lines and columns are one-based;
  columns count Unicode scalar values, matching the lexer's source positions.

These limits keep `Span` at eight bytes without changing the human-readable or
JSON diagnostic fields for valid files.

---

## 3. Source syntax overview

Rupa uses English keywords and indentation-based blocks.

- Four spaces per indentation level.
- Tabs are not allowed.
- Braces are not required.
- Property colons are not required.
- Blank lines do not affect scope.
- Comments begin with `//`.
- Blocks are determined by indentation.
- A child block must be indented exactly one level deeper than its parent.
- The parser must report inconsistent indentation clearly.
- A drawing primitive may write its properties inline on the same line as the
  primitive. An indented block is only needed to continue a primitive's
  properties on following lines, or for a property whose value is itself a
  block (for example `fill gradient`, `fill pattern`, `points`).

A minimal Rupa file:

```rupa
size 800px x 600px
background white

circle center at (400px, 300px)
    radius 100px
    fill red

rectangle at (100px, 100px)
    size 200px x 120px
    fill blue
```

A `.rupa` file is exactly one document; there is no `document` declaration or
wrapper block. Top-level declarations are:

- Document settings: `size`, `background`, `seed`
- Reusable templates and composition: `group`, `layer`, `place`
- Procedural declarations: `let`, `repeat`, `fn`
- Drawing primitives, which are implicitly owned by the root layer (§6.1)

Document settings are order-independent and each may appear at most once.
`size` is optional and defaults to `256px x 256px`; `background` is optional
and defaults to transparent. The output path is not part of the language: it
is supplied on the CLI command line (§5, §21, DECISION-031). The canonical
formatter emits document settings first in the order `size`, `background`,
`seed`.

A `fill` written directly inside a `layer` block is that layer's background
shorthand (§6.4). `fill` is not valid at file top level: the document uses
`background` for its canvas color.

One compilation produces one PNG. Batch generation uses multiple files or
repeated CLI invocations; multiple documents in one file are not part of the
language.

---

## 4. Lexical elements and values

### 4.1 Identifiers and strings

- Identifiers use ASCII letters, digits, and underscores.
- Identifiers cannot begin with a digit.
- Strings use double quotes.
- Escape sequences should include at least `\"`, `\\`, `\n`, and `\t`.
- Group and layer names are strings and are case-sensitive.
- Group names and layer names live in separate namespaces, because `place`
  resolves against group definitions only. Names must still be unique within
  their own kind.

### 4.2 Numbers and units

Support integer and floating-point numeric literals.

Supported units in the initial language:

- `px` — pixels
- `pt` — points
- `%` — percentage, only in contexts where a reference dimension is defined
- `deg` — degrees

The document's `size` is expressed in pixels. For the first implementation, shape geometry and layer dimensions should use pixels. Percentages may be parsed and type-checked only after their reference semantics are implemented; do not interpret percentages inconsistently.

Examples:

```rupa
size 1200px x 800px
radius 48px
rotate 30deg
opacity 0.8
```

Unitless values are permitted only for properties explicitly defined as dimensionless, such as `opacity`, `scale`, and numeric loop expressions.

### 4.3 Colors

Support:

- Named basic colors: `white`, `black`, `red`, `green`, `blue`, `yellow`, `orange`, `purple`, `gray`, `transparent`
- Hex colors: `"#RRGGBB"` and `"#RRGGBBAA"`
- `rgb(r, g, b)` as a later-compatible syntax

Hex colors are strings in the initial grammar to avoid confusing `#` with comment syntax.

### 4.4 Expressions

The language needs expressions for coordinates, dimensions, arithmetic, variables, function calls, and procedural generation.

Initial expression support should include:

- Numeric and string literals
- Boolean literals
- Variable references
- Parentheses
- `+`, `-`, `*`, `/`, `%`
- Comparisons: `==`, `!=`, `<`, `<=`, `>`, `>=`
- Boolean operators: `and`, `or`, `not`
- Function calls
- Tuple-like coordinate pairs `(x, y)`

Use a conventional precedence table and document it. Do not add implicit string-to-number or number-to-string conversions.

---

## 5. Document model and output

A `.rupa` file is exactly one document. The document's canvas is declared at
file level, without a `document` wrapper and without an output declaration.
The output path is supplied on the command line (DECISION-031).

```rupa
size 1200px x 800px
background "#87CEEB"
seed 12345
```

Rules:

- There is no `document` declaration; the file itself is the document.
- The document name is not part of the language. Tooling may derive a display
  name from the input filename.
- `size` is optional. If present it must appear at most once and be positive.
  If omitted, the canvas is `256px x 256px` (DECISION-029).
- `background` and `seed` are optional and may each appear at most once.
- Document settings are order-independent; the canonical formatter emits
  `size`, `background`, then `seed`.
- The document coordinate origin is the top-left corner.
- X increases to the right.
- Y increases downward.
- `background` is optional; if omitted, the document is transparent
  (DECISION-030).
- `seed` supplies the document-level deterministic random seed (§15.5).
- `export` is not part of the language. The CLI `--output` option determines
  the PNG path; without it, `rupa render input.rupa` writes `input.png`
  (§21). Sources are output-agnostic.
- PNG dimensions equal the document dimensions unless a future explicit
  export-scale feature is added.

The compiler creates an implicit root layer with bounds equal to the document
canvas (§6.1). It is not written in source and is not a reusable definition.

### 5.1 No document or export wrapper

Rupa has no `document "Name"` wrapper and no `export "path"` setting. A `.rupa`
file is exactly one document, and the CLI determines the output path. Sources
that use those words are rejected like any other unknown keyword; there is no
migration syntax or special diagnostic.

---

## 6. Groups and layers: definitions and composition

Rupa has two composition concepts with distinct responsibilities:

- A **group** is a reusable visual template. It holds drawing primitives, has
  a local coordinate system, and is instantiated any number of times with
  `place`.
- A **layer** is a single-use composition plane. It holds drawing primitives,
  group instances, its own local coordinate system, and stacks against other
  layers with `z-index`, `opacity`, and blend mode.

**Do not conflate a group with a layer.** The semantic rules (§6.0) are
normative. This design supersedes the former place-a-layer semantics
(DECISION-032).

### 6.0 Semantic rules for groups and layers

| Feature | Group | Layer |
|---|---|---|
| Contains drawing primitives | Yes | Yes |
| Has its own size | Yes (optional; automatic bounding box by default) | Yes (optional; defaults to the document canvas) |
| Has a local coordinate system | Yes | Yes |
| Can be used multiple times | Yes | No |
| Can be placed with `place` | Yes | No |
| Can have `z-index` on the definition | No | Yes |
| Can have `opacity` on the definition | No | Yes |
| Can have a blend mode on the definition | No | Yes |
| Can contain child groups | Via `place` | Via `place` |
| May be empty | Yes | Yes |

Additional rules:

- A `group` block cannot contain another `group` block directly; child groups
  enter through `place`.
- A `layer` block cannot contain another `layer` block; layers are top-level
  declarations.
- Placing a layer name with `place` is a semantic error, as is placing an
  unknown name.

### 6.1 The implicit root layer

Every document has exactly one implicit root layer. It is created by the
compiler and does not appear as a `layer` declaration in source.

- Its local coordinate system is the document coordinate system.
- Its bounds equal the document canvas (defaulting to `256px x 256px` when
  `size` is omitted, DECISION-029).
- It owns top-level drawing primitives and top-level `place` declarations.
- It does not add clipping beyond the output canvas.
- It is not reusable: it cannot be referenced by `place`, and an explicit
  `layer "root"` or `group "root"` definition is invalid (reserved name,
  `S029`).
- Root children keep source order and existing `z-index` ordering rules.
- As a composition plane, the root layer is composited against explicit
  layers in source order, like a layer with the default z-index (§6.5).

Top-level drawing primitives are shorthand for children of this root layer:

```rupa
size 400px x 300px
background white

circle center at (200px, 150px)
    radius 80px
    fill red
```

is semantically equivalent to a primitive inside an implicit root layer with
bounds `400px x 300px`. Authors do not need to write the root layer.

### 6.2 Group definitions

A group is a reusable visual template. Its children use its local coordinate
system, whose origin `(0px, 0px)` is the group's reference point.

```rupa
group "Cloud"

    circle center at (40px, 50px)
        radius 30px
        fill white

    circle center at (80px, 35px)
        radius 35px
        fill white

    circle center at (120px, 50px)
        radius 30px
        fill white
```

Group size:

- Without `size`, the group's size is the **automatic bounding box** of its
  children computed in group-local coordinates. No clipping is applied beyond
  the content itself.
- With an explicit `size`, the declared size defines the local viewport and
  clips content to it, using the same clipping rule as layers (§6.6). The
  size must be positive.
- The group definition itself does not accept `z-index`, `opacity`, or blend
  mode. Those belong to instances or to layers.

Origin and bounds are distinct concepts, and the origin never depends on the
content:

- The group's local origin is always the authored `(0px, 0px)`. Children are
  drawn where they are written, even if they extend to negative coordinates.
- The automatic bounding box is a *measured extent*, expressed in local
  coordinates. It is used for reporting, inspection, and as the transform
  origin of an instance (§11). It never translates the content and never
  changes what `place ... at` means.
- `place "Cloud" at (x, y)` positions the group's local origin at `(x, y)`.
  It does not center or align the automatic bounding box.
- An empty group has an empty automatic bounding box; placing it draws
  nothing.
- An explicit `size` is a viewport anchored at the local origin: it covers
  `(0px, 0px)` to `(w, h)` and clips everything outside it, including content
  at negative coordinates.

### 6.3 Group instances

Use `place` to instantiate a group. The name must reference a group
definition.

```rupa
place "Cloud" at (100px, 100px)

place "Cloud" at (500px, 180px)
    scale 0.8

place "Cloud" at (850px, 120px)
    scale 1.2
    rotate 5deg
    opacity 0.9
    z-index 20
```

Each instance independently supports:

- Position
- Scale
- Rotation
- Opacity
- Z-index (scoped to the containing layer, §6.5)
- Blend mode (after compositing support is implemented)

The definition is shared; instance-specific transforms must not mutate the
definition or other instances. Place is valid at file top level (inside the
implicit root layer), inside a `layer`, and inside a `group`.

### 6.4 Layer definitions

A layer is a single-use composition plane. Each layer has its own local
coordinate system and may declare a size independent of the document canvas.

```rupa
layer "Foreground"
    size 1200px x 800px
    z-index 10

    rectangle at (0px, 600px)
        size 1200px x 200px
        fill "#228B22"
```

Rules:

- `size` is optional. If omitted, the layer's bounds equal the document
  canvas.
- `z-index` orders the layer against sibling composition planes (§6.5).
- `opacity` and blend mode are layer-definition properties; they apply to the
  composited plane as a whole.
- A layer may be empty; it is still a valid plane.
- A `fill` written directly inside a `layer` block is a **background fill**:
  shorthand for painting the layer's full bounds with that paint, before all
  other children. It is not a primitive and appears at most once per layer.

```rupa
layer "Background"
    z-index 0

    fill "#87CEEB"

    place "Cloud" at (100px, 100px)
    place "Cloud" at (500px, 180px)
        scale 0.8
    place "Cloud" at (850px, 120px)
        scale 1.2
```

### 6.5 Composition planes, z-index, and stable ordering

The implicit root layer and every explicit layer are composited planes over
the document canvas.

- Lower z-index values render behind higher values; higher values render in
  front.
- An omitted `z-index` is `0`.
- Instances and planes with equal z-index retain source declaration order
  (stable ordering).
- The implicit root plane participates in the same ordering as explicit
  layers. Among equal z-index values it counts as declared before any explicit
  layer, so top-level content sits behind a layer that uses the default
  z-index. A layer placed behind top-level content must use a negative
  z-index.
- Z-index is scoped to the same composition parent: the document for layers,
  the containing layer or group for group instances.
- Do not use floating-point z-index values; use integers.

### 6.6 Group and layer bounds, clipping

- A layer's local viewport equals its declared size (default: the document
  canvas). Layer content is clipped to the layer bounds.
- Recommended default: clipping is always enabled. If an explicit clipping
  property is implemented, use one canonical spelling, for example `clip true`.
  Avoid adding a configurable `clip` property until its behavior is fully
  implemented and tested.
- A group with an explicit `size` clips its content to that size, both in its
  own scope and when placed. The viewport starts at the group's local origin,
  so content at negative local coordinates is clipped.
- A group with automatic size does not clip; it draws all content within its
  measured bounding box, wherever that box falls.
- The automatic bounding box is measured from the rendered geometry of the
  children, including stroke extents where implemented. It is deterministic:
  the same children always produce the same box.
- Negative or zero dimensions are invalid.

### 6.7 Nested composition

A group may contain placements of other group definitions, and a layer may
contain placements of any group definitions:

```rupa
group "Tree"

    place "Canopy" at (0px, 100px)

    place "Canopy" at (60px, 40px)
        scale 0.8

layer "Forest"
    size 600px x 400px

    place "Tree" at (50px, 50px)
    place "Tree" at (250px, 80px)
        scale 1.4
```

Nested placements use the containing scope's coordinate system. Detect and
reject recursive group-reference cycles unless a future language feature
explicitly defines them.

### 6.8 Forward references

The compiler should resolve group and layer references independent of
declaration order, provided all definitions exist in the file. An unknown
name is a semantic error. Explicit definitions named `root` are rejected
because the root layer is implicit and reserved.

### 6.9 Complete example: one group, two layers

This example uses every rule above and is the reference composition example.

```rupa
group "Cloud"

    circle center at (40px, 50px)
        radius 30px
        fill white

    circle center at (80px, 35px)
        radius 35px
        fill white

    circle center at (120px, 50px)
        radius 30px
        fill white

layer "Background"
    size 1200px x 800px
    z-index 0

    fill "#87CEEB"

    place "Cloud" at (100px, 100px)

    place "Cloud" at (500px, 180px)
        scale 0.8

    place "Cloud" at (850px, 120px)
        scale 1.2

layer "Foreground"
    size 1200px x 800px
    z-index 10

    rectangle at (0px, 600px)
        size 1200px x 200px
        fill "#228B22"
```

With a document canvas of `1200px x 800px` and a transparent background
(written as `size 1200px x 800px` and omitted `background` at file top level),
this document shows the intended responsibilities:

- `"Cloud"` is defined once and placed three times, each instance with its own
  transform; the definition is never mutated.
- `"Background"` and `"Foreground"` are single-use planes, each with its own
  composition, its own size, and its own `z-index`.
- `"Background"` paints its sky with the layer background `fill` shorthand,
  which renders before its placements.
- `"Foreground"` renders above it because `10 > 0`.
- The document itself is not a layer, a group, or an instance: it only states
  the canvas and the background default.

## 7. Drawing primitives

Every drawing primitive is owned by a composition scope: a group or a layer.
Inside an explicit `group` declaration, the primitive belongs to that group.
Inside an explicit `layer` declaration, the primitive belongs to that layer.
At file top level, a drawing primitive is shorthand for a child of the
implicit root layer (§6.1).

Within one group or layer, drawing order follows source order unless an
explicit composition feature changes it. `z-index` orders placements relative
to other children of the same scope; it does not escape the scope hierarchy.

Primitive examples in this section show the primitive syntax itself. They may
appear at file top level (root layer shorthand), inside a `group` block, or
inside a `layer` block.

### 7.0 Inline properties

A primitive's properties may be written on the primitive's own line, in an
indented block, or in both: inline properties first, then an indented block
that continues the same primitive.

```rupa
rectangle at (10px, 40px) size 80px x 40px fill red
circle center at (150px, 50px) radius 30px fill blue

rectangle at (10px, 120px) size 80px x 40px
    fill "#7950F2"
    opacity 0.8
```

- An inline property list ends at the end of its line, so the next primitive is
  never swallowed.
- An indented block always continues the preceding primitive; dedenting ends it.
- Properties whose value is a block (`fill gradient`, `fill pattern`, `points`,
  and path `control` blocks) still require an indented block of their own.
- Structural declarations (`layer`, `group`, `repeat`, `fn`) keep requiring an
  indented body; inline properties apply to drawing primitives, `text`,
  `picture`, and `place`.

The canonical formatter expands inline properties to one property per indented
line (DECISION-035).

### 7.1 Basic shapes

Initial primitive set:

- `rectangle`
- `square`
- `circle`
- `ellipse`
- `triangle`
- `polygon`
- `line`
- `polyline`
- `path`

Examples:

```rupa
rectangle at (50px, 50px)
    size 200px x 120px
    fill blue

square at (300px, 50px)
    size 100px
    fill red

circle center at (550px, 110px)
    radius 60px
    fill green

ellipse center at (200px, 350px)
    size 180px x 100px
    fill orange

triangle at (400px, 300px)
    size 160px x 140px
    fill purple
```

The same shapes may be written compactly (§7.0):

```rupa
rectangle at (50px, 50px) size 200px x 120px fill blue
square at (300px, 50px) size 100px fill red
circle center at (550px, 110px) radius 60px fill green
```

Geometry conventions:

- Rectangle and square positions refer to top-left.
- Circle and ellipse positions refer to center.
- Triangle position refers to its bounding-box top-left.
- Polygon points are local coordinates.
- Line endpoints are explicit.
- Negative dimensions and negative radii are invalid.

### 7.2 Polygon

```rupa
polygon
    points
        (100px, 100px)
        (200px, 50px)
        (300px, 100px)
        (250px, 200px)
        (150px, 200px)
    fill purple
```

The initial implementation should require at least three points for a filled polygon.

### 7.3 Lines and polylines

```rupa
line from (50px, 50px) to (300px, 200px)
    stroke black
    width 4px

polyline
    points
        (50px, 300px)
        (150px, 200px)
        (250px, 300px)
        (350px, 200px)
    stroke blue
    width 5px
```

A line has no fill. A polyline is open unless explicitly closed through a path.

### 7.4 Paths and curves

```rupa
path
    start at (100px, 300px)
    line to (200px, 100px)
    curve to (400px, 100px)
        control (250px, 50px) and (350px, 50px)
    line to (500px, 300px)
    close
    fill purple
    stroke black
    width 3px
```

Initial path commands:

- `start at` — begin a subpath
- `line to` — straight segment
- `curve to` — cubic Bézier segment with two control points
- `quad to` — quadratic Bézier segment
- `close` — close the current subpath

Arc commands can be added after the core path implementation.

### 7.5 Fill and stroke

Geometry and appearance must be represented separately in the IR.

A shape may have:

- Fill only
- Stroke only
- Both fill and stroke
- Neither fill nor stroke (valid but invisible)

```rupa
rectangle at (100px, 100px)
    size 200px x 150px
    fill "#7950F2"
    stroke white
    width 4px
```

Use `fill none` to explicitly disable fill.

### 7.6 Stroke properties

Planned properties:

- `width`
- `cap`: `butt`, `round`, `square`
- `join`: `miter`, `round`, `bevel`
- `dash`
- `miter-limit`

Example:

```rupa
line from (100px, 100px) to (500px, 300px)
    stroke black
    width 8px
    cap round
    join round
    dash 12px, 6px
```

Implement core width, cap, and join first. Add dash patterns after basic stroke rendering is stable.

---

## 8. Paints

A paint describes how an area is colored. Geometry should refer to a paint abstraction rather than embedding renderer-specific paint data.

### 8.1 Solid fill

```rupa
circle center at (200px, 200px)
    radius 80px
    fill "#FF6B6B"
```

### 8.2 Linear gradient

```rupa
rectangle at (0px, 0px)
    size 800px x 600px
    fill gradient
        from orange
        to purple
        angle 135deg
```

### 8.3 Gradient stops

```rupa
rectangle at (0px, 0px)
    size 800px x 600px
    fill gradient
        stop 0% red
        stop 50% yellow
        stop 100% blue
        angle 90deg
```

### 8.4 Radial gradient

Radial gradients are planned, but may be implemented after linear gradients.

### 8.5 Pattern fills

Pattern fills are an advanced feature. Candidate built-in patterns include dots, grid, stripes, and checkerboard. The syntax and coordinate semantics must be specified before implementation.

---

## 9. Text

Text is a first-class scene primitive.

```rupa
text "Hello Rupa"
    at (100px, 100px)
    font "Rupa Sans"
    size 48px
    fill black
```

Centered text:

```rupa
text "Welcome"
    center at (400px, 300px)
    font "Rupa Sans"
    size 64px
    weight bold
    fill "#7950F2"
    align center
```

Planned text properties:

- Font family or font resource
- Font size
- Weight
- Alignment
- Line height
- Letter spacing
- Text decoration

The implementation must explicitly define whether `at` means baseline origin or top-left anchor. Do not leave this ambiguous. For the first implementation, choose one anchor convention and document it; a baseline origin is recommended for typographic consistency.

Font discovery, font fallback, shaping, and missing-font diagnostics must be deterministic. The implementation vendors **Rupa Sans** (regular and bold), a small OFL-licensed face, as the default. The family name is pinned to SHA-256 digests in the renderer, and native builds do not read system-installed fonts: an unrecognised family reports `R003` instead of silently rendering with a host-dependent face. Browser hosts may install additional faces explicitly through `rupa_set_font`; those are resources supplied by the host, not implicit system fallback.

---

## 10. External images

Use `picture` for external raster image resources. Do not introduce an `image` alias: `picture` is the canonical primitive spelling, and there is no document declaration to confuse it with.

```rupa
picture "photo.jpg"
    at (100px, 100px)
    size 400px x 300px
    fit cover
```

Fit modes:

- `contain`
- `cover`
- `fill`
- `none`

The first implementation should support PNG input. Additional input formats can be added later through image-decoding dependencies. Missing resources must produce a clear diagnostic.

---

## 11. Transformations and opacity

Primitive and placed-instance transforms must use consistent semantics.

Supported transform concepts:

- Translation / position
- Rotation
- Scale
- Opacity

Example:

```rupa
rectangle at (100px, 100px)
    size 200px x 100px
    fill blue
    rotate 30deg
    opacity 0.8
```

Define transform origin explicitly. Recommended default: the geometric center of the primitive's local bounding box. For placed group instances, use the center of the group's effective bounds in the group's local coordinates: the rectangle `(0px, 0px)` to `(w, h)` when `size` is declared, or the measured automatic bounding box otherwise. The origin choice affects rotation and scale only; it never changes where `at` places the group's local origin. For layer definitions, use the center of the layer's declared bounds (the document canvas when `size` is omitted).

Transforms should compose using a documented matrix order. Tests must cover nested transforms and rotation/scale composition.

Opacity is a normalized value in the range `[0.0, 1.0]`. Values outside this range are semantic errors rather than silently clamped.

---

## 12. Raster graphics

Rupa combines vector scene elements and raster surfaces.

A raster layer owns a raster target with its own dimensions and local coordinate system.

```rupa
layer "Pixel Art"
    size 320px x 240px
    mode raster

    clear black

    pixel at (10px, 10px)
        fill white
```

Planned raster operations:

- Clear surface
- Set pixel
- Fill rectangle
- Draw raster image
- Brush stroke
- Apply image filter

Brush example:

```rupa
layer "Paint"
    size 800px x 600px
    mode raster

    brush size 12px
        fill white
        stroke
            (20px, 20px)
            (50px, 40px)
            (80px, 60px)
```

Raster operations should execute in source order. The first implementation can support clear, pixel, and filled rectangle before brush rendering.

---

## 13. Compositing, clipping, and masks

Compositing combines layers and raster/vector content into the final image.

Planned blend modes:

- `normal`
- `multiply`
- `screen`
- `overlay`
- `darken`
- `lighten`
- `color-dodge`
- `color-burn`
- `soft-light`
- `hard-light`
- `difference`
- `exclusion`

Masks should eventually support:

- Alpha masks
- Luminance masks
- Shape-based masks

Clipping and masking must have clearly defined coordinate spaces and evaluation order. These are advanced features and should not block the first working PNG renderer.

Use premultiplied alpha internally. Prefer linear-light compositing where the renderer supports it, and document the color-space conversion used for PNG output.

---

## 14. Filters and image processing

Rupa should support static image processing operations.

Planned filters:

- Gaussian blur
- Brightness
- Contrast
- Saturation
- Grayscale
- Hue rotation
- Sharpen
- Color matrix
- Noise
- Pixelate
- Vignette
- Displacement map

Filters should apply to an explicitly scoped target, such as a raster layer or a composited group. Avoid ambiguous global filter commands.

Example syntax proposal:

```rupa
layer "Photo"
    size 400px x 300px

    picture "photo.png"
        at (0px, 0px)
        size 400px x 300px
        fit cover

    effect blur 5px
```

This syntax is provisional. Define effect scope and ordering before implementing filters.

---

## 15. Procedural graphics and language constructs

Procedural programming is a core differentiator from static vector document formats.

### 15.1 Variables

```rupa
let primary = "#7950F2"
let radius = 80px

circle center at (400px, 300px)
    radius radius
    fill primary
```

### 15.2 Repeat

```rupa
repeat 100 times
    circle center at (
        random(0px, 800px),
        random(0px, 600px)
    )
        radius random(2px, 8px)
        fill white
```

### 15.3 Indexed loops

```rupa
repeat 12 times as i
    circle center at (400px, 400px)
        radius 30px + i * 25px
        fill none
        stroke purple
        width 2px
```

### 15.4 Functions

Functions should be available for reusable calculations and drawing abstractions. Their exact grammar and type rules must be specified separately before implementation.

Provisional example:

```rupa
fn star_size(index)
    return 10px + index * 2px
```

### 15.5 Deterministic randomness

Random generation must be reproducible.

- Provide a document-level seed.
- Use a documented deterministic PRNG algorithm.
- The same seed and same program execution must produce the same random sequence.
- Do not depend on system time or platform entropy unless a future explicit nondeterministic mode is added.

Example:

```rupa
size 800px x 600px
seed 12345
background black

repeat 100 times
    circle center at (
        random(0px, 800px),
        random(0px, 600px)
    )
        radius random(2px, 8px)
        fill white
```

---

## 16. Coordinate systems and units

The document, the implicit root layer, and each explicit layer share the same
coordinate conventions but have independent local coordinate systems.

- Origin is top-left.
- X increases rightward.
- Y increases downward.
- The root layer's local coordinates are the document coordinates.
- Group instances and layer transforms map local coordinates into their parent
  coordinate space.
- Nested placements compose transforms through the hierarchy.

Initial geometry should use pixels. Support for physical units (`pt`, `mm`, `cm`, `in`) and percentages requires a fully specified conversion/reference model and may be deferred.

The renderer must use consistent subpixel geometry. Rasterization should not prematurely round all coordinates to integer pixels.

---

## 17. Graphics IR

The internal representation must be independent of SVG and independent of any particular rendering backend.

Suggested high-level model:

```rust
struct GraphicsDocument {
    version: u32,
    width: u32,
    height: u32,
    color_space: ColorSpace,
    resources: ResourceStore,
    layers: LayerStore,
    groups: GroupStore,
    /// Implicit layer whose bounds equal the document canvas.
    root: LayerId,
}

struct LayerDefinition {
    /// None means the document canvas size.
    size: Option<Size>,
    scene: SceneId,
    clip: ClipMode,
    /// Background fill shorthand for the layer's full bounds.
    background_fill: Option<PaintId>,
    z_index: Option<i32>,
    opacity: f32,
    blend_mode: BlendMode,
    /// True for the compiler-created root layer, which cannot be placed.
    implicit: bool,
}

struct GroupDefinition {
    /// None means the automatic bounding box of the children. Either way the
    /// local origin stays at the authored (0px, 0px); nothing re-anchors.
    size: Option<Size>,
    scene: SceneId,
}

struct GroupInstance {
    group: GroupId,
    transform: Transform,
    opacity: f32,
    z_index: Option<i32>,
    blend_mode: BlendMode,
}

enum SceneNode {
    Shape(ShapeNode),
    Text(TextNode),
    Picture(PictureNode),
    Group(GroupNode),
    RasterSurface(RasterSurfaceNode),
}
```

The root layer is always present. Top-level primitives and top-level group
placements lower into it; it is not stored as an author-defined reusable layer
and cannot be referenced by `place`. `place` resolves against group
definitions only; placing a layer name is a semantic error (DECISION-032).
Explicit layer definitions are single-use composition planes, composited
against the root layer by z-index with stable source-order ties. Use
arena-backed storage and compact typed IDs where appropriate.
Avoid deeply nested ownership structures that make cloning and reuse expensive.

Geometry and paint should be separated:

```rust
enum Geometry {
    Rectangle(Rect),
    Circle(Circle),
    Ellipse(Ellipse),
    Polygon(Polygon),
    Polyline(Polyline),
    Path(PathGeometry),
}

struct ShapeNode {
    geometry: Geometry,
    fill: Option<PaintId>,
    stroke: Option<StrokeStyle>,
    transform: Transform,
    opacity: f32,
}
```

The exact IR should evolve with implementation. Do not prematurely implement every planned feature in the first milestone.

### 17.1 Render graph

The render graph is an execution plan derived from the Graphics IR. It is not the same thing as the layer hierarchy.

It should eventually represent:

- Resource loading
- Vector rasterization
- Raster operations
- Filter passes
- Mask passes
- Layer compositing
- Final color conversion
- PNG encoding

---

## 18. Rendering architecture

Use a CPU-first renderer. Keep the backend abstraction sufficiently clean that a GPU backend could be added later, but do not make GPU rendering a requirement for the initial implementation.

Potential Rust ecosystem dependencies to evaluate:

- `tiny-skia` for CPU vector rasterization
- `kurbo` for geometry utilities
- `png` for PNG encoding
- `image` for raster image decoding and pixel buffers
- `rayon` for safe parallel work where useful
- `rustybuzz` for text shaping
- `fontdue` or `swash` for glyph rasterization
- an embedded, hash-pinned font for deterministic default text (no system discovery)

Evaluate dependencies before committing to them. Keep licenses compatible with the project's intended distribution.

### 18.1 Color and alpha

- Use a documented internal color representation.
- Prefer premultiplied alpha for compositing.
- Define the conversion to PNG's output color representation.
- Avoid platform-dependent color behavior.
- Add tests for transparent edges and overlapping semi-transparent shapes.

### 18.2 Determinism

For the same source, resources, renderer version, and settings:

- Geometry evaluation must be deterministic.
- PRNG output must be deterministic.
- Layer ordering must be deterministic.
- Compositing order must be deterministic.
- PNG dimensions and pixel values should be stable within the guarantees documented by the project.

If exact cross-platform pixel identity cannot initially be guaranteed, document the boundary and test determinism on supported target environments.

---

## 19. Compiler pipeline

Recommended pipeline:

```text
Source (.rupa)
    |
    v
Lexer
    |
    v
Parser
    |
    v
AST
    |
    v
Name Resolution
    |
    v
Semantic Analysis and Type Checking
    |
    v
HIR
    |
    v
Graphics IR Lowering
    |
    v
IR Validation / Optimization
    |
    v
Render Graph
    |
    v
CPU Renderer
    |
    v
PNG Encoder
```

Keep the stages independently testable. Parsing should not perform rendering. The renderer should not depend on source syntax.

---

## 20. Rust workspace proposal

Start with a small workspace. Split crates only when a boundary is useful and the implementation justifies it.

Suggested initial structure:

```text
rupa/
├── Cargo.toml
├── crates/
│   ├── rupa-syntax/       # lexer, parser, AST
│   ├── rupa-compiler/     # name resolution, semantic analysis, lowering
│   ├── rupa-ir/           # Graphics IR and validation
│   ├── rupa-geometry/     # geometry, paths, transforms
│   ├── rupa-renderer/     # CPU rendering and compositing
│   ├── rupa-image/        # image resources and pixel buffers
│   └── rupa-cli/          # check, fmt, render, inspect
├── examples/
├── tests/
└── docs/
```

Do not create a separate crate for every small subsystem before there is a real need. Keep public APIs narrow and avoid cyclic dependencies.

---

## 21. CLI

Initial CLI commands:

```text
rupa check <input.rupa>
rupa fmt <input.rupa> [--check]
rupa fmt --stdin [--check]
rupa render <input.rupa> [--output <output.png>]
rupa inspect <input.rupa>
```

Expected behavior:

- `check`: parse, resolve names, type-check, validate resources and semantics; do not render.
- `fmt`: format valid source canonically. Without flags it rewrites the file in
  place. `--check` writes nothing and exits with status 1 when the source is not
  canonical, so it can gate CI or a pre-commit hook. `--stdin` reads source from
  standard input and writes canonical source to standard output for editor
  filters; combining it with `--check` suppresses the output and only reports
  the exit status. Formatting diagnostics still go to stderr and leave the file
  untouched.
- `render`: compile and render a PNG. The output path comes from `--output`;
  without it, the output is the input filename with the `.png` extension in
  the current working directory (DECISION-031). There is no `export` setting
  in source. An explicit `--output` must name a `.png` file (the extension is
  matched case-insensitively); any other path is rejected with a CLI error
  before rendering, so PNG bytes are never written under a misleading
  extension.
- `inspect`: report document settings (including the effective canvas size
  and defaults applied), the implicit root layer, group definitions, layer
  definitions, placements, resource references, and composition order. The
  document has no source-level name; tooling may report the input filename.

The CLI must return non-zero exit codes on failure and print diagnostics to stderr.

---

## 22. LLM-oriented tooling and documentation

LLM usability is a first-class project requirement.

### 22.1 Machine-readable language reference

Provide a structured reference (for example JSON) describing:

- Keywords
- File-level structure and the implicit root layer
- Grammar productions
- Primitive names
- Allowed properties for each primitive
- Property value types and units
- Defaults
- Semantic constraints
- Feature availability by language version
- Short valid examples

The structured reference must be generated from or checked against the implementation where practical, so it does not drift from compiler behavior. The implementation exposes the canonical keyword classes, named colours, units, and primitive property tables as `rupa metadata` from the CLI and `rupa_metadata()` from the browser module; the playground highlighter and the checked-in reference are driven by and validated against that data.

### 22.2 Examples as executable fixtures

Every documented example should be compiled in CI. Keep examples small and focused, with expected diagnostics for intentionally invalid examples.

### 22.3 Diagnostics for iterative repair

Diagnostics should be stable and machine-readable where possible.

Example:

```text
error[R102]: Unknown property 'colour'
  --> poster.rupa:12:5
   |
12 |     colour red
   |     ^^^^^^ Unknown property 'colour'
   |
   = help: Use 'fill' to set the shape's fill paint.
```

Human output quotes the source covered by the primary span: up to three lines,
with carets under the span and the diagnostic message labelling the first caret
run. Spans on one line show that line; spans crossing lines show the first line
from the start column, middle lines in full, and the final line to the end
column.

When the fix is exact, a diagnostic also carries machine-applicable edits.
`rupa check --json` prints one object with `path`, `status` (`ok` or `error`),
and `diagnostics`; each diagnostic has `code`, `severity`, `message`, `line`,
`column`, `endLine`, `endColumn`, `help`, and `edits`. An edit is a span
(`startLine`, `startColumn`, `startOffset`, `endLine`, `endColumn`, `endOffset`,
byte offsets) plus the `replacement` text, so a tool can repair the source
without parsing the help prose. Keywords and named colours are the current
sources of edits; diagnostics whose fix is only advisory carry none.

### 22.4 Inspection output

`rupa inspect` should provide a concise structured representation of the compiled document, including:

- Canvas dimensions (including defaults applied) and document settings
- The implicit root layer and its children
- Group definitions and their effective bounds
- Layer definitions and their background fills
- Group placements
- Local bounds
- Transforms
- Z-index and stable order
- Resource dependencies

A JSON inspection mode is recommended for tool integrations.

### 22.5 No natural-language parser in the core language

Do not make free-form English sentences part of the Rupa grammar. LLMs should translate natural-language requests into the strict Rupa DSL. A strict grammar is easier to validate, format, compile, and repair.

---

## 23. Testing strategy

Testing is essential because Rupa is intended to be generated by LLMs.

### 23.1 Lexer and parser tests

- Valid file-level syntax fixtures
- Legacy wrapper words are rejected as unknown keywords
- Documents without `size` (default 256x256) and without `background`
  (transparent)
- Group definitions with and without explicit `size`
- Invalid indentation
- Unknown keywords
- Malformed units
- Unterminated strings
- Missing required properties
- Duplicate document settings
- Nested blocks
- Expression precedence
- Formatter output for file-level document settings

### 23.2 Semantic tests

- Unknown group reference
- Placing a layer name (semantic error)
- Unknown layer reference
- Duplicate group or layer definition
- Explicit `layer "root"` or `group "root"` reserved-name rejection
- Root-layer ownership of top-level primitives and placements
- Recursive group-reference cycle
- Group automatic bounding box and explicit-size clipping
- Group local origin independent of the automatic bounding box
- `fill` directly inside a layer as background shorthand; `fill` at file top
  level rejected
- Layer size defaulting to the document canvas
- Layer background fill shorthand rendering before other children
- Empty layer and empty group validity
- Invalid dimensions
- Invalid opacity
- Invalid z-index type
- Invalid property for primitive
- Missing external image
- Invalid path command sequence

### 23.3 Renderer tests

- Individual primitives
- Fill and stroke
- Transform composition
- Layer compositing with z-index and stable order
- Group placement, transforms, and instance opacity
- Implicit root layer rendering
- Stable z-index ordering within the root layer
- Clipping at layer bounds
- Alpha compositing
- Raster operations
- Deterministic random generation

### 23.4 Golden image tests

Use small fixtures and compare rendered PNGs against approved reference images. Define tolerances where exact pixel identity is not appropriate. Keep tests deterministic and avoid relying on system fonts for golden tests.

### 23.5 LLM benchmark

Create a separate benchmark suite with tasks such as:

1. Draw a basic shape.
2. Compose a poster from several layers.
3. Reuse one group definition multiple times.
4. Generate a procedural pattern.
5. Use paths, gradients, and text.
6. Modify an existing source file.
7. Repair intentionally invalid source.
8. Reproduce a reference image.

Measure:

- First-attempt compilation success
- Syntax error rate
- Semantic error rate
- Number of repair iterations
- Rendering success
- Visual similarity to the reference
- Token count and source length

Do not optimize only for short source. Prefer clarity and reliability over golfing.

---

## 24. Implementation milestones

Implement incrementally. Each milestone must compile, pass tests, and have a usable example.

### Milestone 1 — Minimal vertical slice

- Rust workspace
- Lexer and indentation-aware parser
- Document settings and dimensions (one file = one document)
- Rectangle and circle
- Solid fills
- Basic CPU renderer
- PNG output
- CLI `check` and `render`

Acceptance: compile a small `.rupa` file and produce a valid PNG with the expected dimensions and pixels.

### Milestone 2 — Core vector geometry

- Ellipse, square, triangle, polygon
- Line and polyline
- Path with line and cubic Bézier segments
- Fill and stroke basics
- Transform support

### Milestone 3 — Layers and composition

- Layer definitions with independent dimensions
- Local coordinate systems
- Position, scale, rotation, opacity
- Stable z-index ordering
- Cycle detection

(Historical: this milestone originally instantiated layers with `place`.
Milestone 10 moves placement to groups; layer placement is no longer part of
the language.)

### Milestone 4 — Resources and text

- External PNG resources
- Picture placement and fit modes
- Text primitive
- Font loading and deterministic test fonts
- Useful missing-resource diagnostics

### Milestone 5 — Raster drawing

- Raster surfaces
- Clear
- Pixel operations
- Filled rectangles
- Basic brush strokes
- Vector/raster composition

### Milestone 6 — Procedural language features

- Variables
- Arithmetic expressions
- Repeat loops
- Indexed loops
- Deterministic random generator
- Basic functions, after their grammar and type rules are specified

### Milestone 7 — Advanced static graphics

- Linear and radial gradients
- Pattern fills
- Clipping and masks
- Blend modes
- Filters and image processing
- Improved color management

### Milestone 8 — LLM authoring experience

- Canonical formatter
- Structured language reference
- `inspect` command and JSON output
- Stable diagnostic codes and suggestions
- Executable documentation
- LLM benchmark suite

### Milestone 9 — Document model and root layer refactor

- Remove the `document` wrapper; one file is one document
- Parse file-level document settings without indentation
- Introduce the implicit root layer for top-level primitives and placements
- Reserve the layer name `root`
- Unify primitive parsing, validation, and lowering to reduce duplication
- Migrate examples, benchmarks, fixtures, and tooling to the new model

### Milestone 10 — Groups, defaults, and CLI output (planned)

- Optional document `size` with the `256px x 256px` default (DECISION-029)
- Optional `background` with transparent default (DECISION-030, formalized)
- Remove `export` from the language; CLI-determined output with the `input.png`
  default (DECISION-031)
- Introduce `group` definitions and `place` for groups (DECISION-032)
- Layer semantics: single-use planes, optional size defaulting to the canvas,
  background fill shorthand, empty layers valid
- Reject placing layer names; migrate existing sources that place layers
- Group automatic bounding box and explicit-size clipping
- Migrate examples, fixtures, formatter, language-reference JSON, inspect, and
  goldens to the new model
- Fold clarifying decision content into the specification (DECISION-033)

Do not implement animation or interactive features in any milestone.

---

## 25. Definition of done for the initial project

The initial project is successful when:

1. A user or LLM can author a valid `.rupa` source file.
2. One `.rupa` file is exactly one document, with no `document` wrapper.
3. The compiler parses and validates it deterministically.
4. Top-level drawing primitives belong to the implicit root layer, and every
   primitive is owned by a group or a layer.
5. A layer can have dimensions independent of the document canvas; an omitted
   layer size defaults to the canvas.
6. A group definition can be placed multiple times with independent transforms
   and z-index; layers are single-use composition planes.
7. The renderer can produce a PNG file whose path is chosen by the CLI.
8. Errors have useful source locations and stable diagnostic codes.
9. Tests cover parsing, semantic validation, and rendered output.
10. Documented examples compile in CI.
11. No SVG exporter or animation/interactive subsystem is required.
12. Documents without `size` render at 256x256; documents without
    `background` render with a transparent background.

---

## 26. Instructions for the coding agent

Implement the project in Rust according to this specification.

Important instructions:

1. **Start with Milestone 1.** Do not attempt to implement the entire language at once.
2. Before coding, inspect the repository and report its existing structure and constraints.
3. If the repository already contains code, preserve useful existing work and adapt the plan rather than replacing it blindly.
4. Write a short implementation plan with concrete modules and tests.
5. Implement a working vertical slice that produces PNG output before adding advanced features.
6. Keep syntax, AST, HIR, Graphics IR, and renderer responsibilities separate.
7. Do not use SVG as the internal IR or implement SVG export.
8. Do not implement animation or interactive graphics.
9. Keep examples and tests executable.
10. Run formatting, tests, and lint checks available in the repository.
11. Record incomplete features explicitly; do not silently accept syntax that is not implemented.
12. Prefer small, reviewable changes over a large speculative architecture.

The first deliverable should be a minimal but real compiler and renderer that accepts a `.rupa` file and writes a PNG.
