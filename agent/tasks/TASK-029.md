# TASK-029: Text shaping and glyph rasterization

**Milestone**: 4 — Resources and Text
**Dependencies**: TASK-028 (text primitive and font loading)
**Status**: Complete

## Goal

Rasterize `text` primitives using a font rasterizer, shaping glyphs for the
requested family, size, weight, and alignment, and composite them into the
rendered document.

## Scope

- Discover fonts by family name and load font bytes.
- Rasterize glyphs with `fontdue`.
- Apply size, weight, fill, alignment, transform, and opacity.
- Treat `at (x, y)` as a baseline origin.
- Report a clear error when no font can be resolved.

## Files

- `crates/rupa-renderer/Cargo.toml`
- `crates/rupa-renderer/src/vector.rs`
- `crates/rupa-renderer/src/text.rs`
- `crates/rupa-cli/src/commands.rs`

## Acceptance Criteria

- [ ] Text primitives render visible glyphs
- [ ] Font family resolution is deterministic
- [ ] Missing font produces a clear render error
- [ ] Transform and opacity are applied
- [ ] Existing tests and golden output remain unchanged
- [ ] `cargo fmt`, `cargo clippy`, `cargo build`, `cargo test` pass
