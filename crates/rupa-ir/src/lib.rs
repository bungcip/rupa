//! Rupa IR crate: Graphics Intermediate Representation.

#![warn(missing_docs)]

pub mod document;
pub mod geometry;
pub mod paint;

/// Re-export commonly used types from document
pub use document::{
    BlendMode, CompositionChild, DEFAULT_CANVAS_HEIGHT, DEFAULT_CANVAS_WIDTH, Filter, FitMode,
    GraphicsDocument, GroupInstance, LayerDefinition, LayerMode, PictureNode, stable_z_index_order,
};
/// Re-export commonly used types from geometry
pub use geometry::{
    Circle, Ellipse, Geometry, GeometryId, Line, LineCap, LineJoin, Path, PathCommand, Point,
    Polygon, Polyline, Rectangle, ShapeNode, Stroke, StrokeStyle, Triangle,
};
/// Re-export commonly used types from paint
pub use paint::{
    Color, Gradient, GradientStop, LinearGradient, Paint, PaintId, PaintStore, PatternPaint,
    RadialGradient,
};
/// Re-export Transform from rupa-geometry
pub use rupa_geometry::Transform;
