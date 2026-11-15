//! The checked-in language reference must match the compiler's metadata.
//!
//! `rupa metadata` emits the canonical keyword, colour, unit, and primitive
//! tables. The playground reads the same JSON through `rupa_metadata()`, so this
//! test is what keeps the human-readable reference from drifting as well.

mod common;

use common::workspace_root;
use serde_json::Value;

#[test]
fn language_reference_matches_compiler_metadata() {
    let root = workspace_root();
    let source = std::fs::read_to_string(root.join("docs/language-reference.json"))
        .expect("read docs/language-reference.json");
    let reference: Value = serde_json::from_str(&source).expect("parse language reference");
    let metadata = rupa_compiler::metadata_json();

    for key in [
        "keyword_classes",
        "keywords",
        "named_colors",
        "units",
        "primitives",
    ] {
        assert_eq!(
            reference[key], metadata[key],
            "docs/language-reference.json `{key}` does not match `rupa metadata`; run `rupa metadata --reference docs/language-reference.json` after updating the source tables"
        );
    }
}
