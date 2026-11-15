# TASK-104: Final M18 verification and close-out

- **Milestone**: M18 — Packed source locations
- **Priority**: High
- **Status**: Complete
- **Dependencies**: TASK-103
- **Owner**: implementation-agent

## Goal

Verify the packed-location change end to end, record the final M18 measurements,
and close the milestone with a handoff.

## Scope

- Run every required validation command with `CARGO_INCREMENTAL=0`.
- Record final `size_of` values and fixture allocations.
- Compare release CLI `check` and `render` against the M17 commit (`7bc951f`)
  for `examples/minimal.rupa`, `layers.rupa`, `text.rupa`, and `paints.rupa`.
- Confirm PNG goldens, diagnostic values, IR JSON, and wasm JSON are unchanged
  for valid files.
- Confirm the wasm release size.
- Update `agent/ROADMAP.md`, `agent/BACKLOG.md`, `agent/STATUS.md`,
  `agent/CURRENT.md`, DECISION-043, and write an M18 handoff.

## Acceptance Criteria

- [x] All validation commands pass.
- [x] `Span` is 8 bytes and `DocumentChild` is 304 bytes.
- [x] Fixture parse is 50,387 bytes.
- [x] wasm shrinks from 1,400,685 to 1,377,325 bytes (-1.7%).
- [x] PNGs are byte-identical; check/inspect JSON matches the M17 build.
- [x] Render/check performance is measured and documented.
- [x] M18 status, backlog, current task, and handoff are updated.
- [x] Commit is made only after explicit user authorization (TASK-104 commit pending).

## Result

All validation ran with `CARGO_INCREMENTAL=0`:

- `cargo fmt --all -- --check`: clean.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: clean.
- `cargo build --workspace`: OK.
- `cargo test --workspace`: 556 passed, 0 failed.
- `cargo build -p rupa-wasm --target wasm32-unknown-unknown --release`: OK.

Final layout: `FileId` 1, `Position` 4, `Location` 4, `Span` 8, `Token` 32,
`Document` 200, `DocumentChild` 304, `Text` 296, `Fill` 40, `Expression` 40,
`GroupDefinition` 96, `LayerDefinition` 208. Fixture parse: **271 allocations /
50,387 bytes**; wasm release **1,377,325 bytes**.

Release render medians (40 alternating runs against the M17 build `7bc951f`):

| Example | M17 | M18 | Delta |
|---------|----:|----:|------:|
| `minimal.rupa` | 10.48 ms | 10.70 ms | +2.1% |
| `layers.rupa` | 12.92 ms | 13.09 ms | +1.3% |
| `text.rupa` | 10.48 ms | 10.53 ms | +0.4% |
| `paints.rupa` | 21.97 ms | 22.03 ms | +0.3% |

All four PNGs are byte-identical. `check` differences were within run-to-run
noise (a 100-run minimal comparison was +0.6%). `check --json` output for every
`examples/invalid/*.rupa` fixture and `inspect` output for every valid example
are byte-identical to the M17 build. The only new diagnostic is `L010` for a
source over the 16 MiB - 1 limit.

## Handoff Notes

**Completed Work**:
- Re-ran every gate, recorded final sizes/allocations/wasm/render results, and
  verified diagnostic JSON, inspect JSON, and all PNGs against `7bc951f`.
- Closed TASK-104 and M18 documentation/state.

**Remaining Work**:
- Commit TASK-103/104 follow-up docs only after explicit user authorization.
- Evaluate a lean interner to remove `ustr`'s ~5.6 ms cold CLI initialization.
- Existing product follow-ups: static playground deployment, font upload
  control, share-by-URL.

**Test Results**:
- 556 workspace tests passed, 0 failed.
- All goldens and M17 JSON outputs are byte-identical.

**Known Issues**:
- 16 MiB - 1 and 256-file limits are now language contract.
- Detached benchmark worktrees/artifacts remain under ignored `target/`;
  cleanup requires user authorization.

**Exact Next Action**:
- Await explicit authorization for the TASK-104 documentation commit; then pick
  the lean-interner or a product follow-up.
