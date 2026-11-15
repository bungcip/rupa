# DECISION-037: Canonical language metadata for tooling

- **Status**: Accepted
- **Date**: 2026-11-14
- **Related**: TASK-088, TASK-084, TASK-051, DECISION-033

## Context

The language's keyword spellings and associated facts were duplicated in four
places: the Rust lexer, `docs/language-reference.json`, `WORD_CLASSES` in
`playground/app.js`, and `NAMED_COLORS` in the compiler. The playground had
already highlighted 14 colours that Rupa does not accept, and the manual
reference had needed separate updates. Every new keyword could therefore make
the editor colour a word incorrectly without any failing test.

## Decision

1. `rupa-syntax::metadata` is the only keyword table. Each entry stores the
   spelling, token variant, highlighter class (`keyword`, `property`, `literal`),
   and whether it is currently documented. A macro generates the lexer lookup,
   the full spelling list used by diagnostic suggestions, and the metadata
   slices, so those cannot disagree.
2. The same module owns `UNITS` and the `PRIMITIVES` property lists. Named
   colours remain in `rupa-compiler::paint::NAMED_COLORS` because their RGBA
   values live there; compiler metadata takes the names from that table.
3. `rupa-compiler::metadata_json()` assembles the canonical JSON:
   `keyword_classes`, flat `keywords`, `named_colors`, `units`, and
   `primitives`.
4. `rupa-wasm` exports `metadata_json()` and the C ABI
   `rupa_metadata() -> *mut u8`. The body is the metadata JSON using the same
   `[total][meta_len][meta][body]` frame header as `rupa_format`, so the host
   reuses its reader and `rupa_free_frame`.
5. The playground builds its highlighter map and its number/unit regex from
   `rupa_metadata()` at load time. There is no JavaScript fallback word list.
6. `rupa metadata` prints the JSON; `--reference FILE` merges canonical keys
   into an existing reference object so prose fields such as `document_model`
   and `example` survive regeneration. A CLI test asserts the checked-in
   reference already matches.

## Alternatives considered

- **Generate only the reference and keep the JS list**: still needs a build step
  and leaves the editor's runtime data separate from the compiler.
- **Embed the metadata as a generated JS module**: works without wasm but adds a
  generated artifact and a build step; the playground already loads the module.
- **Derive primitives/properties from parser source**: attractive long-term but
  much larger; the canonical table is an explicit, reviewable improvement now.

## Consequences

- New keywords must be added once, in `rupa-syntax::metadata`; the lexer,
  suggestions, playground, CLI, and reference follow.
- The wasm frame contract gains a third payload kind, documented in
  `crates/rupa-wasm/src/abi.rs` and `playground/README.md`.
- `docs/language-reference.json` keeps its human-authored fields, but its
  language-fact keys are owned by `rupa metadata` and verified in CI.
- Removed spellings remain lexed but are excluded from tooling metadata.
