//! Graphics document IR for Rupa.
//!
//! A [`GraphicsDocument`] is backend-independent: it owns the canvas
//! dimensions, optional background color, the paint arena, an ordered list of
//! root composition children, and reusable layer definitions. Geometry refers
//! to paints by [`PaintId`] (specification §17).
//!
//! Every document has exactly one implicit root layer whose bounds equal the
//! canvas. Top-level primitives and placements are stored as the root layer's
//! children in source declaration order. Rendering uses a stable z-index order
//! over that list (specification §6.3): lower z-index values render first and
//! equal values keep source order.

use crate::geometry::{Geometry, Point, ShapeNode, Transform};
use crate::paint::{Color, Paint, PaintId, PaintStore};
use serde::{Deserialize, Serialize};

/// Reserved name of the compiler-created root layer.
pub const ROOT_LAYER_NAME: &str = "root";

/// Canvas width used when the document omits `size` (specification §5).
pub const DEFAULT_CANVAS_WIDTH: u32 = 256;

/// Canvas height used when the document omits `size` (specification §5).
pub const DEFAULT_CANVAS_HEIGHT: u32 = 256;

/// A compiled document ready for rendering.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphicsDocument {
    /// Canvas width in pixels; already defaulted to [`DEFAULT_CANVAS_WIDTH`]
    /// when the source omitted `size`.
    pub width: u32,
    /// Canvas height in pixels; already defaulted to [`DEFAULT_CANVAS_HEIGHT`]
    /// when the source omitted `size`.
    pub height: u32,
    /// Background color; `None` means transparent, which is the default when
    /// the source omits `background`.
    pub background: Option<Color>,
    /// Paint arena referenced by shapes.
    pub paints: PaintStore,
    /// Implicit root layer. Its bounds equal the canvas, it does not clip beyond
    /// the output canvas, and it is never a reusable definition.
    pub root: LayerDefinition,
    /// Reusable group definitions, drawn only where placed.
    pub groups: Vec<GroupDefinition>,
    /// Explicit layer definitions, composited once as planes.
    pub layers: Vec<LayerDefinition>,
    /// Deterministic random seed for the document.
    pub seed: u64,
}

impl Default for GraphicsDocument {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

impl GraphicsDocument {
    /// Create a transparent document with the given pixel dimensions and an
    /// implicit root layer with matching bounds.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            background: None,
            paints: PaintStore::new(),
            root: LayerDefinition {
                name: ROOT_LAYER_NAME.to_string(),
                width,
                height,
                mode: LayerMode::Vector,
                clip: false,
                effects: Vec::new(),
                children: Vec::new(),
            },
            groups: Vec::new(),
            layers: Vec::new(),
            seed: 0,
        }
    }

    /// Set the background color.
    pub fn with_background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    /// Add a paint and return its stable ID.
    pub fn add_paint(&mut self, paint: Paint) -> PaintId {
        self.paints.add(paint)
    }

    /// Add a root-layer child at the end of source order.
    pub fn push_child(&mut self, child: CompositionChild) {
        self.root.children.push(child);
    }

    /// Root-layer children in source declaration order.
    pub fn children(&self) -> &[CompositionChild] {
        &self.root.children
    }

    /// Mutable root-layer children.
    pub fn children_mut(&mut self) -> &mut Vec<CompositionChild> {
        &mut self.root.children
    }

    /// Add a directly drawn shape at the end of source order.
    pub fn push_shape(&mut self, shape: ShapeNode) {
        self.push_child(CompositionChild::Shape(shape));
    }

    /// Add a root layer instance at the end of source order.
    pub fn push_instance(&mut self, instance: GroupInstance) {
        self.push_child(CompositionChild::Instance(instance));
    }

    /// Add a directly drawn picture at the end of source order.
    pub fn push_picture(&mut self, picture: PictureNode) {
        self.push_child(CompositionChild::Picture(picture));
    }

    /// Add a directly drawn text at the end of source order.
    pub fn push_text(&mut self, text: TextNode) {
        self.push_child(CompositionChild::Text(text));
    }

    /// Add a reusable group definition.
    pub fn push_group(&mut self, group: GroupDefinition) {
        self.groups.push(group);
    }

    /// Add a layer definition that is composited once as a plane.
    pub fn push_layer(&mut self, layer: LayerDefinition) {
        self.layers.push(layer);
    }

    /// Add a layer plane to the root composition at the end of source order.
    pub fn push_plane(&mut self, plane: LayerPlane) {
        self.push_child(CompositionChild::Plane(plane));
    }

    /// Root-level shapes in source declaration order.
    pub fn shapes(&self) -> Vec<&ShapeNode> {
        self.root_nodes(|child| match child {
            CompositionChild::Shape(shape) => Some(shape),
            _ => None,
        })
    }

    /// Root-level pictures in source declaration order.
    pub fn pictures(&self) -> Vec<&PictureNode> {
        self.root_nodes(|child| match child {
            CompositionChild::Picture(picture) => Some(picture),
            _ => None,
        })
    }

    /// Root-level texts in source declaration order.
    pub fn texts(&self) -> Vec<&TextNode> {
        self.root_nodes(|child| match child {
            CompositionChild::Text(text) => Some(text),
            _ => None,
        })
    }

    /// Root-level layer instances in source declaration order.
    pub fn instances(&self) -> Vec<&GroupInstance> {
        self.root_nodes(|child| match child {
            CompositionChild::Instance(instance) => Some(instance),
            _ => None,
        })
    }

    /// Collect typed root children in source order.
    fn root_nodes<'a, T>(
        &'a self,
        extract: impl Fn(&'a CompositionChild) -> Option<&'a T>,
    ) -> Vec<&'a T> {
        self.root.children.iter().filter_map(extract).collect()
    }

    /// Root children in stable z-index render order.
    ///
    /// Lower z-index values come first; children with equal z-index keep their
    /// source declaration order (specification §6.3).
    pub fn children_in_render_order(&self) -> Vec<&CompositionChild> {
        self.root.children_in_render_order()
    }
}

/// One child of a composition, in source declaration order.
///
/// Used by the root composition and by layer definitions. A composition parent
/// orders its direct shapes and layer instances together. Direct shapes have
/// no `z-index` property and participate at z-index `0`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CompositionChild {
    /// Directly drawn shape.
    Shape(ShapeNode),
    /// External image placement.
    Picture(PictureNode),
    /// Text primitive.
    Text(TextNode),
    /// Group instance placement.
    Instance(GroupInstance),
    /// Layer plane composited once at the document origin.
    Plane(LayerPlane),
    /// Raster clear operation.
    Clear(Color),
    /// Raster pixel operation.
    Pixel(PixelNode),
    /// Raster brush stroke operation.
    Brush(BrushNode),
}

/// How an image is mapped into its picture frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FitMode {
    /// Scale to fit inside the frame, preserving aspect ratio, centered.
    Contain,
    /// Scale to cover the frame, preserving aspect ratio, center-cropped.
    Cover,
    /// Stretch to the frame exactly.
    Fill,
    /// Draw at natural size from the frame's top-left corner.
    None,
}

/// Text primitive placed in a composition (specification §9).
///
/// Position is the baseline origin unless `centered` is set. The IR stores
/// the resolved font size and fill color so rendering stays backend-independent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextNode {
    /// Text content.
    pub text: String,
    /// Baseline/anchor x in the parent coordinate space.
    pub x: f32,
    /// Baseline/anchor y in the parent coordinate space.
    pub y: f32,
    /// Whether the anchor is the text center.
    pub centered: bool,
    /// Font family or resource name.
    pub font: Option<String>,
    /// Font size in pixels.
    pub size: f32,
    /// Font weight.
    pub weight: FontWeight,
    /// Fill color.
    pub fill: Color,
    /// Text alignment.
    pub align: TextAlign,
    /// Text transform applied about the anchor.
    pub transform: Transform,
    /// Text opacity multiplier.
    pub opacity: f32,
}

/// Font weight for text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FontWeight {
    /// Normal weight.
    Normal,
    /// Bold weight.
    Bold,
}

/// Text alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextAlign {
    /// Left alignment.
    Left,
    /// Center alignment.
    Center,
    /// Right alignment.
    Right,
}

/// Raster pixel operation placed in a composition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PixelNode {
    /// X coordinate in the parent coordinate space.
    pub x: f32,
    /// Y coordinate in the parent coordinate space.
    pub y: f32,
    /// Pixel color.
    pub color: Color,
}

/// Raster brush stroke operation placed in a composition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrushNode {
    /// Brush diameter in pixels.
    pub size: f32,
    /// Brush color.
    pub color: Color,
    /// Stroke points in parent coordinate space.
    pub points: Vec<Point>,
}

/// Type aliases for alternate naming.
/// Alias for [`FontWeight`].
pub type TextWeight = FontWeight;
/// Alias for [`TextAlign`].
pub type Alignment = TextAlign;
/// Alias for [`TextAlign`].
pub type TextAlignment = TextAlign;

/// External raster image placed in a composition (specification §10).
///
/// `path` is the resource reference exactly as written in the source; resource
/// resolution and decoding happen at render time so the IR stays
/// backend-independent and serializable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PictureNode {
    /// Resource reference relative to the document directory.
    pub path: String,
    /// Frame top-left x in the parent coordinate space.
    pub x: f32,
    /// Frame top-left y in the parent coordinate space.
    pub y: f32,
    /// Frame width; `None` uses the image's natural width.
    pub width: Option<f32>,
    /// Frame height; `None` uses the image's natural height.
    pub height: Option<f32>,
    /// How the image maps into the frame.
    pub fit: FitMode,
    /// Picture transform applied about the frame center.
    pub transform: Transform,
    /// Picture opacity multiplier.
    pub opacity: f32,
}

impl CompositionChild {
    /// Integer z-index used for ordering within the composition parent.
    pub fn z_index(&self) -> i32 {
        match self {
            CompositionChild::Shape(_)
            | CompositionChild::Picture(_)
            | CompositionChild::Text(_)
            | CompositionChild::Clear(_)
            | CompositionChild::Pixel(_)
            | CompositionChild::Brush(_) => 0,
            CompositionChild::Instance(instance) => instance.z_index,
            CompositionChild::Plane(plane) => plane.z_index,
        }
    }
}

/// Return children in stable z-index render order.
///
/// Lower z-index values come first. `sort_by_key` is a stable sort, so children
/// with equal z-index keep their source declaration order (specification §6.3).
pub fn stable_z_index_order(children: &[CompositionChild]) -> Vec<&CompositionChild> {
    let mut ordered: Vec<&CompositionChild> = children.iter().collect();
    ordered.sort_by_key(|child| child.z_index());
    ordered
}

/// Rendering mode for a compiled layer definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LayerMode {
    /// Vector composition mode (default).
    #[default]
    Vector,
    /// Raster surface mode.
    Raster,
}

/// Filter applied to a rendered layer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Filter {
    /// Gaussian blur with the given radius in pixels.
    Blur(f32),
    /// Brightness multiplier.
    Brightness(f32),
    /// Contrast multiplier.
    Contrast(f32),
    /// Saturation multiplier.
    Saturation(f32),
    /// Convert to grayscale.
    Grayscale,
    /// Hue rotation in degrees.
    HueRotate(f32),
    /// Sharpen amount.
    Sharpen(f32),
    /// Noise amount.
    Noise(f32),
    /// Pixelation block size.
    Pixelate(f32),
    /// Vignette amount.
    Vignette(f32),
}

/// Compiled layer definition with ordered layer-local children.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerDefinition {
    /// Unique definition name.
    pub name: String,
    /// Declared width in pixels.
    pub width: u32,
    /// Declared height in pixels.
    pub height: u32,
    /// Rendering mode for this layer.
    pub mode: LayerMode,
    /// Whether layer content is clipped to its declared bounds.
    pub clip: bool,
    /// Ordered filter effects applied after rendering layer content.
    pub effects: Vec<Filter>,
    /// Shapes and nested instances in layer-local coordinates, in source
    /// declaration order.
    pub children: Vec<CompositionChild>,
}

impl LayerDefinition {
    /// Center of the layer's local bounds, used as transform pivot.
    pub fn center(&self) -> (f32, f32) {
        (self.width as f32 / 2.0, self.height as f32 / 2.0)
    }

    /// Children in stable z-index render order within this composition.
    ///
    /// Lower z-index values come first; equal values keep source order
    /// (specification §6.3).
    pub fn children_in_render_order(&self) -> Vec<&CompositionChild> {
        stable_z_index_order(&self.children)
    }
}

/// Blend mode for compositing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BlendMode {
    /// Normal compositing.
    #[default]
    Normal,
    /// Multiply.
    Multiply,
    /// Screen.
    Screen,
    /// Overlay.
    Overlay,
    /// Darken.
    Darken,
    /// Lighten.
    Lighten,
    /// Color dodge.
    ColorDodge,
    /// Color burn.
    ColorBurn,
    /// Soft light.
    SoftLight,
    /// Hard light.
    HardLight,
    /// Difference.
    Difference,
    /// Exclusion.
    Exclusion,
}

/// Axis-aligned rectangle in a local coordinate space.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Bounds {
    /// Left edge.
    pub x: f32,
    /// Top edge.
    pub y: f32,
    /// Width; never negative.
    pub width: f32,
    /// Height; never negative.
    pub height: f32,
}

impl Bounds {
    /// Center of the bounds, used as the transform pivot of a group.
    pub fn center(&self) -> (f32, f32) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    /// Whether the box has no area.
    pub fn is_empty(&self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }
}

/// Compiled group definition.
///
/// `bounds` is either the declared size anchored at the local origin, or the
/// measured automatic bounding box of the children (specification §6.2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupDefinition {
    /// Unique definition name.
    pub name: String,
    /// Effective bounds in group-local coordinates.
    pub bounds: Bounds,
    /// Whether content is clipped to `bounds` (only with a declared size).
    pub clip: bool,
    /// Shapes and nested instances in group-local coordinates, in source order.
    pub children: Vec<CompositionChild>,
}

/// Compiled placement of a group definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupInstance {
    /// Referenced group definition name.
    pub group: String,
    /// X position of the group's local origin.
    pub x: f32,
    /// Y position of the group's local origin.
    pub y: f32,
    /// Instance transform (scale/rotation) applied around the bounds center.
    pub transform: Transform,
    /// Instance opacity multiplier.
    pub opacity: f32,
    /// Integer z-index; absent values default to `0` (specification §6.5).
    pub z_index: i32,
    /// Blend mode used when compositing this instance.
    pub blend_mode: BlendMode,
}

impl GroupDefinition {
    /// Children in stable z-index render order within this group.
    ///
    /// Lower z-index values come first; equal values keep source order
    /// (specification §6.3).
    pub fn children_in_render_order(&self) -> Vec<&CompositionChild> {
        stable_z_index_order(&self.children)
    }
}

impl GroupInstance {
    /// Map group-local coordinates to the parent composition's coordinates.
    ///
    /// Local `(0, 0)` maps to the instance position. Scale/rotation pivot on
    /// the group's effective bounds center; the automatic bounding box never
    /// re-anchors content (specification §6.2).
    pub fn local_to_parent_transform(&self, group: &GroupDefinition) -> Transform {
        let (cx, cy) = group.bounds.center();
        Transform::translate(self.x, self.y)
            .multiply(&Transform::translate(cx, cy))
            .multiply(&self.transform)
            .multiply(&Transform::translate(-cx, -cy))
    }
}

/// A layer composited once as a plane over the document canvas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerPlane {
    /// Referenced layer definition name.
    pub layer: String,
    /// Integer z-index; absent values default to `0` (specification §6.5).
    pub z_index: i32,
    /// Opacity of the composited plane.
    pub opacity: f32,
    /// Blend mode used when compositing the plane.
    pub blend_mode: BlendMode,
}

/// Measured bounds of one geometry, in its own local coordinates.
fn geometry_bounds(geometry: &Geometry) -> Bounds {
    use crate::geometry::PathCommand;

    fn point_bounds(points: &[crate::geometry::Point]) -> Bounds {
        if points.is_empty() {
            return Bounds {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            };
        }
        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        for point in points {
            min_x = min_x.min(point.x);
            min_y = min_y.min(point.y);
            max_x = max_x.max(point.x);
            max_y = max_y.max(point.y);
        }
        Bounds {
            x: min_x,
            y: min_y,
            width: max_x - min_x,
            height: max_y - min_y,
        }
    }

    match geometry {
        Geometry::Rectangle(rect) => Bounds {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        },
        Geometry::Circle(circle) => Bounds {
            x: circle.cx - circle.radius,
            y: circle.cy - circle.radius,
            width: circle.radius * 2.0,
            height: circle.radius * 2.0,
        },
        Geometry::Ellipse(ellipse) => Bounds {
            x: ellipse.cx - ellipse.width / 2.0,
            y: ellipse.cy - ellipse.height / 2.0,
            width: ellipse.width,
            height: ellipse.height,
        },
        Geometry::Triangle(triangle) => Bounds {
            x: triangle.x,
            y: triangle.y,
            width: triangle.width,
            height: triangle.height,
        },
        Geometry::Polygon(polygon) => point_bounds(&polygon.points),
        Geometry::Polyline(polyline) => point_bounds(&polyline.points),
        Geometry::Line(line) => point_bounds(&[
            crate::geometry::Point {
                x: line.x1,
                y: line.y1,
            },
            crate::geometry::Point {
                x: line.x2,
                y: line.y2,
            },
        ]),
        Geometry::Path(path) => {
            let mut points: Vec<crate::geometry::Point> = Vec::new();
            for command in &path.commands {
                match command {
                    PathCommand::MoveTo(point) | PathCommand::LineTo(point) => {
                        points.push(*point);
                    }
                    PathCommand::CubicTo {
                        control1,
                        control2,
                        to,
                    } => {
                        points.push(*control1);
                        points.push(*control2);
                        points.push(*to);
                    }
                    PathCommand::QuadTo { control, to } => {
                        points.push(*control);
                        points.push(*to);
                    }
                    PathCommand::Close => {}
                }
            }
            point_bounds(&points)
        }
    }
}

/// Apply a transform to a point.
fn transform_point(transform: &Transform, x: f32, y: f32) -> (f32, f32) {
    (
        transform.a * x + transform.c * y + transform.e,
        transform.b * x + transform.d * y + transform.f,
    )
}

/// Axis-aligned bounds of `bounds` transformed by `transform`.
fn transformed_bounds(bounds: Bounds, transform: &Transform) -> Bounds {
    let corners = [
        transform_point(transform, bounds.x, bounds.y),
        transform_point(transform, bounds.x + bounds.width, bounds.y),
        transform_point(transform, bounds.x, bounds.y + bounds.height),
        transform_point(transform, bounds.x + bounds.width, bounds.y + bounds.height),
    ];
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for (x, y) in corners {
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }
    Bounds {
        x: min_x,
        y: min_y,
        width: max_x - min_x,
        height: max_y - min_y,
    }
}

fn union_bounds(a: Bounds, b: Bounds) -> Bounds {
    let min_x = a.x.min(b.x);
    let min_y = a.y.min(b.y);
    let max_x = (a.x + a.width).max(b.x + b.width);
    let max_y = (a.y + a.height).max(b.y + b.height);
    Bounds {
        x: min_x,
        y: min_y,
        width: max_x - min_x,
        height: max_y - min_y,
    }
}

/// Measured automatic bounds of a composition's children (specification §6.2).
///
/// The result is expressed in the composition's local coordinates. Stroke width
/// and font metrics are approximated: strokes are ignored and text is measured
/// from its font size and character count. Raster clear operations are excluded
/// because they cover the whole surface rather than a measurable region.
/// Returns `None` when the composition has no measurable content.
pub fn composition_bounds(
    children: &[CompositionChild],
    groups: &[GroupDefinition],
) -> Option<Bounds> {
    fn inner(
        children: &[CompositionChild],
        groups: &[GroupDefinition],
        active: &mut Vec<String>,
    ) -> Option<Bounds> {
        let mut result: Option<Bounds> = None;
        for child in children {
            let bounds = match child {
                CompositionChild::Shape(shape) => Some(transformed_bounds(
                    geometry_bounds(&shape.geometry),
                    &shape.transform,
                )),
                CompositionChild::Picture(picture) => Some(transformed_bounds(
                    Bounds {
                        x: picture.x,
                        y: picture.y,
                        width: picture.width.unwrap_or(0.0),
                        height: picture.height.unwrap_or(0.0),
                    },
                    &picture.transform,
                )),
                CompositionChild::Text(text) => {
                    let advance = 0.6 * text.size * text.text.chars().count() as f32;
                    let (x, y) = if text.centered {
                        (text.x - advance / 2.0, text.y - text.size * 0.8)
                    } else {
                        (text.x, text.y - text.size * 0.8)
                    };
                    Some(transformed_bounds(
                        Bounds {
                            x,
                            y,
                            width: advance,
                            height: text.size,
                        },
                        &text.transform,
                    ))
                }
                CompositionChild::Instance(instance) => {
                    let group = groups.iter().find(|group| group.name == instance.group)?;
                    if active.contains(&instance.group) {
                        None
                    } else {
                        active.push(instance.group.clone());
                        let nested = inner(&group.children, groups, active);
                        active.pop();
                        nested.map(|bounds| {
                            transformed_bounds(bounds, &instance.local_to_parent_transform(group))
                        })
                    }
                }
                CompositionChild::Plane(_) | CompositionChild::Clear(_) => None,
                CompositionChild::Pixel(pixel) => Some(Bounds {
                    x: pixel.x,
                    y: pixel.y,
                    width: 1.0,
                    height: 1.0,
                }),
                CompositionChild::Brush(brush) => {
                    let half = brush.size / 2.0;
                    let mut bounds: Option<Bounds> = None;
                    for point in &brush.points {
                        let point_bounds = Bounds {
                            x: point.x - half,
                            y: point.y - half,
                            width: brush.size,
                            height: brush.size,
                        };
                        bounds = Some(match bounds {
                            Some(existing) => union_bounds(existing, point_bounds),
                            None => point_bounds,
                        });
                    }
                    bounds
                }
            };
            if let Some(bounds) = bounds {
                result = Some(match result {
                    Some(existing) => union_bounds(existing, bounds),
                    None => bounds,
                });
            }
        }
        result
    }

    inner(children, groups, &mut Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::{Circle, Geometry};
    use crate::paint::Color;
    use rupa_geometry::Transform;

    #[test]
    fn new_document_is_transparent_and_empty() {
        let doc = GraphicsDocument::new(200, 100);
        assert_eq!((doc.width, doc.height), (200, 100));
        assert!(doc.background.is_none());
        assert!(doc.paints.is_empty());
        assert!(doc.root.children.is_empty());
    }

    #[test]
    fn document_has_implicit_root_layer() {
        let mut doc = GraphicsDocument::new(200, 100);
        assert_eq!(doc.root.name, ROOT_LAYER_NAME);
        assert_eq!((doc.root.width, doc.root.height), (200, 100));
        assert_eq!(doc.root.mode, LayerMode::Vector);
        assert!(!doc.root.clip);
        assert!(doc.layers.is_empty());

        doc.push_shape(shape(0.0));
        assert_eq!(doc.children().len(), 1);
        assert_eq!(doc.root.children.len(), 1);
    }

    #[test]
    fn document_owns_paints_and_shapes() {
        let mut doc = GraphicsDocument::new(10, 10).with_background(Color::white());
        let paint = doc.add_paint(Paint::Solid(Color::from_rgb8(255, 0, 0)));
        doc.push_shape(ShapeNode {
            geometry: Geometry::Circle(Circle {
                cx: 5.0,
                cy: 5.0,
                radius: 2.0,
            }),
            fill: Some(paint),
            stroke: None,
            transform: Transform::identity(),
            opacity: 1.0,
        });
        assert_eq!(doc.background, Some(Color::white()));
        assert_eq!(doc.paints.len(), 1);
        assert_eq!(doc.shapes().len(), 1);
        assert_eq!(doc.shapes()[0].fill, Some(PaintId(0)));
    }

    #[test]
    fn document_stores_layers_and_instances_separately() {
        let mut doc = GraphicsDocument::new(100, 100);
        doc.push_layer(LayerDefinition {
            name: "Dot".to_string(),
            width: 10,
            height: 10,
            mode: LayerMode::Vector,
            clip: true,
            effects: Vec::new(),
            children: Vec::new(),
        });
        doc.push_instance(GroupInstance {
            group: "Dot".to_string(),
            x: 5.0,
            y: 6.0,
            transform: Transform::identity(),
            opacity: 1.0,
            z_index: 3,
            blend_mode: BlendMode::Normal,
        });
        assert_eq!(doc.layers.len(), 1);
        assert_eq!(doc.instances().len(), 1);
        assert_eq!(doc.instances()[0].group, "Dot");
        assert_eq!(doc.instances()[0].z_index, 3);
    }

    fn shape(x: f32) -> ShapeNode {
        ShapeNode {
            geometry: Geometry::Circle(Circle {
                cx: x,
                cy: 0.0,
                radius: 1.0,
            }),
            fill: None,
            stroke: None,
            transform: Transform::identity(),
            opacity: 1.0,
        }
    }

    fn instance(layer: &str, z_index: i32) -> GroupInstance {
        GroupInstance {
            group: layer.to_string(),
            x: 0.0,
            y: 0.0,
            transform: Transform::identity(),
            opacity: 1.0,
            z_index,
            blend_mode: BlendMode::Normal,
        }
    }

    #[test]
    fn root_children_keep_source_order_without_z_index() {
        let mut doc = GraphicsDocument::new(10, 10);
        doc.push_shape(shape(0.0));
        doc.push_instance(instance("A", 0));
        doc.push_shape(shape(1.0));

        let order = doc.children_in_render_order();
        assert!(matches!(
            order[0],
            CompositionChild::Shape(ShapeNode { .. })
        ));
        assert!(matches!(order[1], CompositionChild::Instance(_)));
        assert!(matches!(
            order[2],
            CompositionChild::Shape(ShapeNode { .. })
        ));
    }

    #[test]
    fn lower_z_index_renders_behind_higher_values() {
        let mut doc = GraphicsDocument::new(10, 10);
        doc.push_instance(instance("High", 10));
        doc.push_instance(instance("Low", -5));
        doc.push_instance(instance("Middle", 2));

        let layers: Vec<&str> = doc
            .children_in_render_order()
            .iter()
            .map(|child| match child {
                CompositionChild::Clear(_) => "clear",
                CompositionChild::Pixel(_) => "pixel",
                CompositionChild::Brush(_) => "brush",
                CompositionChild::Plane(_) => "plane",
                CompositionChild::Instance(instance) => instance.group.as_str(),
                CompositionChild::Shape(_)
                | CompositionChild::Picture(_)
                | CompositionChild::Text(_) => {
                    panic!("expected instance")
                }
            })
            .collect();
        assert_eq!(layers, vec!["Low", "Middle", "High"]);
    }

    #[test]
    fn equal_z_index_keeps_source_order() {
        let mut doc = GraphicsDocument::new(10, 10);
        doc.push_instance(instance("First", 4));
        doc.push_instance(instance("Second", 4));
        doc.push_instance(instance("Third", 4));

        let layers: Vec<&str> = doc
            .children_in_render_order()
            .iter()
            .map(|child| match child {
                CompositionChild::Clear(_) => "clear",
                CompositionChild::Pixel(_) => "pixel",
                CompositionChild::Brush(_) => "brush",
                CompositionChild::Plane(_) => "plane",
                CompositionChild::Instance(instance) => instance.group.as_str(),
                CompositionChild::Shape(_)
                | CompositionChild::Picture(_)
                | CompositionChild::Text(_) => {
                    panic!("expected instance")
                }
            })
            .collect();
        assert_eq!(layers, vec!["First", "Second", "Third"]);
    }

    #[test]
    fn direct_shapes_participate_at_z_index_zero() {
        let mut doc = GraphicsDocument::new(10, 10);
        doc.push_instance(instance("Above", 5));
        doc.push_shape(shape(0.0));
        doc.push_instance(instance("Below", -1));

        let mut kinds = Vec::new();
        for child in doc.children_in_render_order() {
            kinds.push(match child {
                CompositionChild::Shape(_) => "shape",
                CompositionChild::Picture(_) => "picture",
                CompositionChild::Text(_) => "text",
                CompositionChild::Clear(_) => "clear",
                CompositionChild::Pixel(_) => "pixel",
                CompositionChild::Brush(_) => "brush",
                CompositionChild::Plane(_) => "plane",
                CompositionChild::Instance(instance) => instance.group.as_str(),
            });
        }
        // Below (-1), shape (0), Above (5); the shape precedes the instance
        // declared before it only because of z-index, not source order.
        assert_eq!(kinds, vec!["Below", "shape", "Above"]);
    }

    #[test]
    fn shapes_and_instances_accessors_filter_in_source_order() {
        let mut doc = GraphicsDocument::new(10, 10);
        doc.push_instance(instance("One", 2));
        doc.push_shape(shape(0.0));
        doc.push_instance(instance("Two", 1));
        doc.push_shape(shape(1.0));

        assert_eq!(doc.shapes().len(), 2);
        assert_eq!(doc.instances().len(), 2);
        assert_eq!(doc.instances()[0].group, "One");
        assert_eq!(doc.instances()[1].group, "Two");
    }

    fn group(name: &str, width: f32, height: f32) -> GroupDefinition {
        GroupDefinition {
            name: name.to_string(),
            bounds: Bounds {
                x: 0.0,
                y: 0.0,
                width,
                height,
            },
            clip: true,
            children: Vec::new(),
        }
    }

    #[test]
    fn layer_children_order_by_z_index_independently() {
        let mut outer = group("Outer", 10.0, 10.0);
        outer.children = vec![
            CompositionChild::Instance(GroupInstance {
                group: "High".to_string(),
                x: 0.0,
                y: 0.0,
                transform: Transform::identity(),
                opacity: 1.0,
                z_index: 5,
                blend_mode: BlendMode::Normal,
            }),
            CompositionChild::Shape(shape(0.0)),
            CompositionChild::Instance(GroupInstance {
                group: "Low".to_string(),
                x: 0.0,
                y: 0.0,
                transform: Transform::identity(),
                opacity: 1.0,
                z_index: -1,
                blend_mode: BlendMode::Normal,
            }),
        ];

        let mut order = Vec::new();
        for child in outer.children_in_render_order() {
            order.push(match child {
                CompositionChild::Shape(_) => "shape",
                CompositionChild::Picture(_) => "picture",
                CompositionChild::Text(_) => "text",
                CompositionChild::Clear(_) => "clear",
                CompositionChild::Pixel(_) => "pixel",
                CompositionChild::Brush(_) => "brush",
                CompositionChild::Plane(_) => "plane",
                CompositionChild::Instance(instance) => instance.group.as_str(),
            });
        }
        assert_eq!(order, vec!["Low", "shape", "High"]);
    }

    fn identity_instance(layer: &str, x: f32, y: f32) -> GroupInstance {
        GroupInstance {
            group: layer.to_string(),
            x,
            y,
            transform: Transform::identity(),
            opacity: 1.0,
            z_index: 0,
            blend_mode: BlendMode::Normal,
        }
    }

    #[test]
    fn local_origin_maps_to_instance_position() {
        let layer = group("L", 20.0, 10.0);
        let instance = identity_instance("L", 30.0, 40.0);
        let matrix = instance.local_to_parent_transform(&layer);
        assert_eq!(matrix.transform_point(0.0, 0.0), (30.0, 40.0));
    }

    #[test]
    fn layer_center_maps_to_position_plus_center() {
        let layer = group("L", 20.0, 10.0);
        let instance = identity_instance("L", 30.0, 40.0);
        let matrix = instance.local_to_parent_transform(&layer);
        assert_eq!(matrix.transform_point(10.0, 5.0), (40.0, 45.0));
    }

    #[test]
    fn instance_scale_pivots_on_layer_center() {
        let layer = group("L", 20.0, 10.0);
        let mut instance = identity_instance("L", 30.0, 40.0);
        instance.transform = Transform::scale(2.0, 2.0);
        let matrix = instance.local_to_parent_transform(&layer);
        // Center remains fixed; the local origin moves away from it.
        assert_eq!(matrix.transform_point(10.0, 5.0), (40.0, 45.0));
        assert_eq!(matrix.transform_point(0.0, 0.0), (20.0, 35.0));
    }

    #[test]
    fn local_mapping_does_not_mutate_definition() {
        let layer = group("L", 20.0, 10.0);
        let before = layer.clone();
        let instance = identity_instance("L", 30.0, 40.0);
        let _ = instance.local_to_parent_transform(&layer);
        assert_eq!(layer, before);
    }
}
