# TASK-068: Optional document size and background defaults

## Task Header

- **Task ID**: TASK-068
- **Title**: Optional document `size` (256x256 default) and `background`
  (transparent default) in parser, compiler, renderer
- **Milestone**: M10 — Groups, defaults, and CLI output
- **Priority**: High
- **Estimated Effort**: 2–3 hours
- **Dependencies**: TASK-067 (specification revision)

---

## Description

Make the document canvas fully defaulted, per specification §2.3 and §5
(DECISION-029, DECISION-030).

- `size` is optional. Omitting it renders a `256px x 256px` canvas.
- `background` is optional. Omitting it renders a transparent canvas
  (`None` background in the IR, alpha 0 in the PNG).
- An omitted `size` is **not** a diagnostic. Duplicate declarations, non-pixel
  units, and non-positive dimensions remain errors.
- The implicit root layer bounds always equal the effective canvas.

The parser already produced `Option` fields for both settings; this task removes
the semantic requirement and applies the defaults during lowering, so the
renderer keeps receiving a sized document and needs no change.

`export` removal is TASK-069 and stays out of scope here.

---

## Files to Create/Modify

| File | Action | Notes |
|------|--------|-------|
| `crates/rupa-ir/src/document.rs` | Modify | `DEFAULT_CANVAS_WIDTH` / `DEFAULT_CANVAS_HEIGHT` (256) |
| `crates/rupa-ir/src/lib.rs` | Modify | Re-export the defaults |
| `crates/rupa-compiler/src/document.rs` | Modify | Missing `size` no longer reports `C005` |
| `crates/rupa-compiler/src/lower.rs` | Modify | Apply canvas defaults; always sync root bounds |
| `crates/rupa-compiler/src/validation.rs` | Modify | Update doc comment and tests |
| `crates/rupa-syntax/src/diagnostic.rs` | Modify | `C005` marked legacy (code stays stable) |
| `crates/rupa-syntax/src/parser.rs` | Modify | Test that both settings parse as omitted |
| `crates/rupa-cli/tests/cli.rs` | Modify | End-to-end default-canvas render test |

---

## Acceptance Criteria

**All must pass to mark task complete:**

- [x] A document without `size` compiles and renders at exactly 256x256
- [x] A document without `background` renders with alpha 0 in PNG output
- [x] The implicit root layer bounds equal the effective (defaulted) canvas
- [x] Missing `size` produces no diagnostic; `C003`/`C007`/`C009` still fire
- [x] Renderer needed no change (it already receives concrete dimensions)
- [x] `cargo fmt -- --check` passes
- [x] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [x] `cargo build --workspace` passes
- [x] `cargo test --workspace` passes

---

## Implementation Steps

1. Add `DEFAULT_CANVAS_WIDTH`/`DEFAULT_CANVAS_HEIGHT` to `rupa-ir` and re-export.
2. Drop the missing-`size` branch from `DocumentValidator::validate_size`.
3. In `lower_document`, default the canvas when `size` is absent and always
   propagate the effective size to the root layer.
4. Mark `C005` as legacy and keep it a stable code.
5. Add parser, compiler, and CLI tests.

---

## Testing Requirements

- Unit tests: default canvas on lowering, missing background is `None`,
  validator accepts an omitted `size`, parser accepts omitted settings.
- Integration tests: `rupa render` of a source without `size`/`background`
  writes a 256x256 PNG whose untouched pixels are fully transparent.
- Golden images: not applicable (existing goldens keep explicit sizes).

---

## Specification References

- §2.3 Predictable defaults (defaults table)
- §5 Document model and output
- §23.1 Lexer and parser tests
- §23.3 Renderer tests

---

## Handoff Notes

**Completed Work**:
- `size` and `background` are optional end to end; defaults applied in lowering.
- `C005` retained as a legacy code but never emitted.

**Remaining Work**:
- TASK-069 … TASK-072 (export removal, groups, layer semantics, migration).

**Test Results**:
- `cargo fmt --check`, `cargo clippy --all-targets --all-features -D warnings`,
  `cargo build --workspace`, `cargo test --workspace` all pass (489 tests).

**Known Issues**:
- `export` is still required unless `--output` is supplied (TASK-069).

**Exact Next Action**:
- TASK-069: remove `export` from the language and default the output path to
  the input filename with `.png`.
