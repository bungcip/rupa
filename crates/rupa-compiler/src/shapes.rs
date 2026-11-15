//! Semantic validation for basic shape primitives.
//!
//! See specification §7.1. Position conventions:
//!
//! - `rectangle`/`square`/`triangle` use `at (x, y)` for the top-left corner
//!   (triangle: bounding-box top-left).
//! - `circle`/`ellipse` use `center at (x, y)` for the center.
//!
//! Required properties are retained as `Option` in the AST so that missing
//! properties can be reported here with stable, specific diagnostic codes.
//! For the first implementation, all shape geometry uses pixel units.

use rupa_syntax::ast::{
    Circle, Dimension, Ellipse, Line, Path, PathCommand, Picture, Polygon, Polyline, Position,
    Rectangle, Size, Square, Stroke, Text, Transform, Triangle, Unit,
};
use rupa_syntax::diagnostic::{DiagnosticBag, codes};
use rupa_syntax::span::Span;

/// Validates rectangle and circle primitives.
pub struct ShapeValidator;

impl ShapeValidator {
    fn new_bag(transform: &Option<Transform>, shape: &str) -> DiagnosticBag {
        let mut bag = DiagnosticBag::new();
        bag.append(Self::validate_transform(transform, shape));
        bag
    }

    /// Validate a rectangle (`at` = top-left, positive `size`, optional fill/stroke).
    pub fn validate_rectangle(rect: &Rectangle) -> DiagnosticBag {
        let mut bag = Self::new_bag(&rect.transform, "rectangle");

        Self::validate_required_position(
            &rect.position,
            "rectangle",
            rect.span,
            "rectangle is missing required 'at (x, y)' position",
            "add 'at (x, y)' after 'rectangle'; the position is the top-left corner",
            &mut bag,
        );
        Self::validate_size_property(&rect.size, "rectangle", rect.span, &mut bag);

        Self::validate_optional_width(&rect.width, "rectangle", &mut bag);

        bag
    }

    /// Validate a circle (`center at` = center, positive `radius`, optional fill/stroke).
    pub fn validate_circle(circle: &Circle) -> DiagnosticBag {
        let mut bag = Self::new_bag(&circle.transform, "circle");

        Self::validate_required_position(
            &circle.center,
            "circle",
            circle.span,
            "circle is missing required 'center at (x, y)' position",
            "add 'center at (x, y)' after 'circle'; the position is the center",
            &mut bag,
        );

        match &circle.radius {
            Some(radius) => {
                Self::validate_dimension(radius, codes::NEGATIVE_RADIUS, "circle radius", &mut bag)
            }
            None => bag.error_with_help(
                codes::MISSING_RADIUS,
                "circle is missing required 'radius' property",
                circle.span,
                "add 'radius <r>px'",
            ),
        }

        Self::validate_optional_width(&circle.width, "circle", &mut bag);

        bag
    }

    /// Validate an ellipse (`center at` = center, full positive size, optional fill/stroke).
    pub fn validate_ellipse(ellipse: &Ellipse) -> DiagnosticBag {
        let mut bag = Self::new_bag(&ellipse.transform, "ellipse");

        Self::validate_required_position(
            &ellipse.center,
            "ellipse",
            ellipse.span,
            "ellipse is missing required 'center at (x, y)' position",
            "add 'center at (x, y)' after 'ellipse'; the position is the center",
            &mut bag,
        );

        Self::validate_size_property(&ellipse.size, "ellipse", ellipse.span, &mut bag);
        Self::validate_optional_width(&ellipse.width, "ellipse", &mut bag);

        bag
    }

    /// Validate a square (`at` = top-left, single positive side, optional fill/stroke).
    pub fn validate_square(square: &Square) -> DiagnosticBag {
        let mut bag = Self::new_bag(&square.transform, "square");

        Self::validate_required_position(
            &square.position,
            "square",
            square.span,
            "square is missing required 'at (x, y)' position",
            "add 'at (x, y)' after 'square'; the position is the top-left corner",
            &mut bag,
        );

        match &square.size {
            Some(size) => {
                Self::validate_dimension(size, codes::NEGATIVE_DIMENSION, "square size", &mut bag)
            }
            None => bag.error_with_help(
                codes::MISSING_SIZE,
                "square is missing required 'size' property",
                square.span,
                "add 'size <side>px'",
            ),
        }

        Self::validate_optional_width(&square.width, "square", &mut bag);

        bag
    }

    /// Validate a triangle (`at` = bounding-box top-left, positive size, optional fill/stroke).
    pub fn validate_triangle(triangle: &Triangle) -> DiagnosticBag {
        let mut bag = Self::new_bag(&triangle.transform, "triangle");

        Self::validate_required_position(
            &triangle.position,
            "triangle",
            triangle.span,
            "triangle is missing required 'at (x, y)' position",
            "add 'at (x, y)' after 'triangle'; the position is the bounding-box top-left corner",
            &mut bag,
        );

        Self::validate_size_property(&triangle.size, "triangle", triangle.span, &mut bag);
        Self::validate_optional_width(&triangle.width, "triangle", &mut bag);

        bag
    }

    /// Validate a polygon (nested `points` list, optional fill/stroke).
    pub fn validate_polygon(polygon: &Polygon) -> DiagnosticBag {
        let mut bag = Self::new_bag(&polygon.transform, "polygon");

        match &polygon.points {
            Some(points) => {
                for point in points {
                    Self::validate_position(point, "polygon", &mut bag);
                }
                if points.len() < 3 {
                    bag.error_with_help(
                        codes::TOO_FEW_POINTS,
                        format!(
                            "polygon requires at least three points, found {}",
                            points.len()
                        ),
                        polygon.span,
                        "add more '(x, y)' entries to the points block",
                    );
                }
            }
            None => bag.error_with_help(
                codes::MISSING_POINTS,
                "polygon is missing required 'points' block",
                polygon.span,
                "add an indented 'points' block containing '(x, y)' entries",
            ),
        }

        Self::validate_optional_width(&polygon.width, "polygon", &mut bag);

        bag
    }

    /// Validate a line (required endpoints, required stroke, optional width).
    pub fn validate_line(line: &Line) -> DiagnosticBag {
        let mut bag = Self::new_bag(&line.transform, "line");

        match &line.from {
            Some(from) => Self::validate_position(from, "line", &mut bag),
            None => bag.error_with_help(
                codes::MISSING_POSITION,
                "line is missing required 'from (x, y)' endpoint",
                line.span,
                "add 'from (x, y)' after 'line'",
            ),
        }
        match &line.to {
            Some(to) => Self::validate_position(to, "line", &mut bag),
            None => bag.error_with_help(
                codes::MISSING_POSITION,
                "line is missing required 'to (x, y)' endpoint",
                line.span,
                "add 'to (x, y)' after the start endpoint",
            ),
        }

        Self::validate_stroke(&line.stroke, &line.width, "line", line.span, &mut bag);

        bag
    }

    /// Validate an open polyline (at least two points, required stroke).
    pub fn validate_polyline(polyline: &Polyline) -> DiagnosticBag {
        let mut bag = Self::new_bag(&polyline.transform, "polyline");

        match &polyline.points {
            Some(points) => {
                for point in points {
                    Self::validate_position(point, "polyline", &mut bag);
                }
                if points.len() < 2 {
                    bag.error_with_help(
                        codes::TOO_FEW_POLYLINE_POINTS,
                        format!(
                            "polyline requires at least two points, found {}",
                            points.len()
                        ),
                        polyline.span,
                        "add more '(x, y)' entries to the points block",
                    );
                }
            }
            None => bag.error_with_help(
                codes::MISSING_POINTS,
                "polyline is missing required 'points' block",
                polyline.span,
                "add an indented 'points' block containing '(x, y)' entries",
            ),
        }

        Self::validate_stroke(
            &polyline.stroke,
            &polyline.width,
            "polyline",
            polyline.span,
            &mut bag,
        );

        bag
    }

    /// Validate a path command sequence and optional fill/stroke/width.
    pub fn validate_path(path: &Path) -> DiagnosticBag {
        let mut bag = Self::new_bag(&path.transform, "path");

        if !matches!(path.commands.first(), Some(PathCommand::Start { .. })) {
            bag.error_with_help(
                codes::MISSING_PATH_START,
                "path must begin with a 'start at (x, y)' command",
                path.span,
                "add 'start at (x, y)' as the first path command",
            );
        }

        for command in &path.commands {
            match command {
                PathCommand::Start { position, .. } => {
                    Self::validate_position(position, "path", &mut bag)
                }
                PathCommand::LineTo { to, .. } => Self::validate_position(to, "path", &mut bag),
                PathCommand::CurveTo {
                    to,
                    control1,
                    control2,
                    ..
                } => {
                    Self::validate_position(to, "path", &mut bag);
                    Self::validate_position(control1, "path", &mut bag);
                    Self::validate_position(control2, "path", &mut bag);
                }
                PathCommand::QuadTo { to, control, .. } => {
                    Self::validate_position(to, "path", &mut bag);
                    Self::validate_position(control, "path", &mut bag);
                }
                PathCommand::Close { .. } => {}
            }
        }

        if let Some(width) = &path.width {
            Self::validate_dimension(
                width,
                codes::INVALID_STROKE_WIDTH,
                "path stroke width",
                &mut bag,
            );
        }

        bag
    }

    /// Validate a picture (`at` = frame top-left, optional positive frame
    /// `size`, optional fit mode and transform).
    pub fn validate_picture(picture: &Picture) -> DiagnosticBag {
        let mut bag = Self::new_bag(&picture.transform, "picture");

        if picture.path.trim().is_empty() {
            bag.error_with_help(
                codes::EMPTY_RESOURCE_PATH,
                "picture resource path must not be empty",
                picture.span,
                "provide a path such as 'picture \"photo.png\"'",
            );
        }

        Self::validate_required_position(
            &picture.position,
            "picture",
            picture.span,
            "picture is missing required 'at (x, y)' position",
            "add 'at (x, y)'; the position is the frame's top-left corner",
            &mut bag,
        );

        if let Some(size) = &picture.size {
            Self::validate_size_axes(size, "picture", &mut bag);
        }

        bag
    }

    /// Validate a text primitive.
    pub fn validate_text(text: &Text) -> DiagnosticBag {
        let mut bag = Self::new_bag(&text.transform, "text");

        if text.text.trim().is_empty() {
            bag.error_with_help(
                codes::EMPTY_TEXT,
                "text content must not be empty",
                text.span,
                "provide text such as 'text \"Hello\"'",
            );
        }

        if text.position.is_none() && text.center.is_none() {
            bag.error_with_help(
                codes::MISSING_POSITION,
                "text is missing required 'at (x, y)' or 'center at (x, y)' position",
                text.span,
                "add 'at (x, y)' or 'center at (x, y)'",
            );
        }

        if let Some(position) = &text.position {
            Self::validate_position(position, "text", &mut bag);
        }
        if let Some(center) = &text.center {
            Self::validate_position(center, "text", &mut bag);
        }

        if let Some(size) = &text.size {
            if size.unit != Unit::Px && size.unit != Unit::Pt {
                bag.error_with_help(
                    codes::NON_PIXEL_SHAPE_DIMENSION,
                    "text size must use pixel or point units",
                    size.span,
                    "use a value such as '16px' or '12pt'",
                );
            }
            if !size.value.is_finite() || size.value <= 0.0 {
                bag.error_with_help(
                    codes::NEGATIVE_DIMENSION,
                    "text size must be a positive, finite number",
                    size.span,
                    "use a value greater than zero",
                );
            }
        }

        bag
    }

    /// Validate rotate/scale/opacity transform properties.
    pub fn validate_transform(transform: &Option<Transform>, shape: &str) -> DiagnosticBag {
        let mut bag = DiagnosticBag::new();
        let Some(transform) = transform else {
            return bag;
        };

        if let Some(rotate) = &transform.rotate {
            if rotate.unit != Unit::Deg {
                bag.error_with_help(
                    codes::INVALID_TRANSFORM,
                    format!("{shape} rotation must use degrees"),
                    rotate.span,
                    "use an angle such as '30deg'",
                );
            }
            if !rotate.value.is_finite() {
                bag.error_with_help(
                    codes::INVALID_TRANSFORM,
                    format!("{shape} rotation must be a finite angle"),
                    rotate.span,
                    "use a finite degree value",
                );
            }
        }

        if let Some(scale) = &transform.scale
            && (!scale.x.is_finite() || !scale.y.is_finite() || scale.x <= 0.0 || scale.y <= 0.0)
        {
            bag.error_with_help(
                codes::INVALID_TRANSFORM,
                format!("{shape} scale factors must be positive and finite"),
                scale.span,
                "use a scale such as '0.8' or '2.0 x 0.5'",
            );
        }

        if let Some(opacity) = transform.opacity
            && (!opacity.is_finite() || !(0.0..=1.0).contains(&opacity))
        {
            bag.error_with_help(
                codes::INVALID_TRANSFORM,
                format!("{shape} opacity must be between 0.0 and 1.0"),
                transform.span,
                "use a value such as '0.8'",
            );
        }

        bag
    }

    fn validate_optional_width(width: &Option<Dimension>, shape: &str, bag: &mut DiagnosticBag) {
        if let Some(width) = width {
            Self::validate_dimension(
                width,
                codes::INVALID_STROKE_WIDTH,
                &format!("{shape} stroke width"),
                bag,
            );
        }
    }

    fn validate_stroke(
        stroke: &Option<Stroke>,
        width: &Option<Dimension>,
        shape: &str,
        span: Span,
        bag: &mut DiagnosticBag,
    ) {
        if stroke.is_none() {
            bag.error_with_help(
                codes::MISSING_STROKE,
                format!("{shape} is missing required 'stroke' property"),
                span,
                "add 'stroke <color>'",
            );
        }
        Self::validate_optional_width(width, shape, bag);
    }

    fn validate_required_position(
        position: &Option<Position>,
        shape: &str,
        span: Span,
        missing_message: &str,
        help: &str,
        bag: &mut DiagnosticBag,
    ) {
        match position {
            Some(position) => Self::validate_position(position, shape, bag),
            None => bag.error_with_help(codes::MISSING_POSITION, missing_message, span, help),
        }
    }

    fn validate_size_property(
        size: &Option<Size>,
        shape: &str,
        span: Span,
        bag: &mut DiagnosticBag,
    ) {
        match size {
            Some(size) => Self::validate_size_axes(size, shape, bag),
            None => bag.error_with_help(
                codes::MISSING_SIZE,
                format!("{shape} is missing required 'size' property"),
                span,
                "add 'size <width>px x <height>px'",
            ),
        }
    }

    fn validate_size_axes(size: &Size, shape: &str, bag: &mut DiagnosticBag) {
        Self::validate_dimension(
            &size.width,
            codes::NEGATIVE_DIMENSION,
            &format!("{shape} width"),
            bag,
        );
        Self::validate_dimension(
            &size.height,
            codes::NEGATIVE_DIMENSION,
            &format!("{shape} height"),
            bag,
        );
    }

    /// Shared dimension rule: positive, finite, and expressed in pixels.
    fn validate_dimension(
        dimension: &Dimension,
        non_positive_code: &str,
        label: &str,
        bag: &mut DiagnosticBag,
    ) {
        if dimension.unit != Unit::Px {
            bag.error_with_help(
                codes::NON_PIXEL_SHAPE_DIMENSION,
                format!("{label} must use pixel units"),
                dimension.span,
                "use a pixel value such as '20px'",
            );
        }

        if !dimension.value.is_finite() || dimension.value <= 0.0 {
            bag.error_with_help(
                non_positive_code,
                format!("{label} must be a positive, finite number"),
                dimension.span,
                "use a value greater than zero",
            );
        }
    }

    /// Positions may be negative, but coordinates must be pixels in M1.
    fn validate_position(position: &Position, shape: &str, bag: &mut DiagnosticBag) {
        for (dimension, axis) in [(&position.x, "x"), (&position.y, "y")] {
            if dimension.unit != Unit::Px {
                bag.error_with_help(
                    codes::NON_PIXEL_SHAPE_DIMENSION,
                    format!("{shape} position {axis} coordinate must use pixel units"),
                    dimension.span,
                    "use a pixel value such as '10px'",
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::codes_of;
    use crate::validation::Validator;
    use rupa_syntax::span::FileId;

    fn validate_body(body: &str) -> DiagnosticBag {
        let source = format!("size 100px x 100px\n{body}\n");
        Validator::validate_source(source, FileId(0))
    }

    fn assert_valid(body: &str) {
        let bag = validate_body(body);
        assert!(!bag.has_errors(), "diagnostics: {:?}", bag.as_slice());
    }

    fn assert_invalid(body: &str, expected: &[&str]) {
        let bag = validate_body(body);
        assert_eq!(
            codes_of(&bag).as_slice(),
            expected,
            "diagnostics: {:?}",
            bag.as_slice()
        );
    }

    #[test]
    fn rectangle_missing_position_is_s003() {
        let bag = validate_body("rectangle\n    size 50px x 60px\n    fill red");
        assert_eq!(codes_of(&bag), vec![codes::MISSING_POSITION]);
        assert!(
            bag.as_slice()[0]
                .help
                .as_deref()
                .unwrap()
                .contains("top-left")
        );
    }

    #[test]
    fn rectangle_missing_size_is_s004() {
        assert_invalid(
            "rectangle at (10px, 20px)\n    fill red",
            &[codes::MISSING_SIZE],
        );
    }

    #[test]
    fn rectangle_negative_width_is_s001() {
        assert_invalid(
            "rectangle at (10px, 20px)\n    size -50px x 60px\n    fill red",
            &[codes::NEGATIVE_DIMENSION],
        );
    }

    #[test]
    fn rectangle_negative_height_is_s001() {
        assert_invalid(
            "rectangle at (10px, 20px)\n    size 50px x -60px\n    fill red",
            &[codes::NEGATIVE_DIMENSION],
        );
    }

    #[test]
    fn rectangle_zero_size_is_s001_for_each_axis() {
        assert_invalid(
            "rectangle at (10px, 20px)\n    size 0px x 0px\n    fill red",
            &[codes::NEGATIVE_DIMENSION, codes::NEGATIVE_DIMENSION],
        );
    }

    #[test]
    fn rectangle_non_pixel_size_is_s007() {
        assert_invalid(
            "rectangle at (10px, 20px)\n    size 50% x 60px\n    fill red",
            &[codes::NON_PIXEL_SHAPE_DIMENSION],
        );
    }

    #[test]
    fn rectangle_non_pixel_position_is_s007() {
        assert_invalid(
            "rectangle at (10%, 20px)\n    size 50px x 60px\n    fill red",
            &[codes::NON_PIXEL_SHAPE_DIMENSION],
        );
    }

    #[test]
    fn rectangle_missing_everything_reports_all_errors() {
        assert_invalid(
            "rectangle\n    size 50px x 60px",
            &[codes::MISSING_POSITION],
        );
    }

    #[test]
    fn circle_missing_center_is_s003() {
        let bag = validate_body("circle\n    radius 25px\n    fill blue");
        assert_eq!(codes_of(&bag), vec![codes::MISSING_POSITION]);
        assert!(
            bag.as_slice()[0]
                .help
                .as_deref()
                .unwrap()
                .contains("center")
        );
    }

    #[test]
    fn circle_missing_radius_is_s005() {
        assert_invalid(
            "circle center at (50px, 50px)\n    fill blue",
            &[codes::MISSING_RADIUS],
        );
    }

    #[test]
    fn circle_negative_radius_is_s002() {
        assert_invalid(
            "circle center at (50px, 50px)\n    radius -25px\n    fill blue",
            &[codes::NEGATIVE_RADIUS],
        );
    }

    #[test]
    fn circle_zero_radius_is_s002() {
        assert_invalid(
            "circle center at (50px, 50px)\n    radius 0px\n    fill blue",
            &[codes::NEGATIVE_RADIUS],
        );
    }

    #[test]
    fn circle_non_pixel_radius_is_s007() {
        assert_invalid(
            "circle center at (50px, 50px)\n    radius 25deg\n    fill blue",
            &[codes::NON_PIXEL_SHAPE_DIMENSION],
        );
    }

    #[test]
    fn circle_position_semantics_are_center() {
        // Negative coordinates are allowed; only dimensions/radii must be positive.
        assert_valid("circle center at (-10px, -20px)\n    radius 25px\n    fill blue");
    }

    #[test]
    fn rectangle_position_semantics_are_top_left() {
        // Negative coordinates are allowed for the top-left corner.
        assert_valid("rectangle at (-10px, -20px)\n    size 50px x 60px\n    fill red");
    }
    #[test]
    fn ellipse_missing_center_is_s003() {
        assert_invalid(
            "ellipse\n    size 80px x 40px\n    fill orange",
            &[codes::MISSING_POSITION],
        );
    }

    #[test]
    fn ellipse_missing_size_is_s004() {
        assert_invalid(
            "ellipse center at (50px, 50px)\n    fill orange",
            &[codes::MISSING_SIZE],
        );
    }

    #[test]
    fn ellipse_negative_size_is_s001_for_each_axis() {
        assert_invalid(
            "ellipse center at (50px, 50px)\n    size -80px x 0px\n    fill orange",
            &[codes::NEGATIVE_DIMENSION, codes::NEGATIVE_DIMENSION],
        );
    }

    #[test]
    fn square_missing_position_is_s003() {
        assert_invalid(
            "square\n    size 50px\n    fill red",
            &[codes::MISSING_POSITION],
        );
    }

    #[test]
    fn square_missing_size_is_s004() {
        assert_invalid(
            "square at (10px, 20px)\n    fill red",
            &[codes::MISSING_SIZE],
        );
    }

    #[test]
    fn square_negative_size_is_s001() {
        assert_invalid(
            "square at (10px, 20px)\n    size -50px\n    fill red",
            &[codes::NEGATIVE_DIMENSION],
        );
    }

    #[test]
    fn square_non_pixel_size_is_s007() {
        assert_invalid(
            "square at (10px, 20px)\n    size 50%\n    fill red",
            &[codes::NON_PIXEL_SHAPE_DIMENSION],
        );
    }

    #[test]
    fn triangle_missing_position_is_s003() {
        assert_invalid(
            "triangle\n    size 50px x 60px\n    fill purple",
            &[codes::MISSING_POSITION],
        );
    }

    #[test]
    fn triangle_missing_size_is_s004() {
        assert_invalid(
            "triangle at (10px, 20px)\n    fill purple",
            &[codes::MISSING_SIZE],
        );
    }

    #[test]
    fn triangle_negative_size_is_s001() {
        assert_invalid(
            "triangle at (10px, 20px)\n    size 50px x -60px\n    fill purple",
            &[codes::NEGATIVE_DIMENSION],
        );
    }
    #[test]
    fn polygon_missing_points_is_s008() {
        assert_invalid("polygon\n    fill purple", &[codes::MISSING_POINTS]);
    }

    #[test]
    fn polygon_too_few_points_is_s009() {
        let bag = validate_body(
            "polygon\n    points\n        (0px, 0px)\n        (50px, 0px)\n    fill purple",
        );
        assert_eq!(codes_of(&bag), vec![codes::TOO_FEW_POINTS]);
    }

    #[test]
    fn polygon_non_pixel_point_is_s007() {
        let bag = validate_body(
            "polygon\n    points\n        (0%, 0px)\n        (50px, 0px)\n        (25px, 40px)\n    fill purple",
        );
        assert_eq!(codes_of(&bag), vec![codes::NON_PIXEL_SHAPE_DIMENSION]);
    }

    #[test]
    fn polygon_negative_coordinates_are_valid() {
        let bag = validate_body(
            "polygon\n    points\n        (-10px, -10px)\n        (50px, 0px)\n        (25px, 40px)\n    fill purple",
        );
        assert!(!bag.has_errors(), "diagnostics: {:?}", bag.as_slice());
    }
    #[test]
    fn line_missing_from_is_s003() {
        assert_invalid(
            "line to (50px, 60px)\n    stroke black",
            &[codes::MISSING_POSITION],
        );
    }

    #[test]
    fn line_missing_to_is_s003() {
        assert_invalid(
            "line from (10px, 20px)\n    stroke black",
            &[codes::MISSING_POSITION],
        );
    }

    #[test]
    fn line_missing_stroke_is_s010() {
        assert_invalid(
            "line from (10px, 20px) to (50px, 60px)\n    width 4px",
            &[codes::MISSING_STROKE],
        );
    }

    #[test]
    fn line_negative_width_is_s011() {
        let bag = validate_body(
            "line from (10px, 20px) to (50px, 60px)\n    stroke black\n    width -4px",
        );
        assert_eq!(codes_of(&bag), vec![codes::INVALID_STROKE_WIDTH]);
    }

    #[test]
    fn line_non_pixel_width_is_s007() {
        assert_invalid(
            "line from (10px, 20px) to (50px, 60px)\n    stroke black\n    width 4%",
            &[codes::NON_PIXEL_SHAPE_DIMENSION],
        );
    }

    #[test]
    fn polyline_missing_points_is_s008() {
        assert_invalid("polyline\n    stroke blue", &[codes::MISSING_POINTS]);
    }

    #[test]
    fn polyline_too_few_points_is_s012() {
        assert_invalid(
            "polyline\n    points\n        (10px, 20px)\n    stroke blue",
            &[codes::TOO_FEW_POLYLINE_POINTS],
        );
    }

    #[test]
    fn polyline_missing_stroke_is_s010() {
        assert_invalid(
            "polyline\n    points\n        (10px, 20px)\n        (30px, 40px)",
            &[codes::MISSING_STROKE],
        );
    }

    #[test]
    fn polyline_negative_width_is_s011() {
        let bag = validate_body(
            "polyline\n    points\n        (10px, 20px)\n        (30px, 40px)\n    stroke blue\n    width 0px",
        );
        assert_eq!(codes_of(&bag), vec![codes::INVALID_STROKE_WIDTH]);
    }
    #[test]
    fn path_without_commands_is_s013() {
        assert_invalid("path\n    fill purple", &[codes::MISSING_PATH_START]);
    }

    #[test]
    fn path_not_starting_with_start_is_s013() {
        assert_invalid(
            "path\n    line to (10px, 10px)\n    start at (0px, 0px)",
            &[codes::MISSING_PATH_START],
        );
    }

    #[test]
    fn path_non_pixel_command_coordinate_is_s007() {
        assert_invalid(
            "path\n    start at (0px, 0px)\n    line to (10%, 10px)",
            &[codes::NON_PIXEL_SHAPE_DIMENSION],
        );
    }

    #[test]
    fn path_negative_width_is_s011() {
        let bag = validate_body(
            "path\n    start at (0px, 0px)\n    line to (10px, 10px)\n    stroke black\n    width -2px",
        );
        assert_eq!(codes_of(&bag), vec![codes::INVALID_STROKE_WIDTH]);
    }

    #[test]
    fn path_with_only_fill_or_only_stroke_is_valid() {
        let fill_only = validate_body(
            "path\n    start at (0px, 0px)\n    line to (10px, 0px)\n    close\n    fill red",
        );
        assert!(
            !fill_only.has_errors(),
            "diagnostics: {:?}",
            fill_only.as_slice()
        );
        let stroke_only = validate_body(
            "path\n    start at (0px, 0px)\n    line to (10px, 0px)\n    stroke blue",
        );
        assert!(
            !stroke_only.has_errors(),
            "diagnostics: {:?}",
            stroke_only.as_slice()
        );
    }
    #[test]
    fn rectangle_stroke_only_is_valid() {
        let bag = validate_body(
            "rectangle at (10px, 20px)\n    size 50px x 60px\n    stroke white\n    width 4px",
        );
        assert!(!bag.has_errors(), "diagnostics: {:?}", bag.as_slice());
    }

    #[test]
    fn rectangle_fill_and_stroke_is_valid() {
        let bag = validate_body(
            "rectangle at (10px, 20px)\n    size 50px x 60px\n    fill blue\n    stroke white\n    width 4px",
        );
        assert!(!bag.has_errors(), "diagnostics: {:?}", bag.as_slice());
    }

    #[test]
    fn closed_shape_negative_stroke_width_is_s011() {
        let bag = validate_body(
            "circle center at (50px, 50px)\n    radius 25px\n    stroke black\n    width -2px",
        );
        assert_eq!(codes_of(&bag), vec![codes::INVALID_STROKE_WIDTH]);
    }

    #[test]
    fn closed_shape_non_pixel_stroke_width_is_s007() {
        let bag = validate_body(
            "polygon\n    points\n        (0px, 0px)\n        (50px, 0px)\n        (25px, 40px)\n    stroke black\n    width 2%",
        );
        assert_eq!(codes_of(&bag), vec![codes::NON_PIXEL_SHAPE_DIMENSION]);
    }
    #[test]
    fn valid_primitives_pass() {
        for body in [
            "rectangle at (10px, 20px)\n    size 50px x 60px\n    fill red",
            "circle center at (50px, 50px)\n    radius 25px\n    fill blue",
            "ellipse center at (50px, 50px)\n    size 80px x 40px\n    fill orange",
            "square at (10px, 20px)\n    size 50px\n    fill red",
            "triangle at (10px, 20px)\n    size 50px x 60px\n    fill purple",
            "polygon\n    points\n        (0px, 0px)\n        (50px, 0px)\n        (25px, 40px)\n    fill purple",
            "line from (10px, 20px) to (50px, 60px)\n    stroke black\n    width 4px",
            "polyline\n    points\n        (10px, 20px)\n        (30px, 40px)\n        (50px, 20px)\n    stroke blue\n    width 5px",
            "path\n    start at (10px, 10px)\n    line to (50px, 10px)\n    close\n    fill purple\n    stroke black\n    width 3px",
            "picture \"photo.png\"\n    at (10px, 20px)\n    size 80px x 40px\n    fit cover\n    opacity 0.5",
        ] {
            assert_valid(body);
        }
    }

    #[test]
    fn closed_shape_fill_is_optional() {
        for body in [
            "rectangle at (10px, 20px)\n    size 50px x 60px",
            "circle center at (50px, 50px)\n    radius 25px",
            "ellipse center at (50px, 50px)\n    size 80px x 40px",
            "square at (10px, 20px)\n    size 50px",
            "triangle at (10px, 20px)\n    size 50px x 60px",
            "polygon\n    points\n        (0px, 0px)\n        (50px, 0px)\n        (25px, 40px)",
        ] {
            assert_valid(body);
        }
    }

    #[test]
    fn valid_transform_passes() {
        let bag = validate_body(
            "rectangle at (10px, 10px)\n    size 50px x 40px\n    rotate 30deg\n    scale 0.8\n    opacity 0.5",
        );
        assert!(!bag.has_errors(), "diagnostics: {:?}", bag.as_slice());
    }

    #[test]
    fn non_degree_rotation_is_s014() {
        assert_invalid(
            "rectangle at (10px, 10px)\n    size 50px x 40px\n    rotate 30pt",
            &[codes::INVALID_TRANSFORM],
        );
    }

    #[test]
    fn non_positive_scale_is_s014() {
        assert_invalid(
            "circle center at (50px, 50px)\n    radius 10px\n    scale -1",
            &[codes::INVALID_TRANSFORM],
        );
    }

    #[test]
    fn out_of_range_opacity_is_s014() {
        let bag = validate_body(
            "path\n    start at (0px, 0px)\n    line to (10px, 0px)\n    opacity 1.5",
        );
        assert_eq!(codes_of(&bag), vec![codes::INVALID_TRANSFORM]);
    }

    #[test]
    fn transform_applies_to_line_primitive() {
        let bag = validate_body(
            "line from (0px, 0px) to (10px, 10px)\n    stroke black\n    rotate 45deg\n    opacity 0.7",
        );
        assert!(!bag.has_errors(), "diagnostics: {:?}", bag.as_slice());
    }

    #[test]
    fn picture_without_position_is_s003() {
        assert_invalid(
            "picture \"photo.png\"\n    size 80px x 40px",
            &[codes::MISSING_POSITION],
        );
    }

    #[test]
    fn picture_empty_path_is_s022() {
        assert_invalid(
            "picture \"\"\n    at (0px, 0px)",
            &[codes::EMPTY_RESOURCE_PATH],
        );
    }

    #[test]
    fn picture_negative_size_is_s001() {
        assert_invalid(
            "picture \"photo.png\"\n    at (0px, 0px)\n    size -80px x 40px",
            &[codes::NEGATIVE_DIMENSION],
        );
    }

    #[test]
    fn picture_size_is_optional() {
        assert_valid("picture \"photo.png\"\n    at (0px, 0px)");
    }
}
