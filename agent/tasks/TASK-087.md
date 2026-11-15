# TASK-087: Structured fix-it edits and `rupa check --json`

## Task Header

- **Task ID**: TASK-087
- **Title**: Machine-applicable `edits` on diagnostics, exposed through `rupa check --json`
- **Milestone**: Post-M14 follow-ups
- **Priority**: High
- **Dependencies**: none
- **Status**: Complete

---

## Description

Follow-up item 1: diagnostics that already carry an exact fix now also carry a
structured edit, and the CLI can print the whole diagnostic set as JSON.

### `edits` on diagnostics

`rupa_syntax::diagnostic::Diagnostic` has a new field:

```rust
pub edits: Vec<SuggestedEdit>,

pub struct SuggestedEdit {
    pub span: Span,
    pub replacement: String,
}
```

It is `#[serde(default, skip_serializing_if = "Vec::is_empty")]`, so serialized
diagnostics without a fix are byte-identical to before. Edits are populated
where the fix is exact:

- `Diagnostic::with_suggestion` — the unknown-keyword token span is replaced by
  the suggested keyword (`fil` → `fill`).
- Unknown named colour (`T002`) — the colour span is replaced by the suggested
  name (`gren` → `green`, `colour` → `red`).

Advisory-only suggestions (unknown group/variable names) intentionally carry no
edit: their primary span covers a larger construct, so replacing it verbatim
would be wrong.

### `rupa check --json`

```
rupa check poster.rupa --json
```

prints one object to stdout and keeps the existing exit status (1 when any
diagnostic is an error):

```json
{
  "path": "poster.rupa",
  "status": "error",
  "diagnostics": [
    {
      "code": "L006",
      "severity": "error",
      "message": "unknown keyword or property `fil`",
      "line": 2, "column": 1, "endLine": 2, "endColumn": 4,
      "help": "did you mean `fill`?",
      "edits": [
        {
          "startLine": 2, "startColumn": 1, "startOffset": 19,
          "endLine": 2, "endColumn": 4, "endOffset": 22,
          "replacement": "fill"
        }
      ]
    }
  ]
}
```

Offsets are byte offsets into the source. Diagnostics keep their existing codes,
messages, and human rendering; `--json` only changes the output channel.

## Tests

- `crates/rupa-syntax/src/diagnostic.rs`: suggestion records an edit with the
  primary span and candidate; no candidate means no help and no edit.
- `crates/rupa-syntax/src/parser.rs`: `fil red` produces `L006` whose edit
  replaces the token span with `fill`.
- `crates/rupa-compiler/src/paint.rs`: `gren` carries a `green` edit; `colour`
  carries none.
- `crates/rupa-cli/tests/integration_test.rs`: `check --json` for a valid file
  (`status: ok`, no diagnostics), an unknown keyword (edit spans/offsets match
  `fil` on line 2), and an unknown named colour (`T002` → `green`); JSON mode
  prints nothing to stderr and still exits 1 on errors.

## Validation

`cargo fmt -- --check`, `cargo clippy --all-targets --all-features -D warnings`,
`cargo build --workspace`, `cargo test --workspace` all pass (see handoff).

## Design record

`docs/decisions/DECISION-036-diagnostic-edits.md`; specification §22.3 describes
the JSON shape.
