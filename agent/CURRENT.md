# Current Task

## Active Task
- **Task ID**: none
- **Status**: TASK-116 complete (rupa-syntax unit and integration tests); M19 closed

## Next Action
No scheduled task. Pick a product follow-up from `agent/BACKLOG.md` (publish the
playground, font upload control, or share-by-URL). TASK-108 through TASK-115 are
committed; TASK-116 is committed as `745bb44`.

## Progress
- [x] TASK-103: pack `FileId`/24-bit offset into 8-byte `Span`
- [x] TASK-104: final M18 verification and close-out
- [x] TASK-105: replace `ustr` with a lean pointer interner; verify output parity
- [x] TASK-106: simplify parser dispatch, diagnostics, and expression helpers
- [x] TASK-107: simplify lexer emission, diagnostics, and test assertions
- [x] TASK-108: simplify lowering state, shape construction, and helpers
- [x] TASK-109: simplify layer validation diagnostics and bag handling
- [x] TASK-110: simplify front-end bag merging and lowering control flow
- [x] TASK-111: simplify group, document, and paint validation
- [x] TASK-112: simplify shape validation diagnostics and helpers
- [x] TASK-113: simplify remaining validation, error, and metadata modules
- [x] TASK-114: encapsulate `DiagnosticBag` storage behind accessors and traits
- [x] TASK-115: use `stats_alloc` for the parse allocation baseline
- [x] TASK-116: simplify `rupa-syntax` unit and integration tests

## Notes for the next agent
- TASK-106 reduced `parser.rs` from 3,811 to 3,351 lines (-12.1%) with no output
  changes. Shared child dispatch is `parse_shared_child`; binary precedence uses
  `parse_binary`; diagnostics use `DiagnosticBag::error`/`error_with_help`.
- TASK-107 reduced `lexer.rs` from 1,074 to 852 lines (-20.7%). Token emission
  uses `push_token`/`emit`; tests use `lex_ok!`, `assert_lex_error!`, and
  `assert_tokens!`.
- TASK-108 reduced `lower.rs` from 1,783 to 1,598 lines (production 1,275 to
  1,090, -14.5%). `Lowerer` owns `graphics`/`bag`/`functions`/`rng`; shape
  primitives share `ShapeProps`, the `props!` macro, and `Lowerer::shape`.
- TASK-109 reduced `layer.rs` from 242 to 227 lines; validation uses
  `bag.error_with_help` and a shared `validate_plane(layer, &mut bag)` helper.
- TASK-110 reduced `compile.rs` from 179 to 168 lines; pipeline uses
  `DiagnosticBag::append` and `document.as_ref().and_then(..)`.
- TASK-111 reduced `group.rs`/`document.rs`/`paint.rs` by 103 lines total;
  validators use `error_with_help`/`append`, paint uses `map_err(bag_with)?`.
- TASK-112 reduced `shapes.rs` from 1,131 to 1,052 lines; `new_bag`,
  `validate_size_axes`, and `error_with_help` remove repeated boilerplate.
- TASK-113 reduced `validation.rs` 638 → 593 and `error.rs` 76 → 70; the
  compiler-crate simplification pass is complete.
- TASK-114 made `DiagnosticBag::diagnostics` private; use `iter`/`as_slice`/
  `len`/`is_empty`/`IntoIterator`, and `append`/`extend`/`collect` to mutate.
- TASK-115 replaced the custom allocation counter with `stats_alloc`; the parse
  baseline is now 222 allocations / 32_477 bytes (bounds 230 / 34_000).
- TASK-116 merged parser/lexer error tests into named tables and shared the
  integration-test helpers; test source dropped 150 lines with the same cases.
- `IStr` is now `&'static Entry` (8 bytes native, 4 bytes wasm32) interned in a
  global `OnceLock<Mutex<HashMap<&'static str, &'static Entry,
  BuildHasherDefault<DefaultHasher>>>>`; entries are leaked for process
  lifetime. `IStr::hash` uses the precomputed entry hash.
- `Borrow<str>` was removed; use `Deref`/`as_str` instead.
- Final sizes: `FileId` 1, `Position` 4, `Location` 4, `Span` 8,
  `Token` 32, `Document` 200, `DocumentChild` 304, `Text` 296,
  `Fill` 40, `Expression` 40, `GroupDefinition` 96, `LayerDefinition` 208.
- Fixture parse: 271 allocations / 50,387 bytes (warm).
- wasm release: 1,360,720 bytes.
- Release medians vs M18: `check minimal` 8.35 -> 2.09 ms, `render minimal`
  9.14 -> 3.73 ms, `layers` 12.51 -> 7.26 ms, `text` 9.54 -> 4.34 ms,
  `paints` 21.43 -> 16.38 ms, large-source `check` 730.77 -> 707.78 ms.
  Peak RSS (minimal render) 11,668 -> 4,480 KB.
- All PNGs, `inspect` JSON, and invalid-fixture `check --json` are byte-identical
  to the pre-simplification build; 557 workspace tests pass.
- Run cargo with `CARGO_INCREMENTAL=0` on this UNC workspace.
- Baseline worktrees/benchmark artifacts remain under ignored `target/`;
  cleanup requires user authorization.
