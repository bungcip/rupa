# DECISION-034: Retain legacy diagnostic codes as catalog-only entries

> **Superseded by DECISION-041.** The legacy codes were deleted before the first
> release.

**Date**: 2026-11-13
**Task**: documentation sync (M10 follow-up)

## Context

After the M9 document-model migration and the M10 group model, several
diagnostic codes no longer describe a condition the current language can
produce:

- `C001` `MISSING_DOCUMENT` and `C002` `MULTIPLE_DOCUMENTS` (pre-M9
  `document` wrapper).
- `C004` `MISSING_EXPORT`, `C006` `INVALID_EXPORT_PATH`, and `C012`
  `DUPLICATE_EXPORT` (removed `export` setting).
- `C005` `MISSING_DOCUMENT_SIZE` (`size` is now optional).
- `S015` `MISSING_LAYER_SIZE` (layer `size` is now optional, defaulting to
  the document canvas).
- `S019` `UNKNOWN_LAYER` and `S021` `RECURSIVE_LAYER_REFERENCE` (placements now
  reference groups; see `S033`/`S035`).

The question is whether to delete these codes or keep them.

## Options considered

1. Delete the codes and renumber. Cleaner catalog, but breaks any external
   tooling, fixtures, or cached diagnostics that reference the old numbers,
   and renumbering churns every diagnostic test.
2. Keep the codes but stop emitting them; mark them "Legacy" in the catalog
   and retain them for migration/back-compat reference.
3. Keep emitting them as warnings for old sources. Redundant with the newer
   migration diagnostics (`P004`, `P005`, `S033`, `S035`) and noisy.

## Decision

Option 2. The legacy codes remain defined in `crates/rupa-syntax/src/diagnostic.rs`
with a `Legacy:` prefix in their message text, so the catalog stays stable and
existing references keep resolving. They are **not emitted** by the current
parser, compiler, or checker: migration is handled by the active diagnostics
(`P004`/`P005` for removed `document`/`export`, `S033`/`S034`/`S035` for
group placement). The codes are retained for historical reference and for any
tooling that still maps old diagnostics.

## Consequences

- The diagnostic catalog is stable; no renumbering.
- Legacy codes are catalog-only and must not be added to new emission sites.
- Tests that assert code strings (e.g. in `diagnostic.rs` unit tests) continue
  to pass unchanged.
- `docs/` and `agent/` documentation should describe these codes as legacy only.
- If a future cleanup removes them entirely, it must be a coordinated change
  with a new decision record and updated tests.
}