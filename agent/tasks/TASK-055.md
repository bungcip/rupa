# TASK-055: LLM benchmark suite

**Milestone**: 8 — LLM Authoring Experience
**Dependencies**: TASK-010, TASK-050, TASK-054
**Status**: Complete

## Goal

Provide an offline, reproducible benchmark suite that evaluates LLM-generated
Rupa source and reports the metrics required by specification §23.5.

## Scope

- Add `benchmarks/tasks.json` with eight tasks covering:
  basic shape, poster layers, layer reuse, procedural pattern,
  paths/gradients/text, modifying existing source, repairing invalid source,
  and reproducing a reference composition.
- Add canonical solutions and starters under `benchmarks/`.
- Add `rupa bench` to evaluate candidates without calling any model provider.
  Candidates are supplied as `<task-id>.rupa` or ordered
  `<task-id>/attempt-*.rupa` files.
- Report per-attempt and aggregate metrics:
  - first-attempt compilation success
  - final compilation success
  - syntax and semantic error rates
  - repair iterations
  - rendering success
  - visual similarity and exact pixel-match ratio against the canonical render
  - lexical token count and source length
- Validate benchmark integrity: starters for repair tasks must still emit their
  documented diagnostic codes.
- Add integration tests for perfect reference runs, repair iterations, failed
  candidates, and unattempted tasks.
- Document the suite layout and metric definitions in `benchmarks/README.md`.

## Files Created/Modified

- `crates/rupa-cli/src/benchmark.rs`
- `crates/rupa-cli/src/commands.rs`
- `crates/rupa-cli/src/main.rs`
- `crates/rupa-cli/tests/benchmark.rs`
- `benchmarks/tasks.json`
- `benchmarks/solutions/*.rupa`
- `benchmarks/starters/*.rupa`
- `benchmarks/README.md`

## Metric Definitions

- **Compilation success**: no error diagnostics and graphics IR was produced.
- **Syntax errors**: error diagnostics with `L*` or `P*` codes.
- **Semantic errors**: other error diagnostics.
- **Repair iterations**: attempts after the first.
- **Visual similarity**: `1 - mean absolute channel difference` over
  premultiplied RGBA pixels; size mismatches and failed renders score `0`.
- **Token count**: lexer tokens excluding layout tokens. This is a deterministic
  proxy, not an exact provider tokenizer count.

## Acceptance Criteria

- [x] Eight benchmark tasks with prompts, solutions, starters, and documented
      expected diagnostics
- [x] Candidate evaluation is offline and model-provider independent
- [x] All specification §23.5 metrics are reported
- [x] Invalid candidates do not crash the harness and score zero where appropriate
- [x] Reference solutions score perfect compilation, rendering, and similarity
- [x] `cargo fmt -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`,
      `cargo build --workspace`, and `cargo test --workspace` pass

## Verification

- `cargo fmt -- --check` — pass
- `cargo clippy --all-targets --all-features -- -D warnings` — pass
- `cargo build --workspace` — pass
- `cargo test --workspace` — pass (benchmark module 5 unit tests, CLI benchmark 4 integration tests; full workspace green)
