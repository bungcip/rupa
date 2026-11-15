# TASK-091: Deterministic vendored default font for `text`

## Task Header

- **Task ID**: TASK-091
- **Title**: Vendor and pin the Rupa Sans default font; remove system-font dependence
- **Milestone**: Post-M14 follow-ups
- **Priority**: High
- **Dependencies**: TASK-073, TASK-074
- **Status**: Complete

---

## Description

`text` depended on whatever font the host had installed, so the same source
could render differently on another machine, and the browser playground always
reported `R003` because it had no font source. This was the remaining real hole
in the determinism principle.

The renderer now embeds **Rupa Sans** regular and bold, a small OFL-licensed
5x7-outline face covering printable ASCII. The family name and both SHA-256
digests are pinned in `rupa-renderer::fonts`; a unit test verifies the embedded
bytes against the pins.

Resolution is deterministic:

- no `font` or `font "Rupa Sans"` uses the embedded face (bold for
  `weight bold`);
- native builds no longer read system fonts, so an explicit unknown family
  reports `R003`;
- browser builds use embedded faces by default and may install extra faces
  explicitly through `rupa_set_font`.

Assets live in `crates/rupa-renderer/assets/` with `OFL.txt` and a reproducible
generator. The wasm module and playground therefore render `text` offline with
no upload control.

## Acceptance Criteria

- [x] Regular and bold Rupa Sans faces are embedded and parse with `fontdue`.
- [x] Family/weight selection is pinned to the embedded bytes.
- [x] SHA-256 pins match the checked-in files.
- [x] Native resolution no longer consults system fonts.
- [x] Unknown explicit families report `R003` deterministically.
- [x] Browser default text uses the same embedded bytes, without `rupa_set_font`.
- [x] `examples/text.rupa` renders to `tests/golden/text.png`, and the wasm test
      matches that golden byte for byte.
- [x] Playground has a `text.rupa` example and documents the new limit.
- [x] Spec §9 and §18 describe the deterministic font policy.
- [x] fmt/clippy/build/workspace tests pass.
