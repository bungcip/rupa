//! Canonical formatting of inline property lists and comment preservation.

use rupa_syntax::format_source;

fn format(source: &str) -> String {
    let formatted = format_source(source).expect("format");
    assert_eq!(
        format_source(&formatted).expect("reformat"),
        formatted,
        "formatting must be idempotent"
    );
    formatted
}

#[test]
fn expands_inline_properties_to_one_per_line() {
    let formatted =
        format("size 200px x 100px\nrectangle at (10px, 10px) size 80px x 40px fill red\n");
    assert!(
        formatted.contains("rectangle at (10px, 10px)\n    size 80px x 40px\n    fill red\n"),
        "unexpected output:\n{formatted}"
    );
}

#[test]
fn keeps_circle_center_on_the_primitive_line() {
    let formatted =
        format("size 200px x 100px\ncircle center at (150px, 50px) radius 30px fill blue\n");
    assert!(
        formatted.contains("circle center at (150px, 50px)\n    radius 30px\n    fill blue\n"),
        "unexpected output:\n{formatted}"
    );
}

#[test]
fn expands_multiple_inline_primitive_lines() {
    let source = "size 320px x 200px\nbackground white\nrectangle at (10px, 40px) size 80px x 40px fill red\ncircle center at (150px, 50px) radius 30px fill blue\n";
    let formatted = format(source);
    assert!(
        formatted.contains("\nrectangle at (10px, 40px)\n    size 80px x 40px\n    fill red\n")
    );
    assert!(
        formatted.contains("\ncircle center at (150px, 50px)\n    radius 30px\n    fill blue\n")
    );
}

#[test]
fn keeps_line_comments_and_idempotent_output() {
    let source = "size 200px x 100px\n\n// a red box\nrectangle at (10px, 10px) size 80px x 40px fill red // inline note\n";
    let formatted = format(source);

    assert!(
        formatted.contains("// a red box\n"),
        "comment-only line lost:\n{formatted}"
    );
    assert!(
        formatted.contains("// inline note\n"),
        "trailing comment lost:\n{formatted}"
    );
    assert!(
        formatted.contains("size 80px x 40px"),
        "properties lost:\n{formatted}"
    );
}

#[test]
fn preserves_comments_inside_indented_blocks() {
    let source = "size 100px x 100px\nlayer \"Sky\"\n    // background\n    fill white\n    circle center at (50px, 50px) radius 20px fill blue\n";
    let formatted = format(source);
    assert!(formatted.contains("    // background\n"), "{formatted}");
    assert!(formatted.contains("    fill white\n"), "{formatted}");
}
