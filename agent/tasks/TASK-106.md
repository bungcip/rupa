# TASK-106: Simplify the parser implementation

- **Milestone**: Maintenance (after M19)
- **Priority**: Medium
- **Status**: Complete
- **Dependencies**: TASK-105
- **Owner**: implementation-agent

## Goal

Reduce duplication and incidental complexity in `crates/rupa-syntax/src/parser.rs`
without changing the language, diagnostics, or rendered output.

## Scope

- Replace repetitive raw `Diagnostic` literals with `DiagnosticBag` helpers.
- Share child dispatch across document, layer, group, and repeat bodies.
- Extract identifier, call, unit, binary-expression, position-component, and
  transform-property helpers.
- Remove one-line wrappers with a single caller.
- Verify parser tests, full workspace gates, and output parity against the
  previous commit.

## Non-Goals

- No grammar, AST, diagnostic code/message, span, JSON, or image changes.
- No new dependencies.
- No changes outside `rupa-syntax` except task/status documentation.

## Acceptance Criteria

- [x] Parser source shrinks while preserving behavior.
- [x] All parser/lexer/syntax tests pass.
- [x] Golden PNGs, `inspect` JSON, and invalid-fixture `check --json` are
      byte-identical to `df99783`.
- [x] `cargo fmt --check`, clippy, workspace build, and workspace tests pass.
- [x] Task/status/current/handoff documentation is updated.
- [x] Commit only after explicit user authorization.

## Result

`parser.rs` fell from **3,811 to 3,351 lines** (-460, -12.1%): 1,083 lines
removed and 623 added.

Simplifications:

- 30 raw `Diagnostic { ... }` literals now use
  `DiagnosticBag::error` / `error_with_help`.
- `parse_shared_child` replaces four copies of the child-dispatch chain in the
  document, layer, group, and repeat loops.
- `expect_identifier` and `expect_variable_name` replace four hand-written
  identifier checks.
- `parse_named_or_call` shares identifier/call parsing between user functions
  and `random`.
- `parse_binary` replaces six near-identical precedence-level functions.
- `parse_unit` shares unit conversion between dimensions and primary
  expressions.
- `expression_dimension` and `parse_transform_property` remove duplicated
  conversion branches.
- Removed the single-use `parse_variable_value` wrapper.
- Test module helper macros: `parse_document!` (38 uses), `assert_parse_error!`
  (14), `child!` (35), `parse_errors!` (5), and `find_diagnostic!` (3) replace
  repeated parse/assert/match boilerplate. The test module fell from 1,159 to 987
  lines (-172).

Verification:

- 131 `rupa-syntax` library tests pass; 557 workspace tests pass.
- fmt, clippy with `-D warnings`, and workspace build are clean.
- All PNGs, valid-example `inspect` JSON, and invalid-fixture `check --json`
  are byte-identical to `df99783`.
- 20 alternating release runs against `df99783` showed no regression
  (`check minimal` 2.23 -> 2.04 ms, `render minimal` 4.39 -> 4.03 ms,
  large-source `check` 740.82 -> 733.32 ms; `paints` +1.3% within noise).

## Handoff Notes

**Completed Work**:
- Refactored parser control flow, dispatch, diagnostics, expression parsing,
  units, positions, and transform properties.
- Added unit-test helper macros for positive parses, expected failures,
  typed child extraction, and diagnostic lookup.

**Remaining Work**:
- Commit requires explicit user authorization.

**Known Issues**:
- None introduced; parser behavior is byte-identical.

**Exact Next Action**:
- Await commit authorization, then continue with the next backlog item.
