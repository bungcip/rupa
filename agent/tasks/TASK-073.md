# TASK-073: `rupa-wasm` crate and C ABI

## Task Header

- **Task ID**: TASK-073
- **Title**: Create the `rupa-wasm` crate exposing a C ABI for the browser
- **Milestone**: M11 — Browser renderer core (WASM)
- **Priority**: Critical
- **Dependencies**: TASK-072
- **Status**: Complete

---

## Description

Add a `cdylib` + `rlib` crate that runs the existing pipeline
(`compile_source` → `render_document` → PNG encoding) inside a browser module
and returns a single length-prefixed frame. The safe Rust API is testable
natively; only `abi.rs` owns raw pointers.

Wire format: `[u32 total][u32 meta_len][meta JSON][PNG bytes]`.

Diagnostics carry the stable language code plus 1-based `line`/`column`,
`endLine`/`endColumn`, and `help`. Resource/font render failures map to
`R001`/`R002`/`R003`; other render failures go to `renderError`.

---

## Files Created/Modified

| File | Action | Notes |
|------|--------|-------|
| `Cargo.toml` | Modify | Add `crates/rupa-wasm` workspace member |
| `crates/rupa-wasm/Cargo.toml` | Create | `crate-type = ["cdylib", "rlib"]` |
| `crates/rupa-wasm/src/lib.rs` | Create | `render_source`, `encode_frame`, metadata JSON |
| `crates/rupa-wasm/src/abi.rs` | Create | Documented `unsafe` boundary |
| `crates/rupa-wasm/tests/render.rs` | Create | Native tests |
| `docs/decisions/DECISION-033-wasm-playground.md` | Create | Architecture decision |

---

## Acceptance Criteria

- [x] `render_source` returns PNG bytes plus JSON metadata for valid sources
- [x] Invalid sources return `status: "error"` with coded diagnostics
- [x] Frame header is `[u32 total][u32 meta_len]` and round-trips
- [x] Every exported symbol has `# Safety` documentation; `unsafe` is confined
      to `abi.rs`
- [x] `cargo build --workspace` passes
- [x] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [x] `cargo test -p rupa-wasm` passes
- [x] `cargo build -p rupa-wasm --target wasm32-unknown-unknown --release` passes
      (1,258,052 bytes) and Node can instantiate it
