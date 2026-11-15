//! Allocation-count baseline for parsing (TASK-097, DECISION-042).
//!
//! Uses `stats_alloc` to measure allocator traffic around the parse only.
//! The counters are process-global and this integration-test binary runs a
//! single test, so the measured region stays deterministic.

use std::alloc::System;

use rupa_syntax::{FileId, Lexer, Parser};
use stats_alloc::{INSTRUMENTED_SYSTEM, Region, Stats, StatsAlloc};

#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

const FIXTURE: &str = include_str!("fixtures/baseline.rupa");

fn measure_parse(source: &str) -> Stats {
    let region = Region::new(GLOBAL);

    let (tokens, mut diagnostics) = Lexer::new(source.to_string(), FileId(0)).tokenize();
    let (document, parser_diagnostics) = Parser::new(tokens).parse();

    let stats = region.change();

    diagnostics.append(parser_diagnostics);
    assert!(
        !diagnostics.has_errors(),
        "baseline fixture must parse cleanly: {:?}",
        diagnostics
            .errors()
            .into_iter()
            .map(|diagnostic| diagnostic.message.clone())
            .collect::<Vec<_>>()
    );
    assert!(
        document.is_some(),
        "baseline fixture must produce a document"
    );

    stats
}

#[test]
fn records_the_parse_allocation_baseline() {
    // Warm the global intern table so the measured runs do not include its
    // one-time lazy initialization and growth.
    let _ = measure_parse(FIXTURE);
    let first = measure_parse(FIXTURE);
    let second = measure_parse(FIXTURE);

    println!(
        "TASK-103 parse allocation baseline: {} allocations, {} bytes",
        first.allocations, first.bytes_allocated
    );

    assert_eq!(
        first, second,
        "parsing the same source must allocate deterministically"
    );
    assert!(first.allocations > 0, "parser should allocate");
    assert!(
        first.bytes_allocated >= FIXTURE.len(),
        "parser copies source and AST"
    );

    // Regression guardrail: later tasks must not exceed this baseline.
    // `stats_alloc` reports 222 allocations / 32_477 bytes for the current
    // AST. The earlier custom counter reported 271 / 50_387 because it also
    // counted reallocations as allocations. Both bounds leave a small margin.
    assert!(
        first.allocations <= 230,
        "parse allocations regressed: {} > 230",
        first.allocations
    );
    assert!(
        first.bytes_allocated <= 34_000,
        "parse allocated bytes regressed: {} > 34000",
        first.bytes_allocated
    );
}
