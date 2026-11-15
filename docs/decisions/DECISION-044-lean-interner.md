# DECISION-044: Replace `ustr` with a lean identifier interner

- **Status**: Accepted
- **Date**: 2026-11-14
- **Supersedes in part**: DECISION-042 (interning backend)
- **Related**: TASK-105, DECISION-042, DECISION-043, specification §20

## Context

DECISION-042 selected `ustr` 1.1 as the `IStr` backend and recorded a target
miss: the first intern allocates `ustr`'s global cache (`1 << 20` pointer slots
across 64 shards plus 4 MiB of bump storage), adding about 5.6 ms to one-shot
CLI runs and raising process RSS to 11.7 MB for `examples/minimal.rupa`.

TASK-105 evaluated replacing `ustr` with a lean interner while keeping the
`IStr` contract: an 8-byte `Copy` handle on 64-bit targets (4 bytes on
`wasm32`), canonical equality, process-lifetime retention, transparent plain
string serialization, and no change to diagnostics, JSON, or rendered output.

## Options considered

| Option | Result |
|--------|--------|
| Keep `ustr` and patch its initial capacity | Requires vendoring/forking `ustr` because the capacities are compile-time constants; keeps a sharded table and bump arena sized for large long-lived hosts. Rejected as a maintenance burden. |
| Revert to `Arc<str>` | No global table, but the handle is 16 bytes native; it would grow `TokenKind`, `Token`, and every AST identifier field. Already rejected by the M15–M17 memory work. |
| Indexed `u32` handle into a global vector | 4-byte handle, but every `as_str`/deref needs a locked or synchronized lookup, slowing the parser and compiler hot paths. Rejected. |
| Lean pointer interner | Starts empty, grows only with unique identifiers, keeps an 8-byte handle and O(1) `as_str`. Selected. |

## Decision

Replace `ustr` with a small interner inside `crates/rupa-syntax/src/intern.rs`:

- `IStr` stores a `&'static Entry`; `Entry` holds the text as `Box<str>` and its
  precomputed hash. Entries are leaked, matching the previous process-lifetime
  retention policy. The handle remains 8 bytes native and 4 bytes on `wasm32`.
- A process-global `OnceLock<Mutex<HashMap<&'static str, &'static Entry,
  BuildHasherDefault<DefaultHasher>>>>` canonicalizes insertions. The table
  starts empty instead of reserving a multi-megabyte cache.
- `IStr::hash` writes the entry's precomputed hash, so compiler scope and
  function maps do not re-hash identifier text on every lookup.
- `Borrow<str>` is removed. `IStr` no longer hashes exactly like `str`, so
  keeping the impl would violate the `Borrow`/`Hash` contract. `Deref`,
  `AsRef<str>`, `Display`, `Debug`, and serde behavior are unchanged.
- `ustr` is removed from the workspace and `rupa-syntax` dependencies.
- Interning scope is unchanged: identifiers and finite names only; text
  payloads, resource paths, and string literals stay `Box<str>`.

## Measurements

All CLI numbers are release-build medians of 40 alternating runs against the
M18 build on `x86_64-unknown-linux-gnu`. The large input is a 42,721-byte source
with 300 unique `let` names and 600 rectangles.

| Case | `ustr` | Lean | Delta |
|------|-------:|-----:|------:|
| `--version` | 1.84 ms | 1.77 ms | -3.6% |
| `check examples/minimal.rupa` | 8.35 ms | 2.09 ms | **-75.0%** |
| `render examples/minimal.rupa` | 9.14 ms | 3.73 ms | **-59.2%** |
| `render examples/layers.rupa` | 12.51 ms | 7.26 ms | -42.0% |
| `render examples/text.rupa` | 9.54 ms | 4.34 ms | -54.5% |
| `render examples/paints.rupa` | 21.43 ms | 16.38 ms | -23.6% |
| `check` large source | 730.77 ms | 707.78 ms | -3.1% |

Isolated first intern in a release test binary: **1.405 ms → 1.7 µs**. A repeated
intern fell from 428 ns to 107 ns. Maximum RSS for `render minimal.rupa` fell
from **11,668 KB to 4,480 KB** (-61.6%).

The wasm release module is **1,360,720 bytes**, down from 1,377,325 (-1.2%), and
rendering `examples/minimal.rupa` through the real wasm ABI produced a PNG
byte-identical to the native golden. Fixture parse allocations remain 271 /
50,387 bytes after warm-up, and AST sizes are unchanged (`IStr` stays 8 bytes).

## Verification

- All PNG goldens, `inspect` JSON for every valid example, and `check --json`
  for every invalid fixture are byte-identical to the `ustr` build.
- `cargo fmt --all -- --check`, clippy with `-D warnings`, workspace build, and
  all 557 workspace tests pass with `CARGO_INCREMENTAL=0`.
- A unit test interning the same value from eight threads confirms one canonical
  allocation, covering the global table and `IStr`'s auto `Send`/`Sync`.

## Consequences

- One-shot CLI latency is no longer dominated by interner setup; `check` on
  `minimal.rupa` is roughly four times faster end to end.
- The interner still retains every unique identifier for the process lifetime;
  memory now scales with the source's unique identifiers instead of a fixed
  multi-megabyte reservation. Large unique-name inputs pay per-entry allocation,
  but the measured 42 KB source still improved by 3.1%.
- `ustr`'s `Borrow<str>` support is gone. Any future map that wants to look up
  an `IStr` key by `&str` must use `Deref`/`as_str` or add an explicit adapter
  rather than relying on `Borrow`.
- `DefaultHasher` is used with fixed keys, so interning is deterministic and
  does not perform random-seed initialization. Collision resistance is
  sufficient for local compiler input; adversarial collision resistance was
  never part of the `ustr` integration either.
- Switching the backend again remains an internal `IStr` change as long as the
  handle size, canonical equality, serialization, and retention semantics hold.
