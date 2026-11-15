//! Validation orchestration for Rupa compiler.
//!
//! [`Validator::validate`] validates an already-parsed [`Document`].
//! [`Validator::validate_source`] runs the full front end (lexer, parser, and
//! semantic validation) and additionally enforces that exactly one `document`
//! declaration is present.

use crate::document::DocumentValidator;
use crate::group::GroupValidator;
use crate::layer::LayerValidator;
use crate::paint::PaintValidator;
use crate::shapes::ShapeValidator;
use rupa_syntax::ast::{Document, DocumentChild, Fill, Function, Stroke, Unit, Value};
use rupa_syntax::diagnostic::{Diagnostic, DiagnosticBag, codes, did_you_mean_from};
use rupa_syntax::span::{FileId, Span};
use rupa_syntax::{IStr, Prng};
use std::collections::{HashMap, HashSet};

/// Main validation entry point.
pub struct Validator;

impl Validator {
    /// Validate an already-parsed document.
    pub fn validate(document: &Document) -> DiagnosticBag {
        let mut bag = DiagnosticBag::new();

        // Document-level validation.
        bag.append(DocumentValidator::validate(document));

        // Background paint validation (optional; omission means transparent).
        if let Some(background) = &document.background
            && let Err(diagnostics) =
                PaintValidator::lower_color(&background.color, background.span)
        {
            bag.append(diagnostics);
        }

        let mut rng = Prng::new(document.seed.unwrap_or(Prng::DEFAULT_SEED));
        let mut functions = HashMap::new();
        for function in &document.functions {
            if functions.insert(function.name, function.clone()).is_some() {
                bag.error_with_help(
                    codes::DUPLICATE_VARIABLE_NAME,
                    format!("duplicate function declaration `{}`", function.name),
                    function.span,
                    "choose a different function name",
                );
            }
            let mut seen_params = HashSet::new();
            for param in &function.params {
                if !seen_params.insert(*param) {
                    bag.error_with_help(
                        codes::DUPLICATE_VARIABLE_NAME,
                        format!(
                            "duplicate parameter `{param}` in function `{}`",
                            function.name
                        ),
                        function.span,
                        "choose a different parameter name",
                    );
                }
            }
        }
        let mut scope = HashMap::new();
        validate_block(
            &document.children,
            &mut bag,
            &mut scope,
            &mut rng,
            &functions,
        );

        bag.append(LayerValidator::validate_layers(&document.layers));
        bag.append(GroupValidator::validate_groups(&document.groups));
        bag.append(GroupValidator::validate_placements(document));
        for group in &document.groups {
            let mut scope = HashMap::new();
            validate_block(&group.children, &mut bag, &mut scope, &mut rng, &functions);
        }
        for layer in &document.layers {
            let mut scope = HashMap::new();
            validate_block(&layer.children, &mut bag, &mut scope, &mut rng, &functions);
            // Layer background `fill` shorthand is paint, validated like a
            // shape fill (specification §6.4).
            validate_paint_fields(&layer.fill, &None, &mut bag);
        }

        bag
    }

    fn validate_child(child: &DocumentChild, bag: &mut DiagnosticBag) {
        match child {
            DocumentChild::Rectangle(rect) => {
                bag.append(ShapeValidator::validate_rectangle(rect));
                validate_paint_fields(&rect.fill, &rect.stroke, bag);
            }
            DocumentChild::Circle(circle) => {
                bag.append(ShapeValidator::validate_circle(circle));
                validate_paint_fields(&circle.fill, &circle.stroke, bag);
            }
            DocumentChild::Ellipse(ellipse) => {
                bag.append(ShapeValidator::validate_ellipse(ellipse));
                validate_paint_fields(&ellipse.fill, &ellipse.stroke, bag);
            }
            DocumentChild::Square(square) => {
                bag.append(ShapeValidator::validate_square(square));
                validate_paint_fields(&square.fill, &square.stroke, bag);
            }
            DocumentChild::Triangle(triangle) => {
                bag.append(ShapeValidator::validate_triangle(triangle));
                validate_paint_fields(&triangle.fill, &triangle.stroke, bag);
            }
            DocumentChild::Polygon(polygon) => {
                bag.append(ShapeValidator::validate_polygon(polygon));
                validate_paint_fields(&polygon.fill, &polygon.stroke, bag);
            }
            DocumentChild::Line(line) => {
                bag.append(ShapeValidator::validate_line(line));
                validate_stroke_color(&line.stroke, bag);
            }
            DocumentChild::Polyline(polyline) => {
                bag.append(ShapeValidator::validate_polyline(polyline));
                validate_stroke_color(&polyline.stroke, bag);
            }
            DocumentChild::Path(path) => {
                bag.append(ShapeValidator::validate_path(path));
                validate_paint_fields(&path.fill, &path.stroke, bag);
            }
            DocumentChild::Picture(picture) => {
                bag.append(ShapeValidator::validate_picture(picture));
            }
            DocumentChild::Text(text) => {
                bag.append(ShapeValidator::validate_text(text));
                validate_fill_color(&text.fill, bag);
            }
            DocumentChild::Place(_) => {
                // Instance validation needs the complete definition set and
                // is handled once by `validate_instances`.
            }
            DocumentChild::Clear(clear) => {
                if let Err(diagnostic) = PaintValidator::parse_color(&clear.color, clear.span) {
                    bag.push(diagnostic);
                }
            }
            DocumentChild::Pixel(pixel) => {
                if let Some(position) = &pixel.position {
                    for (dimension, axis) in [(&position.x, "x"), (&position.y, "y")] {
                        if dimension.unit != Unit::Px {
                            bag.error_with_help(
                                codes::NON_PIXEL_SHAPE_DIMENSION,
                                format!("pixel position {axis} coordinate must use pixel units"),
                                dimension.span,
                                "use a pixel value such as '10px'",
                            );
                        }
                    }
                } else {
                    bag.error_with_help(
                        codes::MISSING_POSITION,
                        "pixel is missing required 'at (x, y)' position",
                        pixel.span,
                        "add 'at (x, y)' after 'pixel'",
                    );
                }
                match &pixel.fill {
                    Some(fill) => {
                        if let Err(diagnostics) = PaintValidator::lower_fill(fill) {
                            bag.append(diagnostics);
                        }
                    }
                    None => bag.error_with_help(
                        codes::MISSING_FILL,
                        "pixel is missing required 'fill'",
                        pixel.span,
                        "add 'fill <color>'",
                    ),
                }
            }
            DocumentChild::Let(_) => {
                // Handled by `validate_block`.
            }
            DocumentChild::Repeat(_) => {
                // Handled by `validate_block`.
            }
            DocumentChild::Brush(brush) => {
                if let Some(size) = &brush.size {
                    if size.unit != Unit::Px {
                        bag.error_with_help(
                            codes::NON_PIXEL_SHAPE_DIMENSION,
                            "brush size must use pixel units",
                            size.span,
                            "use a pixel value such as '12px'",
                        );
                    }
                    if size.value <= 0.0 {
                        bag.error(
                            codes::INVALID_DIMENSIONS,
                            "brush size must be positive",
                            size.span,
                        );
                    }
                } else {
                    bag.error_with_help(
                        codes::MISSING_SIZE,
                        "brush is missing required 'size'",
                        brush.span,
                        "add 'size <n>px'",
                    );
                }
                match &brush.fill {
                    Some(fill) => {
                        if let Err(diagnostics) = PaintValidator::lower_fill(fill) {
                            bag.append(diagnostics);
                        }
                    }
                    None => bag.error_with_help(
                        codes::MISSING_FILL,
                        "brush is missing required 'fill'",
                        brush.span,
                        "add 'fill <color>'",
                    ),
                }
                if brush.points.is_empty() {
                    bag.error_with_help(
                        codes::MISSING_POINTS,
                        "brush is missing stroke points",
                        brush.span,
                        "add a 'stroke' block with points",
                    );
                }
                for point in &brush.points {
                    for dimension in [&point.x, &point.y] {
                        if dimension.unit != Unit::Px {
                            bag.error_with_help(
                                codes::NON_PIXEL_SHAPE_DIMENSION,
                                "brush point coordinates must use pixel units",
                                dimension.span,
                                "use a pixel value such as '10px'",
                            );
                        }
                    }
                }
            }
        }
    }

    /// Validate Rupa source: tokenize, parse, then run semantic validation.
    ///
    /// One `.rupa` file is exactly one document. `size` and `background` are
    /// optional (defaults: 256x256 and transparent).
    pub fn validate_source(source: impl Into<String>, file_id: FileId) -> DiagnosticBag {
        let parsed = crate::compile::parse_source(source, file_id);
        let mut bag = parsed.diagnostics;
        if let Some(document) = &parsed.document {
            bag.append(Self::validate(document));
        }
        bag
    }
}

/// Build a diagnostic for an expression evaluation failure, suggesting a close
/// variable name when the failure is an unknown-variable reference.
fn expression_diagnostic(message: String, span: Span, scope: &HashMap<IStr, Value>) -> Diagnostic {
    let unknown = message.starts_with("unknown variable");
    let suggestion = if unknown {
        message
            .strip_prefix("unknown variable `")
            .and_then(|rest| rest.strip_suffix('`'))
            .and_then(|name| {
                let mut candidates: Vec<&str> = scope.keys().map(IStr::as_str).collect();
                candidates.sort_unstable();
                did_you_mean_from(name, &candidates)
            })
    } else {
        None
    };
    let code = if unknown {
        codes::UNKNOWN_VARIABLE
    } else {
        codes::INVALID_EXPRESSION
    };
    let mut diagnostic = Diagnostic::error(code, message, span);
    if let Some(suggestion) = suggestion {
        diagnostic = diagnostic.with_help(format!("did you mean `{suggestion}`?"));
    }
    diagnostic
}

fn validate_block(
    children: &[DocumentChild],
    bag: &mut DiagnosticBag,
    scope: &mut HashMap<IStr, Value>,
    rng: &mut Prng,
    functions: &HashMap<IStr, Function>,
) {
    let mut seen = HashSet::new();
    for child in children {
        if let DocumentChild::Let(variable) = child {
            if !seen.insert(variable.name) {
                bag.error_with_help(
                    codes::DUPLICATE_VARIABLE_NAME,
                    format!("duplicate variable declaration `{}`", variable.name),
                    variable.span,
                    "choose a different variable name",
                );
                continue;
            }
            match variable.value.evaluate(scope, functions, rng) {
                Ok(value) => {
                    scope.insert(variable.name, value);
                }
                Err(message) => {
                    bag.push(expression_diagnostic(message, variable.span, scope));
                }
            }
        } else if let DocumentChild::Repeat(repeat) = child {
            let count_valid = match repeat.count.evaluate(scope, functions, rng) {
                Ok(Value::Number(n)) if n >= 0.0 && n.fract() == 0.0 => true,
                Ok(Value::Dimension(d))
                    if d.unit == Unit::Px && d.value >= 0.0 && d.value.fract() == 0.0 =>
                {
                    true
                }
                Ok(_) => false,
                Err(message) => {
                    bag.push(expression_diagnostic(message, repeat.span, scope));
                    false
                }
            };
            if !count_valid {
                bag.error(
                    codes::INVALID_REPEAT_COUNT,
                    "repeat count must be a non-negative integer",
                    repeat.span,
                );
            }
            let mut body_scope = scope.clone();
            if let Some(index) = &repeat.index {
                body_scope.insert(*index, Value::Number(0.0));
            }
            validate_block(&repeat.body, bag, &mut body_scope, rng, functions);
        } else {
            Validator::validate_child(child, bag);
        }
    }
}

fn validate_fill_color(fill: &Option<Fill>, bag: &mut DiagnosticBag) {
    if let Some(fill) = fill
        && let Err(diagnostics) = PaintValidator::lower_fill(fill)
    {
        bag.append(diagnostics);
    }
}

fn validate_paint_fields(fill: &Option<Fill>, stroke: &Option<Stroke>, bag: &mut DiagnosticBag) {
    validate_fill_color(fill, bag);
    validate_stroke_color(stroke, bag);
}

fn validate_stroke_color(stroke: &Option<Stroke>, bag: &mut DiagnosticBag) {
    if let Some(stroke) = stroke
        && let Some(color) = &stroke.color
        && let Err(diagnostics) = PaintValidator::lower_color(color, stroke.span)
    {
        bag.append(diagnostics);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::codes_of;
    use rupa_syntax::diagnostic::codes;

    fn validate(source: &str) -> DiagnosticBag {
        Validator::validate_source(source, FileId(0))
    }

    const VALID: &str = r#"size 200px x 100px
background white

"#;

    #[test]
    fn valid_source_passes() {
        let bag = validate(VALID);
        assert!(!bag.has_errors(), "diagnostics: {:?}", bag.as_slice());
    }

    #[test]
    fn empty_source_is_valid() {
        let bag = validate("");
        assert!(!bag.has_errors(), "diagnostics: {:?}", bag.as_slice());
        assert!(bag.as_slice().is_empty());
    }

    #[test]
    fn legacy_document_wrapper_is_rejected_as_unknown_keyword() {
        let source = r#"document "Old"
    size 100px x 100px
"#;
        let bag = validate(source);
        assert!(
            bag.as_slice()
                .iter()
                .any(|diagnostic| diagnostic.code == codes::UNKNOWN_KEYWORD),
            "diagnostics: {:?}",
            bag.as_slice()
        );
    }

    #[test]
    fn duplicate_document_settings_are_reported() {
        let source = r#"size 100px x 100px
size 200px x 200px
background white
background black
seed 1
seed 2
"#;
        let bag = validate(source);
        assert_eq!(
            codes_of(&bag),
            vec![
                codes::DUPLICATE_DOCUMENT_SIZE,
                codes::DUPLICATE_BACKGROUND,
                codes::DUPLICATE_SEED,
            ]
        );
    }

    #[test]
    fn omitted_size_and_background_use_defaults() {
        for source in [
            "background white\n",
            "circle center at (10px, 10px)\n    radius 4px\n    fill red\n",
        ] {
            let bag = validate(source);
            assert!(!bag.has_errors(), "diagnostics: {:?}", bag.as_slice());
        }
    }

    #[test]
    fn zero_dimensions_are_c003() {
        let source = r#"size 0px x 100px
"#;
        let bag = validate(source);
        assert_eq!(codes_of(&bag), vec![codes::INVALID_DIMENSIONS]);
    }

    #[test]
    fn legacy_export_is_rejected_as_unknown_keyword() {
        let source = r#"size 100px x 100px
export "out.png""#;
        let bag = validate(source);
        assert!(
            bag.as_slice()
                .iter()
                .any(|diagnostic| diagnostic.code == codes::UNKNOWN_KEYWORD),
            "diagnostics: {:?}",
            bag.as_slice()
        );
    }

    #[test]
    fn invalid_named_fill_color_is_t002() {
        let source = r#"size 100px x 100px
rectangle at (0px, 0px)
    size 10px x 10px
    fill gren
"#;
        let bag = validate(source);
        assert_eq!(codes_of(&bag), vec![codes::UNKNOWN_NAMED_COLOR]);
        assert!(bag.as_slice()[0].help.as_deref().unwrap().contains("green"));
    }

    #[test]
    fn invalid_fill_color_is_reported_for_every_painted_primitive() {
        let bodies = [
            "rectangle at (0px, 0px)\n    size 10px x 10px\n    fill gren",
            "circle center at (10px, 10px)\n    radius 5px\n    fill gren",
            "ellipse center at (10px, 10px)\n    size 10px x 5px\n    fill gren",
            "square at (0px, 0px)\n    size 10px\n    fill gren",
            "triangle at (0px, 0px)\n    size 10px x 10px\n    fill gren",
            "polygon\n    points\n        (0px, 0px)\n        (10px, 0px)\n        (5px, 10px)\n    fill gren",
            "path\n    start at (0px, 0px)\n    line to (10px, 0px)\n    fill gren",
            "text \"Hi\"\n    at (0px, 0px)\n    fill gren",
        ];
        for body in bodies {
            let source = format!("size 100px x 100px\n{body}\n");
            let bag = validate(&source);
            assert!(
                codes_of(&bag).contains(&codes::UNKNOWN_NAMED_COLOR),
                "missing fill diagnostic for:\n{body}\n{:?}",
                bag.as_slice()
            );
        }
    }

    #[test]
    fn invalid_stroke_color_is_reported_for_every_stroked_primitive() {
        let bodies = [
            "rectangle at (0px, 0px)\n    size 10px x 10px\n    stroke gren",
            "circle center at (10px, 10px)\n    radius 5px\n    stroke gren",
            "ellipse center at (10px, 10px)\n    size 10px x 5px\n    stroke gren",
            "square at (0px, 0px)\n    size 10px\n    stroke gren",
            "triangle at (0px, 0px)\n    size 10px x 10px\n    stroke gren",
            "polygon\n    points\n        (0px, 0px)\n        (10px, 0px)\n        (5px, 10px)\n    stroke gren",
            "line from (0px, 0px) to (10px, 0px)\n    stroke gren",
            "polyline\n    points\n        (0px, 0px)\n        (10px, 0px)\n    stroke gren",
            "path\n    start at (0px, 0px)\n    line to (10px, 0px)\n    stroke gren",
        ];
        for body in bodies {
            let source = format!("size 100px x 100px\n{body}\n");
            let bag = validate(&source);
            assert!(
                codes_of(&bag).contains(&codes::UNKNOWN_NAMED_COLOR),
                "missing stroke diagnostic for:\n{body}\n{:?}",
                bag.as_slice()
            );
        }
    }

    #[test]
    fn invalid_hex_fill_color_is_t001() {
        let source = r##"size 100px x 100px
rectangle at (0px, 0px)
    size 10px x 10px
    fill "#XYZ"
"##;
        let bag = validate(source);
        assert_eq!(codes_of(&bag), vec![codes::INVALID_HEX_COLOR_FORMAT]);
    }

    #[test]
    fn fill_none_is_valid_and_produces_no_paint() {
        let source = r#"size 100px x 100px
rectangle at (0px, 0px)
    size 10px x 10px
    fill none
"#;
        let bag = validate(source);
        assert!(!bag.has_errors(), "diagnostics: {:?}", bag.as_slice());
    }

    #[test]
    fn invalid_background_color_is_reported() {
        let source = r##"size 100px x 100px
background "#12345"
"##;
        let bag = validate(source);
        assert_eq!(codes_of(&bag), vec![codes::INVALID_HEX_COLOR_FORMAT]);
    }

    #[test]
    fn unknown_variable_suggests_close_name_in_scope() {
        let source = r#"size 100px x 100px
let primary = 10px
let second = primry + 1px
"#;
        let bag = validate(source);
        let diagnostic = bag
            .as_slice()
            .iter()
            .find(|d| d.code == codes::UNKNOWN_VARIABLE)
            .expect("unknown variable diagnostic");
        assert!(
            diagnostic
                .help
                .as_deref()
                .unwrap_or_default()
                .contains("did you mean `primary`?")
        );
    }

    #[test]
    fn lexer_errors_are_preserved() {
        let source = "\tsize 100px x 100px\n";
        let bag = validate(source);
        assert!(
            bag.as_slice()
                .iter()
                .any(|d| d.code == codes::TAB_CHARACTER)
        );
    }
}
