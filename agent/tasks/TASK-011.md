# TASK-011: Integration test with golden image

## Task Header
- **Task ID**: TASK-011
- **Title**: Integration test with golden image
- **Milestone**: M1 — Minimal Vertical Slice
- **Priority**: Critical
- **Estimated Effort**: 3-4 hours
- **Dependencies**: TASK-010

## Description
Create end-to-end integration test that compiles a minimal `.rupa` example and compares the rendered PNG against a golden reference image. Establish golden image testing infrastructure.

## Files to Create/Modify

| File | Action | Notes |
|------|--------|-------|
| `crates/rupa-cli/tests/integration_test.rs` | Create | Integration test suite (root has no package; see DECISION-008) |
| `crates/rupa-cli/tests/common/mod.rs` | Create | Reusable test/golden utilities |
| `examples/minimal.rupa` | Create | Test input file |
| `tests/golden/minimal.png` | Create | Golden reference image |

## Acceptance Criteria

**All must pass to mark task complete:**

- [x] Example compiles without errors via CLI
- [x] Output PNG matches golden reference (exact pixel match)
- [x] Test runs in `cargo test --workspace` (rupa-cli integration target)
- [x] Golden image added at `tests/golden/minimal.png` (pending commit)
- [x] Deterministic output verified (byte-identical runs and golden)
- [x] Test infrastructure supports adding more golden tests (`assert_render_matches_golden`)
- [x] `cargo fmt -- --check` passes
- [x] `cargo clippy -- -D warnings` passes

## Implementation Steps

1. Create `examples/minimal.rupa` with test content
2. Run `rupa render examples/minimal.rupa --output tests/golden/minimal.png` to generate initial golden
3. Verify golden image visually correct
4. Create `tests/integration_test.rs` with test that:
   - Runs CLI render on example
   - Loads generated PNG and golden PNG
   - Compares pixel-by-pixel
   - Asserts exact match
5. Create test utilities for image comparison
6. Add test to workspace

## Example Source (examples/minimal.rupa)

```rupa
document "Test"
    size 200px x 100px
    background white

    rectangle at (10px, 10px)
        size 80px x 40px
        fill red

    circle center at (150px, 50px)
        radius 30px
        fill blue

export "test.png"
```

## Testing Requirements

- Pixel-exact comparison (no tolerance for M1)
- Test runs in CI
- Golden image regenerated only intentionally
- Test documents expected colors at specific coordinates
- Determinism: run test twice, both pass

## Specification References

- §23.4: Golden image tests
- §24: Milestone 1 acceptance criteria
- §18.2: Determinism

## Handoff Notes

**Completed Work**:
- Added `examples/minimal.rupa` (200×100 white canvas, red rectangle, blue
  circle) and the committed golden `tests/golden/minimal.png`.
- Added reusable golden-test utilities in
  `crates/rupa-cli/tests/common/mod.rs`:
  - workspace/fixture path resolution
  - CLI runner, PNG decoder, pixel-exact image comparison
  - `assert_render_matches_golden(source, golden_name)`
- Added 5 end-to-end tests in
  `crates/rupa-cli/tests/integration_test.rs`:
  - example compiles via `rupa check` with no diagnostics
  - rendered output matches golden pixel-exactly
  - golden dimensions and known colors are asserted
  - two renders are byte-identical and match the golden bytes
  - comparison helper detects a deliberately mismatched image
- Recorded test location and policy in `docs/decisions/DECISION-008.md`.

**Remaining Work**:
- None for TASK-011.

**Test Results**:
- `cargo test --workspace`: passes (rupa-cli 12 + 5 integration,
  rupa-compiler 74, rupa-geometry 9, rupa-image 7, rupa-ir 10,
  rupa-renderer 23, rupa-syntax 77).
- `cargo clippy --all-targets --all-features -- -D warnings`: passes.
- `cargo fmt -- --check`: passes.

**Known Issues**:
- Golden PNG must be regenerated intentionally if renderer/PNG encoder output
  changes.
- Integration tests live in the `rupa-cli` package because the workspace root
  is a virtual manifest (DECISION-008).

**Exact Next Action**:
- Milestone 1 is complete. Begin Milestone 2 with TASK-012.
