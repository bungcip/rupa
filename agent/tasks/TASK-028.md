# TASK-028: Text primitive and font loading

**Milestone**: 4 — Resources and Text
**Dependencies**: TASK-027 (picture placement), TASK-017 (transforms)
**Status**: Complete

## Goal

Make `text "..."` a first-class scene node with font, size, weight, fill, and
alignment properties, and provide a deterministic font-loading store analogous
to `ResourceStore`.

## Syntax

```rupa
text "Hello Rupa"
    at (100px, 100px)
    font "Inter"
    size 48px
    weight bold
    fill black
    align center
    rotate 15deg
    scale 1.2
    opacity 0.8
```

Centered text:

```rupa
text "Welcome"
    center at (400px, 300px)
    font "Inter"
    size 64px
    weight bold
    fill "#7950F2"
    align center
```

## Semantics

- `at (x, y)` anchors the text at its baseline origin (specification §9).
- `center at (x, y)` anchors at the horizontal/vertical center.
- `font` is an optional family/resource name.
- `size` is a positive dimension (default 16px when omitted).
- `weight` is `normal` (default) or `bold`.
- `fill` defaults to black.
- `align` is `left` (default), `center`, or `right`.
- Transform and opacity follow the shared primitive conventions.

## Files

- `crates/rupa-syntax/src/ast.rs`, `parser.rs`
- `crates/rupa-compiler/src/shapes.rs`, `validation.rs`, `lower.rs`
- `crates/rupa-ir/src/document.rs`, `lib.rs`
- `crates/rupa-image/src/font.rs`, `lib.rs`
- `crates/rupa-renderer/src/vector.rs`
- `docs/decisions/DECISION-025.md`

## Acceptance Criteria

- [ ] `text "..."` parses with `at`/`center at`, `font`, `size`, `weight`,
      `fill`, `align`, and transform properties
- [ ] Missing position is reported as `S003`
- [ ] Empty text is reported as an error
- [ ] IR `TextNode` carries text, position, font, size, weight, align, fill,
      transform, and opacity
- [ ] `FontStore` resolves and caches font resources
- [ ] Existing tests and golden output remain unchanged
- [ ] `cargo fmt`, `cargo clippy`, `cargo build`, `cargo test` pass
