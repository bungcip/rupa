# TASK-105: Evaluate and replace the interning backend

- **Milestone**: M19 — Lean interner
- **Priority**: High
- **Status**: Complete
- **Dependencies**: TASK-104 (M18 closed)
- **Owner**: implementation-agent

## Goal

Remove `ustr`'s fixed cold-start cost from one-shot CLI runs without changing
the `IStr` contract, AST sizes, diagnostics, JSON, or rendered output.

## Scope

- Measure the current cold-start cost and memory reservation.
- Evaluate backend options: patched `ustr`, `Arc<str>`, indexed handles, and a
  lean pointer interner.
- Implement the selected lean interner behind the existing `IStr` API.
- Keep `IStr` at 8 bytes native (4 bytes on `wasm32`), canonical, `Copy`,
  `Send`/`Sync`, and serialized as a plain string.
- Remove `ustr` from the workspace and `rupa-syntax` dependencies.
- Re-run size/allocation guardrails, workspace validation, wasm, output parity,
  and release CLI benchmarks.
- Record the decision in DECISION-044 and close M19.

## Non-Goals

- No change to interning scope: identifiers and finite names only. Text
  payloads, resource paths, and string literals remain `Box<str>`.
- No change to diagnostics, JSON, PNG output, language behavior, or wasm ABI.
- No change to AST layouts or source-location limits.

## Acceptance Criteria

- [x] First intern drops from millisecond scale to microsecond scale.
- [x] `check minimal.rupa` and all example renders improve materially.
- [x] A large source with many unique identifiers does not regress.
- [x] Peak RSS falls relative to the `ustr` build.
- [x] `size_of::<IStr>() <= 8` on 64-bit targets and `<= 4` on `wasm32`.
- [x] Equal identifiers from multiple threads share one canonical allocation.
- [x] PNG goldens, `inspect` JSON, and invalid-fixture `check --json` are
      byte-identical to the M18 build.
- [x] wasm release builds and its rendered PNG matches the native golden.
- [x] `cargo fmt`, clippy, workspace build, and workspace tests pass.
- [x] DECISION-044, roadmap, backlog, status, current task, and handoff are
      updated.

## Result

Selected backend: a global `OnceLock<Mutex<HashMap<...>>>` holding leaked
`Entry { hash, text }` values; `IStr(&'static Entry)` remains 8 bytes native and
4 bytes on `wasm32`. `IStr::hash` uses the precomputed entry hash. `Borrow<str>`
was removed because `IStr` no longer hashes exactly like `str`.

Release CLI medians (40 alternating runs) against the M18 `ustr` build:

| Case | `ustr` | Lean | Delta |
|------|-------:|-----:|------:|
| `--version` | 1.84 ms | 1.77 ms | -3.6% |
| `check examples/minimal.rupa` | 8.35 ms | 2.09 ms | -75.0% |
| `render examples/minimal.rupa` | 9.14 ms | 3.73 ms | -59.2% |
| `render examples/layers.rupa` | 12.51 ms | 7.26 ms | -42.0% |
| `render examples/text.rupa` | 9.54 ms | 4.34 ms | -54.5% |
| `render examples/paints.rupa` | 21.43 ms | 16.38 ms | -23.6% |
| `check` 42,721-byte source | 730.77 ms | 707.78 ms | -3.1% |

Additional results:

- First intern: 1.405 ms → 1.7 µs; repeated intern: 428 ns → 107 ns.
- Peak RSS for `render minimal.rupa`: 11,668 KB → 4,480 KB (-61.6%).
- wasm release: 1,377,325 → 1,360,720 bytes (-1.2%).
- Fixture parse: 271 allocations / 50,387 bytes after warm-up, unchanged.
- All golden/JSON outputs are byte-identical; wasm ABI render matches native.
- 557 workspace tests pass; fmt, clippy, build, and wasm release are clean.

## Handoff Notes

**Completed Work**:
- Replaced `ustr` with a lean interner, added a cross-thread canonicalization
  test, removed the dependency, recorded DECISION-044, and re-verified sizes,
  outputs, wasm, tests, and release performance.

**Remaining Work**:
- Commit requires explicit user authorization.
- Product follow-ups remain: deploy the playground as a static site, add a font
  upload control (`rupa_set_font`), and share-by-URL.

**Known Issues**:
- The lean interner still retains unique identifiers for the process lifetime,
  matching the documented `IStr` contract.
- `Borrow<str>` is no longer implemented for `IStr`.

**Exact Next Action**:
- Await commit authorization, then pick a product follow-up.
