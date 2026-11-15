# LLM Benchmark Suite

This directory contains the offline benchmark suite for LLM-generated Rupa
source (specification §23.5). The harness does not call a model; it evaluates
sources produced elsewhere.

## Layout

```
benchmarks/
├── tasks.json              # suite manifest: prompts, task kinds, solutions
├── solutions/              # canonical solution for every task
└── starters/               # starting source for modify/repair tasks
```

Candidate sources live outside this directory and are supplied with
`--candidates`. For each task, the harness looks for either:

- `<candidates>/<task-id>.rupa` — a single attempt, or
- `<candidates>/<task-id>/attempt-*.rupa` — ordered attempts (lexicographic).

Attempts after the first count as repair iterations.

## Running

```bash
rupa bench --tasks benchmarks --candidates path/to/candidates
rupa bench --tasks benchmarks --candidates path/to/candidates --output report.json
```

The JSON report is printed to stdout when `--output` is omitted. A benchmark run
exits successfully even when tasks fail; failures are reported as metrics.

## Task kinds

| Kind | Meaning |
|------|---------|
| `create` | Produce a new document from the prompt. |
| `modify` | Start from `starter` and produce the requested change. |
| `repair` | Start from intentionally invalid source and fix it. |
| `reference` | Reproduce the composition described by the prompt. |

`starter` and `expected_diagnostics` are optional. When present, the harness
verifies that the starter still emits the documented diagnostic codes so the
benchmark cannot silently drift.

## Metrics

Each attempt records compilation success, rendering success, syntax and semantic
error counts, diagnostic codes, lexical token count, source bytes, source lines,
exact pixel-match ratio, and normalized visual similarity to the canonical
solution render. Visual similarity is `1 - mean absolute channel difference` over
premultiplied RGBA pixels; failed or size-mismatched renders score `0`.

The suite report aggregates first-attempt compilation success, final compilation
success, rendering success, the fraction of attempts with syntax/semantic
errors, average repair iterations, average final visual similarity, and mean
token/source size. Token count is the number of lexer tokens excluding layout
tokens; it is a deterministic proxy for model token usage, not an exact
provider tokenizer count.
