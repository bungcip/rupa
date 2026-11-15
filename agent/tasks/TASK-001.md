# TASK-001: Workspace and crate structure

## Task Header
- **Task ID**: TASK-001
- **Title**: Workspace and crate structure
- **Milestone**: M1 — Minimal Vertical Slice
- **Priority**: Critical
- **Estimated Effort**: 2-3 hours
- **Dependencies**: None

## Description
Initialize Rust workspace with crate structure per specification §20. Create root Cargo.toml with workspace configuration and 7 crates with minimal scaffolding.

## Files to Create/Modify

| File | Action | Notes |
|------|--------|-------|
| `Cargo.toml` | Create | Workspace root |
| `crates/rupa-syntax/Cargo.toml` | Create | Lexer, parser, AST |
| `crates/rupa-syntax/src/lib.rs` | Create | Minimal lib |
| `crates/rupa-compiler/Cargo.toml` | Create | Name resolution, semantic analysis |
| `crates/rupa-compiler/src/lib.rs` | Create | Minimal lib |
| `crates/rupa-ir/Cargo.toml` | Create | Graphics IR and validation |
| `crates/rupa-ir/src/lib.rs` | Create | Minimal lib |
| `crates/rupa-geometry/Cargo.toml` | Create | Geometry, paths, transforms |
| `crates/rupa-geometry/src/lib.rs` | Create | Minimal lib |
| `crates/rupa-renderer/Cargo.toml` | Create | CPU rendering and compositing |
| `crates/rupa-renderer/src/lib.rs` | Create | Minimal lib |
| `crates/rupa-image/Cargo.toml` | Create | Image resources and pixel buffers |
| `crates/rupa-image/src/lib.rs` | Create | Minimal lib |
| `crates/rupa-cli/Cargo.toml` | Create | CLI commands |
| `crates/rupa-cli/src/main.rs` | Create | Entry point |

## Acceptance Criteria

**All must pass to mark task complete:**

- [x] `cargo build --workspace` succeeds
- [x] `cargo test --workspace` passes (empty test suites OK)
- [x] `cargo fmt -- --check` passes
- [x] `cargo clippy -- -D warnings` passes
- [x] No cyclic dependencies between crates
- [x] Workspace uses Rust 2024 edition
- [x] All crates publish = false (private)

## Implementation Steps

1. Create root `Cargo.toml` with `[workspace]` configuration
2. Create each crate directory and `Cargo.toml`
3. Add minimal `lib.rs` (or `main.rs` for cli) with `//!` crate docs
4. Declare crate dependencies in root Cargo.toml `[workspace.dependencies]`
5. Run `cargo build --workspace` to verify
6. Run `cargo fmt -- --check`
7. Run `cargo clippy -- -D warnings`
8. Run `cargo test --workspace`

## Testing Requirements

- No functional tests needed for scaffolding
- Build and lint checks serve as verification

## Specification References

- §20: Rust workspace proposal
- §2: Coding conventions (edition, formatting, linting)

## Handoff Notes

**Completed Work**:
- Initialized Rust workspace with 7 crates
- Created root Cargo.toml with workspace configuration
- Created all crate manifests with appropriate dependencies
- Implemented minimal lib.rs/main.rs scaffolding for all crates
- Fixed all compilation errors, clippy warnings, and formatting issues
- All validation commands pass

**Remaining Work**:
- None for this task

**Test Results**:
- `cargo build --workspace`: ✓
- `cargo test --workspace`: ✓ (7 crates, 0 tests each)
- `cargo fmt -- --check`: ✓
- `cargo clippy -- -D warnings`: ✓

**Known Issues**:
- No unit tests implemented yet (acceptable for initial scaffold)

**Exact Next Action**:
- Start TASK-002: Implement indentation-aware lexer