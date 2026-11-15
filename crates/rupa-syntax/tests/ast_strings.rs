//! AST string representation regression tests (TASK-100).

mod common;

use common::{error_messages, parse};
use rupa_syntax::ast::{Document, DocumentChild, Text};

fn text_child(document: &Document) -> &Text {
    document
        .children
        .iter()
        .find_map(|child| match child {
            DocumentChild::Text(text) => Some(text),
            _ => None,
        })
        .expect("text child")
}

#[test]
fn text_stores_its_content_once() {
    let (document, diagnostics) = parse("text \"Hello\"\n    at (0px, 0px)\n");
    assert!(
        !diagnostics.has_errors(),
        "{:?}",
        error_messages(&diagnostics)
    );
    let document = document.expect("document");
    let text = text_child(&document);

    assert_eq!(text.text.as_ref(), "Hello");

    let value = serde_json::to_value(text).expect("serialize text");
    let object = value.as_object().expect("text object");
    assert_eq!(object.get("text"), Some(&serde_json::json!("Hello")));
    assert!(
        !object.contains_key("content"),
        "Text must not duplicate its content field: {value}"
    );

    let round_trip: Text = serde_json::from_value(value).expect("deserialize text");
    assert_eq!(&round_trip, text);
}

#[test]
fn interned_names_serialize_as_plain_strings() {
    let source = "group \"Card\"\n    rectangle at (0px, 0px) size 10px x 10px fill red\n\nlayer \"Sky\"\n    fill blue\n";
    let (document, diagnostics) = parse(source);
    assert!(!diagnostics.has_errors());
    let document = document.expect("document");

    let value = serde_json::to_value(&document).expect("serialize document");
    assert_eq!(value["groups"][0]["name"], "Card");
    assert_eq!(value["layers"][0]["name"], "Sky");
    assert_eq!(value["layers"][0]["fill"]["color"]["Named"], "blue");
}
