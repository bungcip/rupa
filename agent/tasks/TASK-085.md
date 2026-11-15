# TASK-085: `rupa_format` in the WASM module

## Task Header

- **Task ID**: TASK-085
- **Title**: Expose the canonical formatter through `rupa-wasm` and its C ABI
- **Milestone**: M14 — Playground editing experience
- **Priority**: Critical
- **Dependencies**: TASK-084
- **Status**: Complete

---

## Description

The playground needs the same formatting as `rupa fmt` instead of a JavaScript
reimplementation. `crates/rupa-wasm` now exposes it in two layers:

- Safe API: `format_source_text(source: &str) -> FormatFrame { meta, text }` and
  `encode_format_frame(&FormatFrame) -> Vec<u8>`.
- C ABI: `rupa_format(src_ptr, src_len) -> *mut u8`.

`rupa_format` returns the **same header** as `rupa_render`
(`[u32 total][u32 meta_len][meta JSON][body]`), so the host parses both results
with one reader and frees both with `rupa_free_frame`. The body is the formatted
UTF-8 text, or empty when formatting failed.

### Metadata

| Field | Meaning |
|-------|---------|
| `status` | `"ok"` or `"error"` |
| `diagnostics` | same shape as render diagnostics (`code`, `severity`, `message`, `line`, `column`, `endLine`, `endColumn`, `help`), sorted |
| `error` | reserved; null for formatting (parse/lex problems arrive as diagnostics) |

### Behaviour

- Success returns exactly what `rupa_syntax::format_source` produces, i.e. what
  `rupa fmt` writes: inline properties expanded to one per indented line,
  comments preserved, idempotent.
- Lexical errors and removed `document`/`export` syntax return `status: "error"`
  with diagnostics and empty text, so the host keeps its buffer.
- Formatting is token-based, so a syntactically invalid but lexically valid
  source is still laid out (`size 10px 10px`); the render pass reports it.
  This matches `rupa fmt` and is covered by a test so it cannot drift.

### Refactor

`rupa_render` and `rupa_format` share two private helpers in `abi.rs`:
`read_source` (host bytes → `String`, rejecting a null pointer with a length) and
`copy_out` (bytes → fresh host allocation). `rupa_render` behaviour is unchanged
and its tests still pin the wire format.

---

## Files Created/Modified

| File | Action | Notes |
|------|--------|-------|
| `crates/rupa-wasm/src/lib.rs` | Modify | `FormatFrame`, `format_source_text`, `encode_format_frame` |
| `crates/rupa-wasm/src/abi.rs` | Modify | `rupa_format`, `read_source`, `copy_out`, module contract docs |
| `crates/rupa-wasm/tests/format.rs` | Create | 8 tests: expansion, idempotency, comments, lexical error, removed wrapper, invalid-but-formattable, wire layout, ABI parity |
| `crates/rupa-wasm/Cargo.toml` | Modify | `serde_json` as a dev-dependency for tests |
| `docs/decisions/DECISION-033-wasm-playground.md` | Modify | Record the new export |
| `playground/README.md` | Modify | Document `rupa_format` in the browser contract |
| `agent/tasks/TASK-085.md` | Create | This document |

---

## Verification

| Check | Result |
|-------|--------|
| `cargo test -p rupa-wasm` | 18 passed, 0 failed (3 unit, 8 format, 7 render) |
| `cargo build -p rupa-wasm --target wasm32-unknown-unknown --release` | OK, module 1,277,458 bytes |
| Node against the real module | exports `rupa_alloc`, `rupa_free`, `rupa_render`, `rupa_free_frame`, `rupa_format` all functions; formatting the inline rectangle returns `status: ok` and the expanded block text |
| Workspace validation | see `agent/STATUS.md` (fmt/clippy/build/test) |

---

## Acceptance Criteria

- [x] `rupa_format` exported through the C ABI and reachable from JavaScript
- [x] Output equals `rupa fmt` (same `rupa_syntax::format_source`)
- [x] Failures report diagnostics and never return partial text
- [x] Wire format documented and reused from `rupa_render`
- [x] Tests cover layout, failure, idempotency, and ABI parity
- [x] Native and `wasm32-unknown-unknown` builds succeed
