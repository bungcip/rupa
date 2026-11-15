//! AST memory guardrails (TASK-097…TASK-103, DECISION-042/043).
//!
//! These tests record the sizes of the source-location, token, and AST types
//! and guard against accidental growth. TASK-103 packs source locations into a
//! 24-bit offset plus an 8-bit file id, so `Span` is 8 bytes and all AST nodes
//! shrink with it.

use std::mem::size_of;
use std::sync::Arc;

mod common;

use common::{error_messages, parse};
use rupa_syntax::ast::{
    Brush, Circle, Clear, Document, DocumentChild, Ellipse, Expression, Fill, GradientFill,
    GroupDefinition, GroupInstance, LayerDefinition, Line, Path, PatternFill, Picture, Pixel,
    Polygon, Polyline, Position as AstPosition, Rectangle, Repeat, Square, Text, Transform,
    Triangle, Variable,
};
use rupa_syntax::{FileId, IStr, Position, Span, Token, TokenKind};

const FIXTURE: &str = include_str!("fixtures/baseline.rupa");

fn baseline() -> [(&'static str, usize, Option<usize>); 14] {
    [
        ("FileId", size_of::<FileId>(), Some(1)),
        ("Position", size_of::<Position>(), Some(4)),
        ("Span", size_of::<Span>(), Some(8)),
        ("Token", size_of::<Token>(), Some(32)),
        ("TokenKind", size_of::<TokenKind>(), None),
        ("IStr", size_of::<IStr>(), Some(8)),
        ("Document", size_of::<Document>(), Some(200)),
        ("DocumentChild", size_of::<DocumentChild>(), Some(304)),
        ("Text", size_of::<Text>(), Some(296)),
        ("Fill", size_of::<Fill>(), Some(40)),
        ("Expression", size_of::<Expression>(), Some(40)),
        ("GroupDefinition", size_of::<GroupDefinition>(), Some(96)),
        ("LayerDefinition", size_of::<LayerDefinition>(), Some(208)),
        ("Arc<str> payload handle", size_of::<Arc<str>>(), Some(16)),
    ]
}

#[test]
fn records_the_ast_size_baseline() {
    println!("TASK-103 final AST sizes (bytes, 64-bit target):");
    for (name, bytes, limit) in baseline() {
        println!("  {name:<24} {bytes:>4}");
        if let Some(limit) = limit {
            // Final layout guardrails from DECISION-043.
            assert!(
                bytes <= limit,
                "{name} grew past its packed target: {bytes} > {limit}"
            );
        }
    }
}

#[test]
fn records_enum_layout_variants() {
    println!("TASK-103 variant payload sizes (bytes):");
    let payloads: [(&str, usize); 16] = [
        ("Rectangle", size_of::<Rectangle>()),
        ("Circle", size_of::<Circle>()),
        ("Ellipse", size_of::<Ellipse>()),
        ("Square", size_of::<Square>()),
        ("Triangle", size_of::<Triangle>()),
        ("Polygon", size_of::<Polygon>()),
        ("Line", size_of::<Line>()),
        ("Polyline", size_of::<Polyline>()),
        ("Path", size_of::<Path>()),
        ("Picture", size_of::<Picture>()),
        ("Text", size_of::<Text>()),
        ("Place", size_of::<GroupInstance>()),
        ("Clear", size_of::<Clear>()),
        ("Pixel", size_of::<Pixel>()),
        ("Brush", size_of::<Brush>()),
        ("Repeat", size_of::<Repeat>()),
    ];
    for (name, bytes) in payloads {
        println!("  DocumentChild::{name:<12} {bytes:>4}");
    }
    println!("  DocumentChild::Let {:<5}", size_of::<Variable>());
    println!("Fill components:");
    for (name, bytes) in [
        ("Fill", size_of::<Fill>()),
        ("Option<Fill>", size_of::<Option<Fill>>()),
        ("GradientFill", size_of::<GradientFill>()),
        ("PatternFill", size_of::<PatternFill>()),
        ("Transform", size_of::<Transform>()),
        ("Expression", size_of::<Expression>()),
        ("rupa_syntax::ast::Position", size_of::<AstPosition>()),
    ] {
        println!("  {name:<28} {bytes:>4}");
    }
    assert!(size_of::<DocumentChild>() >= payloads.iter().map(|(_, bytes)| *bytes).max().unwrap());
}

#[test]
fn arc_str_handle_remains_a_sixteen_byte_alternative() {
    // DECISION-044 replaced `ustr` with the lean interner and removed the
    // `Arc<str>` fallback. Keep the alternative's handle size recorded for any
    // future backend comparison; `IStr` itself remains 8 bytes.
    assert!(
        size_of::<Arc<str>>() <= 16,
        "the Arc<str> alternative must stay one pointer plus a refcount"
    );
}

#[test]
fn baseline_fixture_parses_and_repeats_content() {
    let (document, diagnostics) = parse(FIXTURE);

    assert!(
        !diagnostics.has_errors(),
        "baseline fixture must parse cleanly: {:?}",
        error_messages(&diagnostics)
    );
    let document = document.expect("document");

    assert_eq!(document.groups.len(), 3);
    assert_eq!(document.layers.len(), 4);
    assert_eq!(document.groups[0].name, "Card");
    assert_eq!(document.groups[1].name, "Card");
    assert_eq!(document.layers[0].name, "Background");

    let repeated_text = document
        .layers
        .iter()
        .flat_map(|layer| &layer.children)
        .filter(|child| matches!(child, DocumentChild::Text(_)))
        .count();
    assert_eq!(
        repeated_text, 2,
        "fixture keeps repeated text payloads for dedup measurement"
    );
}
