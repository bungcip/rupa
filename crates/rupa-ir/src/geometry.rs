//! Geometry IR for Rupa.
//!
//! Geometry and paint are separate (specification §17): a [`ShapeNode`] owns
//! geometry, an optional [`PaintId`] referencing the paint store, an optional
//! stroke, a transform, and opacity. The same geometry can be reused with
//! different paints, and the same paint can be reused by many shapes.

use crate::paint::PaintId;
use serde::{Deserialize, Serialize};

/// Unique identifier for geometry objects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GeometryId(pub u64);

/// Rectangle geometry
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rectangle {
    /// X coordinate of top-left corner
    pub x: f32,
    /// Y coordinate of top-left corner
    pub y: f32,
    /// Width of rectangle
    pub width: f32,
    /// Height of rectangle
    pub height: f32,
}

/// Circle geometry
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Circle {
    /// X coordinate of center
    pub cx: f32,
    /// Y coordinate of center
    pub cy: f32,
    /// Radius of circle
    pub radius: f32,
}

/// Ellipse geometry (center plus full width and height).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ellipse {
    /// X coordinate of center
    pub cx: f32,
    /// Y coordinate of center
    pub cy: f32,
    /// Full width
    pub width: f32,
    /// Full height
    pub height: f32,
}

/// Triangle geometry defined by its bounding box.
///
/// The triangle is isosceles with its apex at the top-center of the bounding
/// box and its base along the bottom edge (DECISION-009).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Triangle {
    /// X coordinate of bounding-box top-left
    pub x: f32,
    /// Y coordinate of bounding-box top-left
    pub y: f32,
    /// Bounding-box width
    pub width: f32,
    /// Bounding-box height
    pub height: f32,
}

/// A polygon vertex in the current coordinate system.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    /// X coordinate
    pub x: f32,
    /// Y coordinate
    pub y: f32,
}

/// Closed polygon geometry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Polygon {
    /// Ordered vertices; at least three after validation
    pub points: Vec<Point>,
}

/// Line geometry with explicit endpoints.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Line {
    /// Start X
    pub x1: f32,
    /// Start Y
    pub y1: f32,
    /// End X
    pub x2: f32,
    /// End Y
    pub y2: f32,
}

/// Open polyline geometry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Polyline {
    /// Ordered points; at least two after validation
    pub points: Vec<Point>,
}

/// Path command in the IR.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PathCommand {
    /// Begin a subpath
    MoveTo(Point),
    /// Straight segment
    LineTo(Point),
    /// Cubic Bézier segment
    CubicTo {
        /// First control point
        control1: Point,
        /// Second control point
        control2: Point,
        /// End point
        to: Point,
    },
    /// Quadratic Bézier segment
    QuadTo {
        /// Control point
        control: Point,
        /// End point
        to: Point,
    },
    /// Close the current subpath
    Close,
}

/// Path geometry as an ordered command sequence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Path {
    /// Commands in source order
    pub commands: Vec<PathCommand>,
}

/// Geometry enum for all primitive types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Geometry {
    /// Rectangle geometry (also used for squares)
    Rectangle(Rectangle),
    /// Circle geometry
    Circle(Circle),
    /// Ellipse geometry
    Ellipse(Ellipse),
    /// Triangle geometry
    Triangle(Triangle),
    /// Closed polygon geometry
    Polygon(Polygon),
    /// Line geometry
    Line(Line),
    /// Open polyline geometry
    Polyline(Polyline),
    /// Path geometry
    Path(Path),
}

/// Shape node in the IR
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShapeNode {
    /// Geometry of the shape
    pub geometry: Geometry,
    /// Fill paint ID
    pub fill: Option<PaintId>,
    /// Stroke paint and style
    pub stroke: Option<Stroke>,
    /// Transform matrix
    pub transform: rupa_geometry::Transform,
    /// Opacity (0.0 to 1.0)
    pub opacity: f32,
}

/// Stroke paint reference plus style.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stroke {
    /// Paint used to stroke the geometry
    pub paint: PaintId,
    /// Stroke style
    pub style: StrokeStyle,
}

/// Stroke style
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrokeStyle {
    /// Stroke width
    pub width: f32,
    /// Line cap style
    pub cap: LineCap,
    /// Line join style
    pub join: LineJoin,
    /// Dash pattern
    pub dash: Option<Vec<f32>>,
    /// Miter limit
    pub miter_limit: f32,
}

impl StrokeStyle {
    /// Create a default stroke style with the given width.
    pub fn new(width: f32) -> Self {
        Self {
            width,
            cap: LineCap::Butt,
            join: LineJoin::Miter,
            dash: None,
            miter_limit: 4.0,
        }
    }
}

/// Line cap style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineCap {
    /// Butt cap
    Butt,
    /// Round cap
    Round,
    /// Square cap
    Square,
}

/// Line join style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineJoin {
    /// Miter join
    Miter,
    /// Round join
    Round,
    /// Bevel join
    Bevel,
}

/// Re-export Transform from rupa-geometry
pub use rupa_geometry::Transform;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paint::{Color, Paint};

    #[test]
    fn geometry_and_paint_are_separate() {
        // The same geometry can be associated with no paint, or a paint from
        // the paint store, without embedding color data in the geometry.
        let geometry = Geometry::Circle(Circle {
            cx: 20.0,
            cy: 30.0,
            radius: 10.0,
        });
        let paint_id = PaintId(7);
        let paint = Paint::Solid(Color::from_rgb8(255, 0, 0));

        let shape = ShapeNode {
            geometry: geometry.clone(),
            fill: Some(paint_id),
            stroke: None,
            transform: rupa_geometry::Transform::identity(),
            opacity: 1.0,
        };

        assert_eq!(shape.geometry, geometry);
        assert_eq!(shape.fill, Some(paint_id));
        assert_eq!(paint, Paint::Solid(Color::from_rgb8(255, 0, 0)));
    }

    #[test]
    fn rectangle_geometry_uses_top_left_and_size() {
        let rect = Geometry::Rectangle(Rectangle {
            x: 10.0,
            y: 20.0,
            width: 50.0,
            height: 60.0,
        });
        match rect {
            Geometry::Rectangle(rect) => {
                assert_eq!((rect.x, rect.y), (10.0, 20.0));
                assert_eq!((rect.width, rect.height), (50.0, 60.0));
            }
            _ => panic!("expected rectangle"),
        }
    }

    #[test]
    fn path_geometry_stores_commands() {
        let path = Geometry::Path(Path {
            commands: vec![
                PathCommand::MoveTo(Point { x: 0.0, y: 0.0 }),
                PathCommand::LineTo(Point { x: 20.0, y: 0.0 }),
                PathCommand::CubicTo {
                    control1: Point { x: 30.0, y: 10.0 },
                    control2: Point { x: 30.0, y: 20.0 },
                    to: Point { x: 20.0, y: 30.0 },
                },
                PathCommand::QuadTo {
                    control: Point { x: 10.0, y: 20.0 },
                    to: Point { x: 0.0, y: 0.0 },
                },
                PathCommand::Close,
            ],
        });
        match path {
            Geometry::Path(path) => assert_eq!(path.commands.len(), 5),
            _ => panic!("expected path"),
        }
    }

    #[test]
    fn line_geometry_stores_endpoints() {
        let line = Geometry::Line(Line {
            x1: 1.0,
            y1: 2.0,
            x2: 30.0,
            y2: 40.0,
        });
        match line {
            Geometry::Line(line) => {
                assert_eq!((line.x1, line.y1, line.x2, line.y2), (1.0, 2.0, 30.0, 40.0));
            }
            _ => panic!("expected line"),
        }
    }

    #[test]
    fn polyline_geometry_stores_ordered_points() {
        let polyline = Geometry::Polyline(Polyline {
            points: vec![
                Point { x: 0.0, y: 0.0 },
                Point { x: 10.0, y: 20.0 },
                Point { x: 30.0, y: 5.0 },
            ],
        });
        match polyline {
            Geometry::Polyline(polyline) => assert_eq!(polyline.points.len(), 3),
            _ => panic!("expected polyline"),
        }
    }

    #[test]
    fn stroke_style_defaults_and_paint_reference() {
        let style = StrokeStyle::new(3.0);
        assert_eq!(style.width, 3.0);
        assert_eq!(style.cap, LineCap::Butt);
        assert_eq!(style.join, LineJoin::Miter);
        assert!(style.dash.is_none());
        assert_eq!(style.miter_limit, 4.0);

        let stroke = Stroke {
            paint: PaintId(2),
            style,
        };
        assert_eq!(stroke.paint, PaintId(2));
    }

    #[test]
    fn polygon_geometry_stores_ordered_points() {
        let polygon = Geometry::Polygon(Polygon {
            points: vec![
                Point { x: 0.0, y: 0.0 },
                Point { x: 20.0, y: 0.0 },
                Point { x: 10.0, y: 10.0 },
            ],
        });
        match polygon {
            Geometry::Polygon(polygon) => {
                assert_eq!(polygon.points.len(), 3);
                assert_eq!(polygon.points[1], Point { x: 20.0, y: 0.0 });
            }
            _ => panic!("expected polygon"),
        }
    }

    #[test]
    fn ellipse_geometry_uses_center_and_full_size() {
        let ellipse = Geometry::Ellipse(Ellipse {
            cx: 100.0,
            cy: 50.0,
            width: 80.0,
            height: 40.0,
        });
        match ellipse {
            Geometry::Ellipse(ellipse) => {
                assert_eq!((ellipse.cx, ellipse.cy), (100.0, 50.0));
                assert_eq!((ellipse.width, ellipse.height), (80.0, 40.0));
            }
            _ => panic!("expected ellipse"),
        }
    }

    #[test]
    fn triangle_geometry_uses_bounding_box() {
        let triangle = Geometry::Triangle(Triangle {
            x: 10.0,
            y: 20.0,
            width: 30.0,
            height: 40.0,
        });
        match triangle {
            Geometry::Triangle(triangle) => {
                assert_eq!((triangle.x, triangle.y), (10.0, 20.0));
                assert_eq!((triangle.width, triangle.height), (30.0, 40.0));
            }
            _ => panic!("expected triangle"),
        }
    }

    #[test]
    fn circle_geometry_uses_center_and_radius() {
        let circle = Geometry::Circle(Circle {
            cx: 50.0,
            cy: 60.0,
            radius: 25.0,
        });
        match circle {
            Geometry::Circle(circle) => {
                assert_eq!((circle.cx, circle.cy), (50.0, 60.0));
                assert_eq!(circle.radius, 25.0);
            }
            _ => panic!("expected circle"),
        }
    }
}
