# DECISION-042: Measure AST memory and select the interning backend

- **Status**: Accepted
- **Date**: 2026-11-14
- **Superseded in part by**: DECISION-043 (source-location layout) and
  DECISION-044 (interning backend)
- **Related**: TASK-097, TASK-098, TASK-099, TASK-100, TASK-101, TASK-102,
  specification §20

## Context

The AST baseline audit (`agent/tasks/TASK-097.md`) identified source spans and
repeated string fields as the main memory cost. Before changing representations,
Milestone 15 needs measured numbers, deterministic regression guardrails, and a
decision about how identifiers will be interned.

At TASK-097 time the shell could not reach the registry index and neither
`ustr` nor `stats_alloc` was available locally, so TASK-097 used an in-house
counting allocator and recorded an `Arc<str>` fallback. The registry was later
confirmed to be online, so TASK-099 should add `ustr` while keeping the fallback
documented for environments where it is unavailable.

## Baseline

Measured with `cargo test -p rupa-syntax --test sizes -- --nocapture` on
`x86_64-unknown-linux-gnu` (debug profile, 64-bit pointers):

| Type | Bytes |
|------|------:|
| `FileId` | 8 |
| `Position` | 24 |
| `Span` | 56 |
| `Token` | 88 |
| `TokenKind` | 32 |
| `Document` | 456 |
| `DocumentChild` | 1456 |
| `Text` | 1432 |
| `Fill` | 600 |
| `Expression` | 200 |
| `GroupDefinition` | 304 |
| `LayerDefinition` | 992 |
| `Arc<str>` (fallback payload handle) | 16 |

Parsing `crates/rupa-syntax/tests/fixtures/baseline.rupa` (3 repeated groups,
4 layers, repeated placements, and repeated text payloads) allocates
**381 times / 150,297 bytes**, measured with the in-house counting allocator in
`crates/rupa-syntax/tests/allocations.rs`. Two consecutive parses produce the
same counts.

The table drives the planned work:

- `Span` is 56 bytes and appears throughout the AST; TASK-098 reduced it to
  40 bytes by moving `FileId` and line/column counters to `u32` while keeping
  the byte offset at 64 bits.
- `DocumentChild` (1456) and `Text` (1432) are large mostly because
  `Text` carries two `String` payloads (`text` and `content`) plus several
  `Option<Position>` values. TASK-100 removes the duplicate field and
  TASK-101 boxes large variants only where the measurements justify it.
- Repeated `"Card"`, `"Background"`, `"Repeated label"`, and `"#7950F2"`
  payloads show that interning identifiers and finite names can remove many
  duplicate allocations. Large unique payloads must not be interned.

## Decision

1. **Use `ustr` 1.1 as the primary `IStr` backend.** `IStr` wraps
   `ustr::Ustr` and provides `Clone`, `Copy`, `Deref<Target = str>`,
   `AsRef<str>`, `Borrow<str>`, `PartialEq`/`Eq`/`Hash` by string value, and
   plain-string serde. Serde is implemented on `IStr` directly, so the backend's
   optional `serde` feature is not required and serialization stays a plain
   JSON string. Keep the documented in-house fallback of `Arc<str>` for builds
   where `ustr` is unavailable: `size_of::<Arc<str>>()` is 16 bytes on native
   and two 4-byte words (8 bytes) on `wasm32`. The `ustr` handle is 8 bytes
   native and 4 bytes wasm32.
2. **Interning scope**: intern identifiers, keyword-like finite names, group and
   layer names, named colours, pattern kinds, blend names, and font families.
   Do not intern string literals, resource paths, or other large unique
   payloads; store those as `Box<str>`.
3. **Serialization stays unchanged**: `IStr` serializes as a plain JSON string
   and deserializes from a string. Diagnostics, JSON, and the wasm wire format
   keep their current shapes and values.
4. **Measurement tooling stays dependency-free**: use a test-only counting
   global allocator instead of `stats_alloc`. The allocator lives in one
   integration-test binary, counts `alloc`/`alloc_zeroed`/`realloc`, and is
   enabled only around the measured parse, so the numbers are deterministic.
5. **Guardrails**: `crates/rupa-syntax/tests/sizes.rs` asserts the recorded
   upper bounds and `Span <= 40`. The allocation test asserts both run-to-run
   determinism and an upper bound of 420 allocations / 130,000 bytes.

## TASK-098 result

`FileId` is now `u32`, `Position` holds `line: u32`, `column: u32`, and
`offset: u64`, and `Span` is **40 bytes** (16-byte `Position`). Lexer counters
are `u32` and saturate instead of wrapping; host-facing conversions use
`Position::new` / `Position::from_usize` / `Position::offset_usize`. Reported
line, column, and byte-offset values, diagnostic text, JSON, and golden images
are unchanged.

| Type | Baseline | TASK-098 |
|------|---------:|---------:|
| `FileId` | 8 | 4 |
| `Position` | 24 | 16 |
| `Span` | 56 | 40 |
| `Token` | 88 | 72 |
| `Document` | 456 | 376 |
| `DocumentChild` | 1456 | 1168 |
| `Text` | 1432 | 1160 |
| `Fill` | 600 | 504 |
| `Expression` | 200 | 152 |
| `GroupDefinition` | 304 | 240 |
| `LayerDefinition` | 992 | 832 |

Parsing the fixture now allocates **381 times / 123,097 bytes** (down from
150,297 bytes); the allocation count is unchanged because spans are inline.

## TASK-099 result

`IStr` is a `Copy` 8-byte handle over `ustr::Ustr` on 64-bit targets, with a
compile-time `<= 4` bound that the `wasm32-unknown-unknown` release build
checks. `TokenKind::Identifier` uses `IStr`, so `TokenKind` fell from 32 to
24 bytes and `Token` from 72 to 64 bytes. The lexer scans identifiers directly
from the source slice instead of building a temporary `String`; repeated
identifiers share one interned payload.

String literals use `Box<str>` and are deliberately not interned. A lexer test
confirms that two equal literals keep distinct allocations, while two equal
identifiers share the same pointer.

After warming the intern table, parsing the fixture allocates **304 times /
114,327 bytes**, down from 381 / 123,097 in TASK-098. The allocation count drops
because repeated identifier strings no longer allocate; the bytes drop because
the lexer no longer builds per-identifier `String`s.

The intern table retains interned identifiers for the process lifetime by
design. Only identifiers and finite names enter it; text content, resource
paths, and string literals stay as `Box<str>` so large unique payloads are not
retained globally. Serialized tokens, diagnostics, and JSON remain unchanged.

## TASK-100 result

AST identifier and finite-name fields now use `IStr`; unique payloads use
`Box<str>`. The duplicate `Text.content` field is removed, so `Text` stores its
content once and serializes only a `text` key. Compiler scopes and function
tables key on `IStr`, and string values stay interned through evaluation. IR
fields remain `String` and are converted once at the lowering boundary.

| Type | TASK-097 baseline | TASK-100 |
|------|------------------:|---------:|
| `Document` | 456 | 360 |
| `DocumentChild` | 1456 | 1056 |
| `Text` | 1432 | 1016 |
| `Fill` | 600 | 408 |
| `Expression` | 200 | 152 |
| `GroupDefinition` | 304 | 224 |
| `LayerDefinition` | 992 | 704 |

After intern-table warm-up the fixture parse is **278 allocations /
109,651 bytes**, down from 304 / 114,327 in TASK-099. Diagnostic messages,
codes, spans, JSON values, and golden images are unchanged.

## TASK-101 result

Measured variant payloads showed that `Fill` (408 bytes) dominated every shape
node, not `Text` (1016) or `Path` (768). Boxing the gradient and pattern
payloads reduced `Fill` to 72 bytes and therefore reduced the largest
`DocumentChild` variants from 1048 to 712 bytes. `DocumentChild` fell from 1056
to **720 bytes** (50.5% below the TASK-097 baseline of 1456).

| Item | Before | After |
|------|-------:|------:|
| `Fill` | 408 | 72 |
| `DocumentChild` | 1056 | 720 |
| `DocumentChild::Rectangle` / `Ellipse` / `Triangle` | 1048 | 712 |
| `DocumentChild::Text` | 1016 | 680 |
| `DocumentChild::Path` | 768 | 432 |
| `LayerDefinition` | 704 | 368 |
| `Expression` | 152 | 72 |
| `Repeat` | 224 | 144 |
| `Variable` | 200 | 120 |

Boxing `Expression::Position` reduced `Expression` from 152 to 72 bytes; only
position expressions pay one allocation. `DocumentChild::Text` and
`DocumentChild::Path` were **not** boxed: after the `Fill` change the largest
variant is the 712-byte rectangle/ellipse/triangle, so boxing Text or Path would
not reduce the 720-byte enum and would add an allocation for common nodes.

Finite names (`blend`, filter `kind`, pattern `kind`) remain `IStr`. Converting
them to closed enums would either change diagnostics for unknown values or
require an extra raw-name variant, and `IStr` is already pointer-sized.

Parse allocation for the fixture fell to **271 allocations / 101,203 bytes**
(from 278 / 109,651). Each boxed payload is exactly one `Box::new` per node that
uses it. The release render benchmark on `examples/paints.rupa` was unchanged
within noise (40 alternating runs: median 20.97 -> 20.94 ms, mean -0.9%), and
the rendered PNG was byte-identical.

## Final verification (TASK-102)

Final sizes against the TASK-097 baseline (64-bit, debug test):

| Type | Baseline | Final | Change |
|------|---------:|------:|-------:|
| `FileId` | 8 | 4 | -50.0% |
| `Position` | 24 | 16 | -33.3% |
| `Span` | 56 | 40 | -28.6% |
| `Token` | 88 | 64 | -27.3% |
| `TokenKind` | 32 | 24 | -25.0% |
| `Document` | 456 | 360 | -21.1% |
| `DocumentChild` | 1456 | 720 | -50.5% |
| `Text` | 1432 | 680 | -52.5% |
| `Fill` | 600 | 72 | -88.0% |
| `Expression` | 200 | 72 | -64.0% |
| `GroupDefinition` | 304 | 224 | -26.3% |
| `LayerDefinition` | 992 | 368 | -62.9% |

`Document` is the one type below the 25% target (21.1%): its own fields are
vector handles and option discriminants, so the large node-type reductions do
not compound inside the small root struct.

The repetitive fixture parse is **271 allocations / 101,203 bytes**, down from
381 / 150,297 at TASK-097: **32.7% fewer allocated bytes**. The wasm release
module is 1,400,685 bytes versus 1,388,056 at the TASK-097 baseline (+0.91%,
inside the 5% budget). Every workspace test, diagnostic/JSON test, wasm parity
test, and PNG golden test passes.

### Render-time target

Measured end-to-end release CLI medians over 40 alternating runs against a build
of the TASK-097 commit:

| Example | Baseline render | Final render | Delta |
|---------|----------------:|-------------:|------:|
| `examples/minimal.rupa` | 4.78 ms | 11.27 ms | +135.8% |
| `examples/layers.rupa` | 8.98 ms | 15.36 ms | +70.9% |
| `examples/text.rupa` | 5.27 ms | 12.29 ms | +133.1% |

The rendered PNGs are byte-identical. The regression is a fixed cold-start cost,
not renderer work: `check` on the same examples regresses by ~5.6 ms, and
`--version` is unchanged. `ustr` allocates its global cache on first intern
(`INITIAL_CAPACITY = 1 << 20` pointers and 4 MiB of bump storage across 64
bins); process RSS rises from 3.6 MB to 10.9 MB even for `minimal.rupa`, which
only needs the built-in colour names. The renderer and lowering paths are
otherwise unchanged, and TASK-101's alternating benchmark (both builds already
using `ustr`) showed no regression.

**Decision: keep the changes and record the miss.** The memory and wasm targets
pass, rendering output is unchanged, and long-lived hosts (the playground
wasm module) pay the intern-table setup once. The 5% render-time target is not
met for cold one-shot CLI runs. A follow-up should evaluate a lean interner
(smaller initial capacity, or an `Arc<str>`/indexed fallback) before treating
cold CLI latency as closed.

**Follow-up:** TASK-105 replaced `ustr` with a lean pointer interner; see
DECISION-044. `check minimal.rupa` fell from 8.35 ms to 2.09 ms and peak RSS
from 11.7 MB to 4.5 MB, so the cold-start miss is closed.

## Consequences

- TASK-099/TASK-100 pay a small `ustr` handle instead of a 24-byte `String`
  plus one allocation. The fallback keeps the repository buildable if `ustr` is
  unavailable, and switching backends remains an internal `IStr` change.
- Diagnostic and JSON positions still serialize as ordinary JSON numbers with
  the same values; only the in-memory widths changed.
- The measurement tests are not golden-output tests; they must not change any
  production syntax, diagnostic, JSON, or renderer behavior.
