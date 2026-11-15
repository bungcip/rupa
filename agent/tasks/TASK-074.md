# TASK-074: wasm-safe font resolution

## Task Header

- **Task ID**: TASK-074
- **Title**: Move font resolution out of `vector.rs` and make it wasm-safe
- **Milestone**: M11 — Browser renderer core (WASM)
- **Priority**: Critical
- **Dependencies**: TASK-073
- **Status**: Complete (with a documented deviation on bundled fonts)

---

## Description

`resolve_font_bytes` used `fontdb::Database::load_system_fonts()` and
`std::fs::read`. `fontdb`'s default features (`fs`, `memmap`) do not build for
`wasm32-unknown-unknown`, and browsers have no font database.

Changes:

- New `crates/rupa-renderer/src/fonts.rs` owns resolution.
- `fontdb` moved to `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]`,
  so the browser build does not compile `fontdb`, `memmap2`, or
  `fontconfig-parser` at all.
- Browser builds resolve from an in-memory registry:
  `rupa_renderer::install_font(family, bold, bytes)`, exposed to JS as
  `rupa_set_font`.
- Lookup preference: requested family at requested weight → requested family →
  default face at requested weight → default face → first face.
- Missing font reports `RenderError::MissingFont`, which `rupa-wasm` surfaces as
  `R003`, never a panic.

**Deviation:** the plan called for vendoring an OFL font (Regular + Bold) into
the crate. This environment has no outbound HTTP from the shell, so no font
binary could be downloaded and committing one is not possible. The registry
approach keeps the language's `text` support working once the host installs
font bytes, without shipping an unlicensed or unavailable asset.

---

## Files Created/Modified

| File | Action | Notes |
|------|--------|-------|
| `crates/rupa-renderer/src/fonts.rs` | Create | Native + browser resolution, tests |
| `crates/rupa-renderer/src/lib.rs` | Modify | Export `fonts` module |
| `crates/rupa-renderer/src/vector.rs` | Modify | Use `crate::fonts::resolve_font_bytes` |
| `crates/rupa-renderer/Cargo.toml` | Modify | `fontdb` only for non-wasm targets |
| `crates/rupa-wasm/src/abi.rs` | Modify | `rupa_set_font` export |

---

## Acceptance Criteria

- [x] `resolve_font_bytes` lives in `fonts.rs` with one implementation per target
- [x] Browser build compiles without `fontdb` (no fs/memmap dependencies)
- [x] Installing a default face makes it available to text rendering
- [x] Text without an installed font yields `R003`, not a panic
- [x] Native resolution behaviour and CLI output are unchanged
- [x] `cargo test -p rupa-renderer`, `cargo test --workspace` pass
