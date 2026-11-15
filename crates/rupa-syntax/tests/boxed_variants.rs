//! Boxed AST variant construction and matching (TASK-101).

mod common;

use common::parse;
use rupa_syntax::ast::{Document, DocumentChild, Expression, Fill};

fn fill_of(document: &Document) -> &Fill {
    document
        .children
        .iter()
        .find_map(|child| match child {
            DocumentChild::Rectangle(rectangle) => rectangle.fill.as_ref(),
            _ => None,
        })
        .expect("rectangle fill")
}

#[test]
fn boxed_gradient_fill_parses_and_serializes_transparently() {
    let source = "rectangle at (0px, 0px)\n    size 100px x 50px\n    fill gradient\n        from red\n        to blue\n        angle 90deg\n";
    let (document, diagnostics) = parse(source);
    assert!(!diagnostics.has_errors());
    let document = document.expect("document");
    let fill = fill_of(&document);

    let gradient = fill.gradient.as_deref().expect("gradient payload");
    assert_eq!(gradient.stops.len(), 0);
    assert!(gradient.from.is_some());
    assert!(gradient.to.is_some());

    let value = serde_json::to_value(fill).expect("serialize fill");
    assert_eq!(value["gradient"]["angle"]["value"], 90.0);
    assert!(value["pattern"].is_null());
}

#[test]
fn boxed_pattern_fill_parses_and_serializes_transparently() {
    let source = "rectangle at (0px, 0px)\n    size 100px x 50px\n    fill pattern dots\n        color red\n        size 12px\n";
    let (document, diagnostics) = parse(source);
    assert!(!diagnostics.has_errors());
    let document = document.expect("document");
    let fill = fill_of(&document);

    let pattern = fill.pattern.as_deref().expect("pattern payload");
    assert_eq!(pattern.kind, "dots");
    assert!(pattern.color.is_some());

    let value = serde_json::to_value(fill).expect("serialize fill");
    assert_eq!(value["pattern"]["kind"], "dots");
    assert!(value["gradient"].is_null());
}

#[test]
fn boxed_position_expression_matches_and_evaluates() {
    let source = "let corner = (10px, 20px)\n";
    let (document, diagnostics) = parse(source);
    assert!(!diagnostics.has_errors());
    let document = document.expect("document");

    let variable = document
        .children
        .iter()
        .find_map(|child| match child {
            DocumentChild::Let(variable) => Some(variable),
            _ => None,
        })
        .expect("let binding");
    let Expression::Position(position) = &variable.value else {
        panic!("expected a boxed position expression");
    };
    assert_eq!(position.x.value, 10.0);
    assert_eq!(position.y.value, 20.0);
}
