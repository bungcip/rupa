# TASK-003: Parser for minimal grammar

## Task Header
- **Task ID**: TASK-003
- **Title**: Parser for minimal grammar
- **Milestone**: M1 — Minimal Vertical Slice
- **Priority**: Critical
- **Estimated Effort**: 6-8 hours
- **Dependencies**: TASK-002

## Description
Build recursive descent parser producing AST from lexer tokens. Parse the minimal grammar subset for Milestone 1: document declaration, rectangle, circle, and export.

## Files to Create/Modify

| File | Action | Notes |
|------|--------|-------|
| `crates/rupa-syntax/src/ast.rs` | Create | AST node definitions |
| `crates/rupa-syntax/src/parser.rs` | Create | Parser implementation |
| `crates/rupa-syntax/src/error.rs` | Modify | Add parse errors |
| `crates/rupa-syntax/src/lib.rs` | Modify | Export modules |

## Acceptance Criteria

**All must pass to mark task complete:**

- [ ] Valid minimal example parses to correct AST
- [ ] Grammar subset parsed:
  - `document "name"` with block
  - `size <width> x <height>` (required)
  - `background <color>` (optional)
  - `export "file.png"` (required or CLI override)
  - `rectangle at (x, y)` with `size` and `fill`
  - `circle center at (x, y)` with `radius` and `fill`
- [ ] Invalid syntax produces parse error with location
- [ ] Missing required properties detected (size, position, radius, fill)
- [ ] Indentation-based block structure correctly parsed
- [ ] Expressions parsed for coordinates, dimensions, colors
- [ ] AST structure matches specification semantics
- [ ] Parser tests for valid and invalid inputs
- [ ] `cargo test -p rupa-syntax` passes
- [ ] `cargo fmt -- --check` passes
- [ ] `cargo clippy -- -D warnings` passes

## Implementation Steps

1. Define AST node types in `ast.rs`:
   - `Document`, `Size`, `Background`, `Export`
   - `Rectangle`, `Circle`
   - `Position` (at/center at), `Dimensions`, `Radius`, `Fill`
   - `Expression` (literals, binary ops, coordinate pairs)
2. Implement `Parser` struct with token stream
3. Implement parsing methods for each grammar rule
4. Handle indentation-based blocks using `Indent`/`Dedent` tokens
5. Error recovery: report first error, continue for more
6. Write comprehensive parser tests

## Testing Requirements

- Valid minimal example → correct AST
- Missing `size` → parse error
- Missing `export` → parse error (or warn if CLI override)
- Invalid indentation → parse error
- Rectangle missing `at`/`size`/`fill` → errors
- Circle missing `center at`/`radius`/`fill` → errors
- Invalid coordinate syntax → error
- Nested blocks (for future layer support) → parsed correctly

## Specification References

- §3: Source syntax overview
- §5: Document and export
- §7.1: Basic shapes (rectangle, circle)
- §4.2: Numbers and units
- §4.3: Colors
- §4.4: Expressions

## Handoff Notes

**Completed Work**:
- 

**Remaining Work**:
- 

**Test Results**:
- 

**Known Issues**:
- 

**Exact Next Action**:
- Define AST node types in ast.rs