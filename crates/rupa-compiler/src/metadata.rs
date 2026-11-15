//! Machine-readable language metadata for the CLI, language reference, and
//! browser playground.
//!
//! Keyword classes, units, and primitive properties come from
//! `rupa-syntax::metadata`; named colours come from [`crate::paint::NAMED_COLORS`].
//! This module only assembles those canonical tables into JSON so consumers do
//! not have to maintain their own copies.

use crate::paint::NAMED_COLORS;
use rupa_syntax::metadata::{KeywordClass, PRIMITIVES, UNITS, keywords_in_class};
use serde_json::{Map, Value, json};

/// Complete language metadata as a JSON value.
///
/// The shape is stable for tooling:
///
/// ```json
/// {
///   "keyword_classes": {
///     "keyword": ["..."],
///     "property": ["..."],
///     "literal": ["true", "false"]
///   },
///   "keywords": ["..."],
///   "named_colors": ["..."],
///   "units": ["px", "pt", "%", "deg"],
///   "primitives": {
///     "rectangle": { "properties": ["..."] }
///   }
/// }
/// ```
///
/// `keywords` is the sorted union of every class, kept for consumers that want
/// one flat list.
pub fn metadata_json() -> Value {
    let classes = [
        ("keyword", KeywordClass::Keyword),
        ("property", KeywordClass::Property),
        ("literal", KeywordClass::Literal),
    ];

    let mut keyword_classes = Map::new();
    let mut keywords = Vec::new();
    for (name, class) in classes {
        let words = keywords_in_class(class);
        keywords.extend(words.iter().copied());
        keyword_classes.insert(name.to_string(), json!(words));
    }
    keywords.sort_unstable();

    let named_colors: Vec<&str> = NAMED_COLORS.iter().map(|(name, _)| *name).collect();
    let primitives: Map<String, Value> = PRIMITIVES
        .iter()
        .map(|primitive| {
            (
                primitive.name.to_string(),
                json!({ "properties": primitive.properties }),
            )
        })
        .collect();

    json!({
        "keyword_classes": Value::Object(keyword_classes),
        "keywords": keywords,
        "named_colors": named_colors,
        "units": UNITS,
        "primitives": Value::Object(primitives),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_contains_every_documented_keyword_class() {
        let metadata = metadata_json();
        for class in ["keyword", "property", "literal"] {
            assert!(
                metadata["keyword_classes"][class]
                    .as_array()
                    .is_some_and(|words| !words.is_empty()),
                "missing keyword class {class}"
            );
        }
        assert_eq!(
            metadata["keyword_classes"]["literal"],
            json!(["false", "true"])
        );
    }

    #[test]
    fn metadata_uses_the_compiler_named_colors() {
        let metadata = metadata_json();
        let colors = metadata["named_colors"].as_array().expect("named colours");
        assert_eq!(colors.len(), NAMED_COLORS.len());
        for (name, _) in NAMED_COLORS {
            assert!(colors.iter().any(|color| color == name), "missing {name}");
        }
    }

    #[test]
    fn legacy_wrapper_words_are_absent() {
        let metadata = metadata_json();
        let keywords = metadata["keywords"].as_array().expect("keywords");
        assert!(!keywords.iter().any(|keyword| keyword == "document"));
        assert!(!keywords.iter().any(|keyword| keyword == "export"));
    }
}
