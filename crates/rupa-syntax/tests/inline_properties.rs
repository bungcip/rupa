//! Inline drawing-primitive properties (specification §7).

mod common;

use common::{error_messages, parse};
use rupa_syntax::ast::DocumentChild;

#[test]
fn properties_may_share_the_primitive_line() {
    let source = "size 320px x 200px\nbackground white\n\nrectangle at (10px, 40px) size 80px x 40px fill red\ncircle center at (150px, 50px) radius 30px fill blue\n";
    let (document, diagnostics) = parse(source);
    let document = document.expect("document");

    assert!(
        !diagnostics.has_errors(),
        "unexpected diagnostics: {:?}",
        error_messages(&diagnostics)
    );
    assert_eq!(document.children.len(), 2);
    assert!(matches!(document.children[0], DocumentChild::Rectangle(_)));
    assert!(matches!(document.children[1], DocumentChild::Circle(_)));
}

#[test]
fn inline_primitive_does_not_consume_the_next_line() {
    let source = "size 100px x 100px\nrectangle at (0px, 0px) size 10px x 10px fill red\nrectangle at (20px, 20px) size 5px x 5px fill blue\n";
    let (document, diagnostics) = parse(source);
    let document = document.expect("document");

    assert!(
        !diagnostics.has_errors(),
        "{:?}",
        error_messages(&diagnostics)
    );
    assert_eq!(document.children.len(), 2);
}

#[test]
fn an_indented_block_continues_an_inline_primitive() {
    let source = "size 100px x 100px\nrectangle at (0px, 0px) size 10px x 10px\n    fill red\n    opacity 0.5\n";
    let (document, diagnostics) = parse(source);
    let document = document.expect("document");

    assert!(
        !diagnostics.has_errors(),
        "{:?}",
        error_messages(&diagnostics)
    );
    assert_eq!(document.children.len(), 1);
}

#[test]
fn a_primitive_without_properties_is_a_syntax_error() {
    let source = "size 100px x 100px\nrectangle at (0px, 0px)\n";
    let (_document, diagnostics) = parse(source);

    let messages = error_messages(&diagnostics);
    assert!(
        messages
            .iter()
            .any(|message| message.contains("inline properties")),
        "expected an inline-properties hint, got {messages:?}"
    );
}

#[test]
fn inline_properties_work_inside_layers_and_repeats() {
    let source = "size 100px x 100px\n\nlayer \"Sky\"\n    fill white\n    circle center at (50px, 50px) radius 20px fill blue\n\nrepeat 2 times\n    rectangle at (0px, 0px) size 10px x 10px fill red\n";
    let (document, diagnostics) = parse(source);
    let document = document.expect("document");

    assert!(
        !diagnostics.has_errors(),
        "{:?}",
        error_messages(&diagnostics)
    );
    assert_eq!(document.layers.len(), 1);
    assert_eq!(document.layers[0].children.len(), 1);
    // The repeat is the only top-level child; the layer lives in `layers`.
    assert_eq!(document.children.len(), 1);
}

#[test]
fn a_placement_without_properties_is_still_valid() {
    let source = "size 100px x 100px\ngroup \"Dot\"\n    size 10px x 10px\n    rectangle at (0px, 0px) size 10px x 10px fill red\n\nplace \"Dot\" at (0px, 0px)\n";
    let (_document, diagnostics) = parse(source);

    assert!(
        !diagnostics.has_errors(),
        "{:?}",
        error_messages(&diagnostics)
    );
}

#[test]
fn block_valued_properties_still_require_an_indented_block() {
    let source = "size 100px x 100px\nrectangle at (0px, 0px) size 10px x 10px fill gradient from red to blue\n";
    let (_document, diagnostics) = parse(source);

    assert!(
        diagnostics.has_errors(),
        "a gradient inline must be rejected until gradients can be written inline"
    );
}
