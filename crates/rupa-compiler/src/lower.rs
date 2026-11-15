//! Lowering from the syntax AST to backend-independent graphics IR.
//!
//! Callers must validate the document first ([`crate::Validator`]). Lowering
//! still checks required properties and canvas dimensions defensively so it
//! never panics on malformed input.

use std::collections::HashMap;

use rupa_ir::document::{
    Bounds, BrushNode, CompositionChild, DEFAULT_CANVAS_HEIGHT, DEFAULT_CANVAS_WIDTH,
    FitMode as IrFitMode, FontWeight as IrFontWeight, GraphicsDocument,
    GroupDefinition as IrGroupDefinition, GroupInstance as IrGroupInstance,
    LayerDefinition as IrLayerDefinition, LayerMode as IrLayerMode, LayerPlane as IrLayerPlane,
    PictureNode as IrPictureNode, PixelNode, TextAlign as IrTextAlign, TextNode as IrTextNode,
    composition_bounds,
};
use rupa_ir::geometry::{
    Circle as IrCircle, Ellipse as IrEllipse, Geometry, Line as IrLine, Path as IrPath,
    PathCommand as IrPathCommand, Point as IrPoint, Polygon as IrPolygon, Polyline as IrPolyline,
    Rectangle as IrRectangle, ShapeNode, Stroke as IrStroke, StrokeStyle, Triangle as IrTriangle,
};
use rupa_ir::paint::{Color, PaintId};
use rupa_ir::{BlendMode, Filter, Transform as IrTransform};
use rupa_syntax::ast::{
    Brush, Circle, Dimension, Document, DocumentChild, Ellipse, Fill, FitMode, FontWeight,
    Function, GroupInstance, LayerDefinition, LayerEffect, LayerMode, Line, Path, PathCommand,
    Picture, Pixel, Polygon, Polyline, Position, Rectangle, Repeat, Square, Stroke, StrokeCap,
    StrokeJoin, Text, TextAlign, Transform, Triangle, Unit, Value,
};
use rupa_syntax::diagnostic::{Diagnostic, DiagnosticBag, codes};
use rupa_syntax::span::Span;
use rupa_syntax::{IStr, Prng};

use crate::paint::PaintValidator;

const ZERO_BOUNDS: Bounds = Bounds {
    x: 0.0,
    y: 0.0,
    width: 0.0,
    height: 0.0,
};

/// Lower a validated AST document into graphics IR.
///
/// Canvas dimensions are rounded to the nearest pixel. Shapes are emitted in
/// source order, with paints stored in the document's arena.
pub fn lower_document(document: &Document) -> Result<GraphicsDocument, DiagnosticBag> {
    let mut graphics = GraphicsDocument::default();
    let mut bag = DiagnosticBag::new();
    let seed = document.seed.unwrap_or(Prng::DEFAULT_SEED);
    graphics.seed = seed;

    {
        let mut lowerer = Lowerer {
            graphics: &mut graphics,
            bag: &mut bag,
            functions: document
                .functions
                .iter()
                .map(|f| (f.name, f.clone()))
                .collect(),
            rng: Prng::new(seed),
        };
        lowerer.lower(document);
    }

    if bag.has_errors() {
        Err(bag)
    } else {
        Ok(graphics)
    }
}

/// Appearance fields shared by every shape primitive.
struct ShapeProps<'a> {
    fill: Option<&'a Fill>,
    stroke: &'a Option<Stroke>,
    width: &'a Option<Dimension>,
    transform: &'a Option<Transform>,
}

/// Build [`ShapeProps`] from any shape whose fields use the standard names.
macro_rules! props {
    ($shape:expr) => {
        ShapeProps {
            fill: $shape.fill.as_ref(),
            stroke: &$shape.stroke,
            width: &$shape.width,
            transform: &$shape.transform,
        }
    };
}

/// Shared state for lowering one document.
struct Lowerer<'a> {
    graphics: &'a mut GraphicsDocument,
    bag: &'a mut DiagnosticBag,
    functions: HashMap<IStr, Function>,
    rng: Prng,
}

impl Lowerer<'_> {
    fn lower(&mut self, document: &Document) {
        self.canvas(document);
        if let Some(background) = &document.background {
            match PaintValidator::parse_color(&background.color, background.span) {
                Ok(color) => self.graphics.background = Some(color),
                Err(diagnostic) => self.bag.push(diagnostic),
            }
        }

        let mut scope = HashMap::new();
        let mut root_children = self.composition_children(&document.children, &mut scope);

        // Explicit layers are single-use planes. Appending them after the root's
        // own children keeps top-level content behind a layer that uses the same
        // z-index (specification §6.5).
        for layer in &document.layers {
            if let Some(lowered) = self.layer(layer) {
                self.graphics.layers.push(lowered);
            }
            root_children.push(CompositionChild::Plane(IrLayerPlane {
                layer: layer.name.to_string(),
                z_index: layer.z_index.unwrap_or(0.0) as i32,
                opacity: layer.opacity.unwrap_or(1.0) as f32,
                blend_mode: self.blend(layer.blend.as_deref(), layer.span),
            }));
        }
        self.graphics.root.children = root_children;

        self.groups(document);
    }

    fn canvas(&mut self, document: &Document) {
        // `size` is optional; an omitted size renders at the documented default
        // canvas (specification §5).
        match &document.size {
            Some(size) => {
                if let Some(width) = self.canvas_dimension(&size.width, "width") {
                    self.graphics.width = width;
                }
                if let Some(height) = self.canvas_dimension(&size.height, "height") {
                    self.graphics.height = height;
                }
            }
            None => {
                self.graphics.width = DEFAULT_CANVAS_WIDTH;
                self.graphics.height = DEFAULT_CANVAS_HEIGHT;
            }
        }
        self.graphics.root.width = self.graphics.width;
        self.graphics.root.height = self.graphics.height;
    }

    /// Lower composition children in source order, interleaving shapes and
    /// instances. Used for the document root and for layer definitions.
    fn composition_children(
        &mut self,
        children: &[DocumentChild],
        scope: &mut HashMap<IStr, Value>,
    ) -> Vec<CompositionChild> {
        let mut lowered = Vec::new();
        for child in children {
            match child {
                DocumentChild::Let(variable) => {
                    if let Ok(value) =
                        variable
                            .value
                            .evaluate(scope, &self.functions, &mut self.rng)
                    {
                        scope.insert(variable.name, value);
                    }
                }
                DocumentChild::Repeat(repeat) => lowered.extend(self.repeat(repeat, scope)),
                DocumentChild::Place(instance) => {
                    if let Some(instance) = self.instance(instance) {
                        lowered.push(CompositionChild::Instance(instance));
                    }
                }
                DocumentChild::Picture(picture) => {
                    if let Some(picture) = self.picture(picture) {
                        lowered.push(CompositionChild::Picture(picture));
                    }
                }
                DocumentChild::Text(text) => {
                    if let Some(text) = self.text(text) {
                        lowered.push(CompositionChild::Text(text));
                    }
                }
                DocumentChild::Clear(clear) => {
                    match PaintValidator::parse_color(&clear.color, clear.span) {
                        Ok(color) => lowered.push(CompositionChild::Clear(color)),
                        Err(diagnostic) => self.bag.push(diagnostic),
                    }
                }
                DocumentChild::Pixel(pixel) => {
                    if let Some(pixel) = self.pixel(pixel) {
                        lowered.push(pixel);
                    }
                }
                DocumentChild::Brush(brush) => {
                    if let Some(brush) = self.brush(brush) {
                        lowered.push(brush);
                    }
                }
                child => {
                    if let Some(shape) = self.shape_node(child) {
                        lowered.push(CompositionChild::Shape(shape));
                    }
                }
            }
        }
        lowered
    }

    fn repeat(
        &mut self,
        repeat: &Repeat,
        scope: &mut HashMap<IStr, Value>,
    ) -> Vec<CompositionChild> {
        let count = match repeat.count.evaluate(scope, &self.functions, &mut self.rng) {
            Ok(Value::Number(n)) if n >= 0.0 => n as usize,
            Ok(Value::Dimension(d)) if d.unit == Unit::Px && d.value >= 0.0 => d.value as usize,
            _ => 0,
        };

        let mut lowered = Vec::new();
        for i in 0..count {
            let mut body_scope = scope.clone();
            if let Some(index) = &repeat.index {
                body_scope.insert(*index, Value::Number(i as f64));
            }
            lowered.extend(self.composition_children(&repeat.body, &mut body_scope));
        }
        lowered
    }

    fn shape_node(&mut self, child: &DocumentChild) -> Option<ShapeNode> {
        match child {
            DocumentChild::Rectangle(rect) => self.rectangle(rect),
            DocumentChild::Circle(circle) => self.circle(circle),
            DocumentChild::Ellipse(ellipse) => self.ellipse(ellipse),
            DocumentChild::Square(square) => self.square(square),
            DocumentChild::Triangle(triangle) => self.triangle(triangle),
            DocumentChild::Polygon(polygon) => self.polygon(polygon),
            DocumentChild::Line(line) => self.line(line),
            DocumentChild::Polyline(polyline) => self.polyline(polyline),
            DocumentChild::Path(path) => self.path(path),
            _ => None,
        }
    }

    fn picture(&mut self, picture: &Picture) -> Option<IrPictureNode> {
        let position = self.required(
            picture.position.as_ref(),
            codes::MISSING_POSITION,
            "picture",
            "at (x, y)",
            picture.span,
            "add 'at (x, y)'; the position is the frame's top-left corner",
        )?;
        let (transform, opacity) = lower_transform(&picture.transform);
        Some(IrPictureNode {
            path: picture.path.to_string(),
            x: position.x.value as f32,
            y: position.y.value as f32,
            width: picture.size.as_ref().map(|size| size.width.value as f32),
            height: picture.size.as_ref().map(|size| size.height.value as f32),
            fit: match picture.fit.unwrap_or(FitMode::Contain) {
                FitMode::Contain => IrFitMode::Contain,
                FitMode::Cover => IrFitMode::Cover,
                FitMode::Fill => IrFitMode::Fill,
                FitMode::None => IrFitMode::None,
            },
            transform,
            opacity,
        })
    }

    fn text(&mut self, text: &Text) -> Option<IrTextNode> {
        // Accept either anchor form; prefer `center at` when present.
        let (x, y, centered) = if let Some(center) = &text.center {
            (center.x.value as f32, center.y.value as f32, true)
        } else if let Some(position) = &text.position {
            (position.x.value as f32, position.y.value as f32, false)
        } else {
            self.bag.push(
                Diagnostic::error(
                    codes::MISSING_POSITION,
                    "cannot lower text without 'at (x, y)' or 'center at (x, y)'",
                    text.span,
                )
                .with_help("add 'at (x, y)' or 'center at (x, y)'"),
            );
            return None;
        };

        let (transform, opacity) = lower_transform(&text.transform);
        let fill = match &text.fill {
            None => Color::black(),
            Some(fill) => match &fill.color {
                None => Color::transparent(),
                Some(color) => {
                    PaintValidator::parse_color(color, fill.span).unwrap_or(Color::black())
                }
            },
        };

        Some(IrTextNode {
            text: text.text.to_string(),
            x,
            y,
            centered,
            font: text.font.map(|font| font.to_string()),
            size: text.size.as_ref().map_or(16.0, |d| d.value as f32),
            weight: match text.weight {
                Some(FontWeight::Bold) => IrFontWeight::Bold,
                _ => IrFontWeight::Normal,
            },
            fill,
            align: match text.align {
                Some(TextAlign::Center) => IrTextAlign::Center,
                Some(TextAlign::Right) => IrTextAlign::Right,
                _ => IrTextAlign::Left,
            },
            transform,
            opacity,
        })
    }

    fn instance(&mut self, instance: &GroupInstance) -> Option<IrGroupInstance> {
        let position = self.required(
            instance.position.as_ref(),
            codes::MISSING_POSITION,
            "placement",
            "at (x, y)",
            instance.span,
            "add 'at (x, y)' after the group name",
        )?;

        let (transform, opacity) = lower_transform(&instance.transform);
        Some(IrGroupInstance {
            group: instance.name.to_string(),
            x: position.x.value as f32,
            y: position.y.value as f32,
            transform,
            opacity,
            z_index: instance.z_index.unwrap_or(0.0) as i32,
            blend_mode: self.blend(instance.blend.as_deref(), instance.span),
        })
    }

    fn pixel(&mut self, pixel: &Pixel) -> Option<CompositionChild> {
        let position = self.required(
            pixel.position.as_ref(),
            codes::MISSING_POSITION,
            "pixel",
            "at (x, y)",
            pixel.span,
            "add 'at (x, y)'",
        )?;
        let fill = self.required(
            pixel.fill.as_ref(),
            codes::MISSING_FILL,
            "pixel",
            "fill",
            pixel.span,
            "add 'fill <color>'",
        )?;
        let color = match &fill.color {
            // fill none -> transparent pixel
            None => Color::transparent(),
            Some(color) => match PaintValidator::parse_color(color, fill.span) {
                Ok(color) => color,
                Err(diagnostic) => {
                    self.bag.push(diagnostic);
                    return None;
                }
            },
        };
        Some(CompositionChild::Pixel(PixelNode {
            x: position.x.value as f32,
            y: position.y.value as f32,
            color,
        }))
    }

    fn brush(&mut self, brush: &Brush) -> Option<CompositionChild> {
        let size = self.required(
            brush.size.as_ref(),
            codes::MISSING_SIZE,
            "brush",
            "size",
            brush.span,
            "add 'size <n>px'",
        )?;
        if size.unit != Unit::Px {
            self.bag.push(
                Diagnostic::error(
                    codes::NON_PIXEL_SHAPE_DIMENSION,
                    "brush size must use pixel units",
                    size.span,
                )
                .with_help("use a pixel value such as '12px'"),
            );
            return None;
        }
        if size.value <= 0.0 {
            self.bag.error(
                codes::INVALID_DIMENSIONS,
                "brush size must be positive",
                size.span,
            );
            return None;
        }
        let fill = self.required(
            brush.fill.as_ref(),
            codes::MISSING_FILL,
            "brush",
            "fill",
            brush.span,
            "add 'fill <color>'",
        )?;
        let Some(color) = &fill.color else {
            self.bag.error(
                codes::MISSING_FILL,
                "brush fill cannot be 'none'",
                fill.span,
            );
            return None;
        };
        let color = match PaintValidator::parse_color(color, fill.span) {
            Ok(color) => color,
            Err(diagnostic) => {
                self.bag.push(diagnostic);
                return None;
            }
        };
        Some(CompositionChild::Brush(BrushNode {
            size: size.value as f32,
            color,
            points: brush.points.iter().map(ir_point).collect(),
        }))
    }

    /// Lower a blend-mode name, reporting an unknown name.
    fn blend(&mut self, blend: Option<&str>, span: Span) -> BlendMode {
        use BlendMode::*;
        match blend {
            None | Some("normal") => Normal,
            Some("multiply") => Multiply,
            Some("screen") => Screen,
            Some("overlay") => Overlay,
            Some("darken") => Darken,
            Some("lighten") => Lighten,
            Some("color-dodge") => ColorDodge,
            Some("color-burn") => ColorBurn,
            Some("soft-light") => SoftLight,
            Some("hard-light") => HardLight,
            Some("difference") => Difference,
            Some("exclusion") => Exclusion,
            Some(other) => {
                self.bag.error(
                    codes::INVALID_PROPERTY_VALUE,
                    format!("unknown blend mode `{other}`"),
                    span,
                );
                Normal
            }
        }
    }

    /// Lower every group definition, then measure the automatic bounds.
    ///
    /// All definitions are lowered before any bounds are measured so nested
    /// placements resolve regardless of declaration order. Two measurement
    /// passes settle bounds of nested groups, whose instance pivot depends on
    /// them.
    fn groups(&mut self, document: &Document) {
        let mut pending = Vec::new();
        for group in &document.groups {
            self.rng = Prng::new(self.graphics.seed);
            let mut scope = HashMap::new();
            let children = self.composition_children(&group.children, &mut scope);
            pending.push((group, children));
        }

        for (group, children) in pending {
            let (bounds, clip) = match &group.size {
                Some(size) => {
                    let width = self.canvas_dimension(&size.width, "group width");
                    let height = self.canvas_dimension(&size.height, "group height");
                    match (width, height) {
                        (Some(width), Some(height)) => (
                            Bounds {
                                x: 0.0,
                                y: 0.0,
                                width: width as f32,
                                height: height as f32,
                            },
                            true,
                        ),
                        _ => (ZERO_BOUNDS, true),
                    }
                }
                None => (ZERO_BOUNDS, false),
            };
            self.graphics.groups.push(IrGroupDefinition {
                name: group.name.to_string(),
                bounds,
                clip,
                children,
            });
        }

        for _ in 0..2 {
            for index in 0..self.graphics.groups.len() {
                if self.graphics.groups[index].clip {
                    continue;
                }
                let measured = composition_bounds(
                    &self.graphics.groups[index].children,
                    &self.graphics.groups,
                );
                self.graphics.groups[index].bounds = measured.unwrap_or(ZERO_BOUNDS);
            }
        }
    }

    fn filter(&mut self, effect: &LayerEffect) -> Option<Filter> {
        let value = |default: f32| effect.value.as_ref().map_or(default, |d| d.value as f32);
        match effect.kind.as_str() {
            "blur" => Some(Filter::Blur(value(0.0))),
            "brightness" => Some(Filter::Brightness(value(1.0))),
            "contrast" => Some(Filter::Contrast(value(1.0))),
            "saturation" => Some(Filter::Saturation(value(1.0))),
            "grayscale" => Some(Filter::Grayscale),
            "hue-rotate" => Some(Filter::HueRotate(value(0.0))),
            "sharpen" => Some(Filter::Sharpen(value(1.0))),
            "noise" => Some(Filter::Noise(value(0.1))),
            "pixelate" => Some(Filter::Pixelate(value(4.0))),
            "vignette" => Some(Filter::Vignette(value(0.5))),
            other => {
                self.bag.error(
                    codes::INVALID_PROPERTY_VALUE,
                    format!("unknown filter `{other}`"),
                    effect.span,
                );
                None
            }
        }
    }

    fn layer(&mut self, layer: &LayerDefinition) -> Option<IrLayerDefinition> {
        // An omitted layer size defaults to the document canvas (specification
        // §6.4); the canvas is already resolved at this point.
        let (width, height) = match &layer.size {
            Some(size) => (
                self.canvas_dimension(&size.width, "layer width")?,
                self.canvas_dimension(&size.height, "layer height")?,
            ),
            None => (self.graphics.width, self.graphics.height),
        };

        self.rng = Prng::new(self.graphics.seed);
        let mut scope = HashMap::new();
        let mut children = self.composition_children(&layer.children, &mut scope);

        // A `fill` written directly inside a layer paints the full layer bounds
        // before any other child (specification §6.4).
        if let Some(paint) = self.fill(layer.fill.as_ref()) {
            children.insert(
                0,
                CompositionChild::Shape(ShapeNode {
                    geometry: Geometry::Rectangle(IrRectangle {
                        x: 0.0,
                        y: 0.0,
                        width: width as f32,
                        height: height as f32,
                    }),
                    fill: Some(paint),
                    stroke: None,
                    transform: IrTransform::identity(),
                    opacity: 1.0,
                }),
            );
        }

        let effects = layer
            .effects
            .iter()
            .filter_map(|effect| self.filter(effect))
            .collect();
        Some(IrLayerDefinition {
            name: layer.name.to_string(),
            width,
            height,
            mode: match layer.mode.unwrap_or(LayerMode::Vector) {
                LayerMode::Vector => IrLayerMode::Vector,
                LayerMode::Raster => IrLayerMode::Raster,
            },
            clip: layer.clip.unwrap_or(true),
            effects,
            children,
        })
    }

    fn canvas_dimension(&mut self, dimension: &Dimension, name: &str) -> Option<u32> {
        if dimension.unit != Unit::Px {
            self.bag.push(
                Diagnostic::error(
                    codes::NON_PIXEL_DIMENSION,
                    format!("document {name} must use pixel units"),
                    dimension.span,
                )
                .with_help("use a pixel value such as '800px'"),
            );
            return None;
        }
        if !dimension.value.is_finite() || dimension.value <= 0.0 {
            self.bag.push(
                Diagnostic::error(
                    codes::INVALID_DIMENSIONS,
                    format!("document {name} must be a positive, finite number"),
                    dimension.span,
                )
                .with_help("use a value greater than zero"),
            );
            return None;
        }

        let rounded = dimension.value.round();
        if rounded > f64::from(u32::MAX) {
            self.bag.push(
                Diagnostic::error(
                    codes::CANVAS_TOO_LARGE,
                    format!("document {name} is too large for a PNG canvas"),
                    dimension.span,
                )
                .with_help("use a dimension no larger than 4294967295px"),
            );
            return None;
        }
        Some(rounded as u32)
    }

    /// Lower the appearance fields shared by every shape primitive.
    fn shape(
        &mut self,
        props: ShapeProps<'_>,
        kind: &str,
        span: Span,
        require_stroke: bool,
        geometry: Geometry,
    ) -> ShapeNode {
        let fill = self.fill(props.fill);
        let stroke = if require_stroke || props.stroke.is_some() {
            self.stroke(props.stroke, props.width, kind, span)
        } else {
            None
        };
        let (transform, opacity) = lower_transform(props.transform);
        ShapeNode {
            geometry,
            fill,
            stroke,
            transform,
            opacity,
        }
    }

    fn rectangle(&mut self, rect: &Rectangle) -> Option<ShapeNode> {
        let position = self.required(
            rect.position.as_ref(),
            codes::MISSING_POSITION,
            "rectangle",
            "at (x, y)",
            rect.span,
            "add 'at (x, y)' (top-left corner)",
        )?;
        let size = self.required(
            rect.size.as_ref(),
            codes::MISSING_SIZE,
            "rectangle",
            "size",
            rect.span,
            "add 'size <width>px x <height>px'",
        )?;
        Some(self.shape(
            props!(rect),
            "rectangle",
            rect.span,
            false,
            Geometry::Rectangle(IrRectangle {
                x: position.x.value as f32,
                y: position.y.value as f32,
                width: size.width.value as f32,
                height: size.height.value as f32,
            }),
        ))
    }

    fn circle(&mut self, circle: &Circle) -> Option<ShapeNode> {
        let center = self.required(
            circle.center.as_ref(),
            codes::MISSING_POSITION,
            "circle",
            "center at (x, y)",
            circle.span,
            "add 'center at (x, y)'",
        )?;
        let radius = self.required(
            circle.radius.as_ref(),
            codes::MISSING_RADIUS,
            "circle",
            "radius",
            circle.span,
            "add 'radius <r>px'",
        )?;
        Some(self.shape(
            props!(circle),
            "circle",
            circle.span,
            false,
            Geometry::Circle(IrCircle {
                cx: center.x.value as f32,
                cy: center.y.value as f32,
                radius: radius.value as f32,
            }),
        ))
    }

    fn ellipse(&mut self, ellipse: &Ellipse) -> Option<ShapeNode> {
        let center = self.required(
            ellipse.center.as_ref(),
            codes::MISSING_POSITION,
            "ellipse",
            "center at (x, y)",
            ellipse.span,
            "add 'center at (x, y)'",
        )?;
        let size = self.required(
            ellipse.size.as_ref(),
            codes::MISSING_SIZE,
            "ellipse",
            "size",
            ellipse.span,
            "add 'size <width>px x <height>px'",
        )?;
        Some(self.shape(
            props!(ellipse),
            "ellipse",
            ellipse.span,
            false,
            Geometry::Ellipse(IrEllipse {
                cx: center.x.value as f32,
                cy: center.y.value as f32,
                width: size.width.value as f32,
                height: size.height.value as f32,
            }),
        ))
    }

    fn square(&mut self, square: &Square) -> Option<ShapeNode> {
        let position = self.required(
            square.position.as_ref(),
            codes::MISSING_POSITION,
            "square",
            "at (x, y)",
            square.span,
            "add 'at (x, y)' (top-left corner)",
        )?;
        let size = self.required(
            square.size.as_ref(),
            codes::MISSING_SIZE,
            "square",
            "size",
            square.span,
            "add 'size <side>px'",
        )?;
        let side = size.value as f32;
        Some(self.shape(
            props!(square),
            "square",
            square.span,
            false,
            Geometry::Rectangle(IrRectangle {
                x: position.x.value as f32,
                y: position.y.value as f32,
                width: side,
                height: side,
            }),
        ))
    }

    fn triangle(&mut self, triangle: &Triangle) -> Option<ShapeNode> {
        let position = self.required(
            triangle.position.as_ref(),
            codes::MISSING_POSITION,
            "triangle",
            "at (x, y)",
            triangle.span,
            "add 'at (x, y)' (bounding-box top-left corner)",
        )?;
        let size = self.required(
            triangle.size.as_ref(),
            codes::MISSING_SIZE,
            "triangle",
            "size",
            triangle.span,
            "add 'size <width>px x <height>px'",
        )?;
        Some(self.shape(
            props!(triangle),
            "triangle",
            triangle.span,
            false,
            Geometry::Triangle(IrTriangle {
                x: position.x.value as f32,
                y: position.y.value as f32,
                width: size.width.value as f32,
                height: size.height.value as f32,
            }),
        ))
    }

    fn polygon(&mut self, polygon: &Polygon) -> Option<ShapeNode> {
        let points = self.points(
            polygon.points.as_ref(),
            3,
            "polygon",
            polygon.span,
            codes::TOO_FEW_POINTS,
        )?;
        Some(self.shape(
            props!(polygon),
            "polygon",
            polygon.span,
            false,
            Geometry::Polygon(IrPolygon { points }),
        ))
    }

    fn line(&mut self, line: &Line) -> Option<ShapeNode> {
        let from = self.required(
            line.from.as_ref(),
            codes::MISSING_POSITION,
            "line",
            "from (x, y)",
            line.span,
            "add 'from (x, y)' after 'line'",
        )?;
        let to = self.required(
            line.to.as_ref(),
            codes::MISSING_POSITION,
            "line",
            "to (x, y)",
            line.span,
            "add 'to (x, y)' after the start endpoint",
        )?;
        Some(self.shape(
            ShapeProps {
                fill: None,
                stroke: &line.stroke,
                width: &line.width,
                transform: &line.transform,
            },
            "line",
            line.span,
            true,
            Geometry::Line(IrLine {
                x1: from.x.value as f32,
                y1: from.y.value as f32,
                x2: to.x.value as f32,
                y2: to.y.value as f32,
            }),
        ))
    }

    fn polyline(&mut self, polyline: &Polyline) -> Option<ShapeNode> {
        let points = self.points(
            polyline.points.as_ref(),
            2,
            "polyline",
            polyline.span,
            codes::TOO_FEW_POLYLINE_POINTS,
        )?;
        Some(self.shape(
            ShapeProps {
                fill: None,
                stroke: &polyline.stroke,
                width: &polyline.width,
                transform: &polyline.transform,
            },
            "polyline",
            polyline.span,
            true,
            Geometry::Polyline(IrPolyline { points }),
        ))
    }

    fn path(&mut self, path: &Path) -> Option<ShapeNode> {
        if !matches!(path.commands.first(), Some(PathCommand::Start { .. })) {
            self.bag.push(
                Diagnostic::error(
                    codes::MISSING_PATH_START,
                    "cannot lower a path that does not begin with 'start at (x, y)'",
                    path.span,
                )
                .with_help("add 'start at (x, y)' as the first path command"),
            );
            return None;
        }

        let commands = path
            .commands
            .iter()
            .map(|command| match command {
                PathCommand::Start { position, .. } => IrPathCommand::MoveTo(ir_point(position)),
                PathCommand::LineTo { to, .. } => IrPathCommand::LineTo(ir_point(to)),
                PathCommand::CurveTo {
                    to,
                    control1,
                    control2,
                    ..
                } => IrPathCommand::CubicTo {
                    control1: ir_point(control1),
                    control2: ir_point(control2),
                    to: ir_point(to),
                },
                PathCommand::QuadTo { to, control, .. } => IrPathCommand::QuadTo {
                    control: ir_point(control),
                    to: ir_point(to),
                },
                PathCommand::Close { .. } => IrPathCommand::Close,
            })
            .collect();

        Some(self.shape(
            props!(path),
            "path",
            path.span,
            false,
            Geometry::Path(IrPath { commands }),
        ))
    }

    /// Lower and validate a points list shared by polygon and polyline.
    fn points(
        &mut self,
        points: Option<&Vec<Position>>,
        minimum: usize,
        shape: &str,
        span: Span,
        too_few_code: &str,
    ) -> Option<Vec<IrPoint>> {
        let points = self.required(
            points,
            codes::MISSING_POINTS,
            shape,
            "points",
            span,
            "add an indented 'points' block with '(x, y)' entries",
        )?;
        if points.len() < minimum {
            let minimum_word = if minimum == 2 { "two" } else { "three" };
            self.bag.push(
                Diagnostic::error(
                    too_few_code,
                    format!(
                        "cannot lower a {shape} with fewer than {minimum_word} points (found {})",
                        points.len()
                    ),
                    span,
                )
                .with_help(format!("add more points to the {shape}")),
            );
            return None;
        }
        Some(points.iter().map(ir_point).collect())
    }

    fn stroke(
        &mut self,
        stroke: &Option<Stroke>,
        width: &Option<Dimension>,
        shape: &str,
        span: Span,
    ) -> Option<IrStroke> {
        let Some(stroke) = stroke else {
            self.bag.push(
                Diagnostic::error(
                    codes::MISSING_STROKE,
                    format!("cannot lower a {shape} without 'stroke'"),
                    span,
                )
                .with_help("add 'stroke <color>'"),
            );
            return None;
        };
        let Some(color) = &stroke.color else {
            return None;
        };
        let paint = match PaintValidator::lower_color(color, stroke.span) {
            Ok(paint) => self.graphics.paints.add(paint),
            Err(diagnostics) => {
                self.bag.append(diagnostics);
                return None;
            }
        };
        let mut style = StrokeStyle::new(width.as_ref().map_or(1.0, |d| d.value as f32));
        style.cap = match stroke.cap {
            None | Some(StrokeCap::Butt) => rupa_ir::geometry::LineCap::Butt,
            Some(StrokeCap::Round) => rupa_ir::geometry::LineCap::Round,
            Some(StrokeCap::Square) => rupa_ir::geometry::LineCap::Square,
        };
        style.join = match stroke.join {
            None | Some(StrokeJoin::Miter) => rupa_ir::geometry::LineJoin::Miter,
            Some(StrokeJoin::Round) => rupa_ir::geometry::LineJoin::Round,
            Some(StrokeJoin::Bevel) => rupa_ir::geometry::LineJoin::Bevel,
        };
        Some(IrStroke { paint, style })
    }

    fn fill(&mut self, fill: Option<&Fill>) -> Option<PaintId> {
        let fill = fill?;
        match PaintValidator::lower_fill_to_store(fill, &mut self.graphics.paints) {
            Ok(id) => id,
            Err(diagnostics) => {
                self.bag.append(diagnostics);
                None
            }
        }
    }

    /// Extract a required property for lowering, reporting a stable diagnostic.
    fn required<'a, T>(
        &mut self,
        value: Option<&'a T>,
        code: &str,
        shape: &str,
        property: &str,
        span: Span,
        help: &str,
    ) -> Option<&'a T> {
        let Some(value) = value else {
            self.bag.push(
                Diagnostic::error(
                    code,
                    format!("cannot lower a {shape} without '{property}'"),
                    span,
                )
                .with_help(help),
            );
            return None;
        };
        Some(value)
    }
}

fn lower_transform(transform: &Option<Transform>) -> (IrTransform, f32) {
    let Some(transform) = transform else {
        return (IrTransform::identity(), 1.0);
    };

    let scale = transform
        .scale
        .as_ref()
        .map_or_else(IrTransform::identity, |scale| {
            IrTransform::scale(scale.x as f32, scale.y as f32)
        });
    let rotate = transform
        .rotate
        .as_ref()
        .map_or_else(IrTransform::identity, |rotate| {
            IrTransform::rotate((rotate.value as f32).to_radians())
        });
    // Scale first, then rotate. Translation is geometry, not transform.
    (
        rotate.multiply(&scale),
        transform.opacity.unwrap_or(1.0) as f32,
    )
}

fn ir_point(position: &Position) -> IrPoint {
    IrPoint {
        x: position.x.value as f32,
        y: position.y.value as f32,
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    use rupa_ir::document::ROOT_LAYER_NAME;
    use rupa_ir::paint::{Color, Paint};
    use rupa_syntax::span::FileId;

    fn lower(source: &str) -> GraphicsDocument {
        let compiled = crate::compile_source(source, FileId(0));
        assert!(
            !compiled.diagnostics.has_errors(),
            "diagnostics: {:?}",
            compiled.diagnostics.as_slice()
        );
        compiled.graphics.expect("lowered document")
    }

    #[test]
    fn lowers_document_dimensions_and_background() {
        let doc = lower("size 200px x 100px\nbackground white\n");
        assert_eq!((doc.width, doc.height), (200, 100));
        assert_eq!(doc.background, Some(Color::white()));
        assert!(doc.shapes().is_empty());
    }

    #[test]
    fn lowered_document_has_implicit_root_layer() {
        let doc = lower("size 200px x 100px\n");
        assert_eq!(doc.root.name, ROOT_LAYER_NAME);
        assert_eq!((doc.root.width, doc.root.height), (200, 100));
        assert!(!doc.root.clip);
        assert!(doc.layers.is_empty());
        assert!(doc.root.children.is_empty());
    }

    #[test]
    fn lowers_rectangle_with_paint() {
        let doc = lower(
            "size 100px x 100px\nrectangle at (10px, 20px)\n    size 30px x 40px\n    fill red\n",
        );
        assert_eq!(doc.shapes().len(), 1);
        match &doc.shapes()[0].geometry {
            Geometry::Rectangle(rect) => {
                assert_eq!((rect.x, rect.y), (10.0, 20.0));
                assert_eq!((rect.width, rect.height), (30.0, 40.0));
            }
            _ => panic!("expected rectangle"),
        }
        assert_eq!(
            doc.paints.get(doc.shapes()[0].fill.expect("fill")),
            Some(&Paint::Solid(Color::from_rgb8(255, 0, 0)))
        );
    }

    #[test]
    fn lowers_circle_with_paint() {
        let doc = lower(
            "size 100px x 100px\ncircle center at (50px, 60px)\n    radius 25px\n    fill \"#0000FF\"\n",
        );
        assert_eq!(doc.shapes().len(), 1);
        match &doc.shapes()[0].geometry {
            Geometry::Circle(circle) => {
                assert_eq!((circle.cx, circle.cy), (50.0, 60.0));
                assert_eq!(circle.radius, 25.0);
            }
            _ => panic!("expected circle"),
        }
        assert_eq!(
            doc.paints.get(doc.shapes()[0].fill.expect("fill")),
            Some(&Paint::Solid(Color::from_rgb8(0, 0, 255)))
        );
    }

    #[test]
    fn fill_none_lowers_to_no_paint() {
        let doc = lower(
            "size 100px x 100px\nrectangle at (0px, 0px)\n    size 50px x 50px\n    fill none\n",
        );
        assert_eq!(doc.shapes().len(), 1);
        assert!(doc.shapes()[0].fill.is_none());
        assert!(doc.paints.is_empty());
    }

    #[test]
    fn fractional_canvas_dimensions_round_to_nearest_pixel() {
        let doc = lower("size 100.6px x 99.4px\n");
        assert_eq!((doc.width, doc.height), (101, 99));
    }

    #[test]
    fn missing_size_lowers_to_default_canvas() {
        let doc = lower("background white\n");
        assert_eq!(
            (doc.width, doc.height),
            (DEFAULT_CANVAS_WIDTH, DEFAULT_CANVAS_HEIGHT)
        );
        assert_eq!((doc.root.width, doc.root.height), (256, 256));
    }

    #[test]
    fn missing_background_lowers_to_transparent() {
        let doc = lower("size 40px x 30px\n");
        assert!(doc.background.is_none());
    }

    #[test]
    fn lowers_ellipse_with_paint() {
        let doc = lower(
            "size 200px x 200px\nellipse center at (100px, 80px)\n    size 80px x 40px\n    fill orange\n",
        );
        match &doc.shapes()[0].geometry {
            Geometry::Ellipse(ellipse) => {
                assert_eq!((ellipse.cx, ellipse.cy), (100.0, 80.0));
                assert_eq!((ellipse.width, ellipse.height), (80.0, 40.0));
            }
            _ => panic!("expected ellipse"),
        }
        assert_eq!(
            doc.paints.get(doc.shapes()[0].fill.expect("fill")),
            Some(&Paint::Solid(Color::from_rgb8(255, 165, 0)))
        );
    }

    #[test]
    fn lowers_square_as_equal_side_rectangle() {
        let doc =
            lower("size 200px x 200px\nsquare at (10px, 20px)\n    size 50px\n    fill red\n");
        match &doc.shapes()[0].geometry {
            Geometry::Rectangle(rect) => {
                assert_eq!((rect.x, rect.y), (10.0, 20.0));
                assert_eq!((rect.width, rect.height), (50.0, 50.0));
            }
            _ => panic!("expected rectangle for square"),
        }
    }

    #[test]
    fn lowers_triangle_with_bounding_box() {
        let doc = lower(
            "size 200px x 200px\ntriangle at (10px, 20px)\n    size 50px x 60px\n    fill purple\n",
        );
        match &doc.shapes()[0].geometry {
            Geometry::Triangle(triangle) => {
                assert_eq!((triangle.x, triangle.y), (10.0, 20.0));
                assert_eq!((triangle.width, triangle.height), (50.0, 60.0));
            }
            _ => panic!("expected triangle"),
        }
    }

    #[test]
    fn lowers_new_shapes_in_source_order() {
        let doc = lower(
            "size 200px x 200px\nellipse center at (20px, 20px)\n    size 10px x 10px\n    fill red\nsquare at (30px, 30px)\n    size 10px\n    fill blue\ntriangle at (40px, 40px)\n    size 10px x 10px\n    fill green\n",
        );
        assert_eq!(doc.shapes().len(), 3);
        assert!(matches!(doc.shapes()[0].geometry, Geometry::Ellipse(_)));
        assert!(matches!(doc.shapes()[1].geometry, Geometry::Rectangle(_)));
        assert!(matches!(doc.shapes()[2].geometry, Geometry::Triangle(_)));
    }
    #[test]
    fn lowers_polygon_with_ordered_points() {
        let doc = lower(
            "size 200px x 200px\npolygon\n    points\n        (10px, 10px)\n        (50px, 10px)\n        (30px, 40px)\n    fill purple\n",
        );
        match &doc.shapes()[0].geometry {
            Geometry::Polygon(polygon) => {
                assert_eq!(polygon.points.len(), 3);
                assert_eq!(polygon.points[0], IrPoint { x: 10.0, y: 10.0 });
                assert_eq!(polygon.points[2], IrPoint { x: 30.0, y: 40.0 });
            }
            _ => panic!("expected polygon"),
        }
        assert_eq!(
            doc.paints.get(doc.shapes()[0].fill.expect("fill")),
            Some(&Paint::Solid(Color::from_rgb8(128, 0, 128)))
        );
    }

    #[test]
    fn polygon_without_points_reports_s008_when_lowering_directly() {
        let source = "size 100px x 100px\npolygon\n    fill red\n";
        let parsed = crate::parse_source(source, FileId(0));
        let document = parsed.document.expect("document");
        let bag = lower_document(&document).expect_err("missing points");
        assert!(
            bag.as_slice()
                .iter()
                .any(|d| d.code == codes::MISSING_POINTS)
        );
    }

    #[test]
    fn polygon_with_two_points_reports_s009_when_lowering_directly() {
        let source = "size 100px x 100px\npolygon\n    points\n        (0px, 0px)\n        (10px, 0px)\n    fill red\n";
        let parsed = crate::parse_source(source, FileId(0));
        let document = parsed.document.expect("document");
        let bag = lower_document(&document).expect_err("too few points");
        assert!(
            bag.as_slice()
                .iter()
                .any(|d| d.code == codes::TOO_FEW_POINTS)
        );
    }
    #[test]
    fn lowers_line_with_stroke_paint_and_width() {
        let doc = lower(
            "size 200px x 200px\nline from (10px, 20px) to (50px, 60px)\n    stroke black\n    width 4px\n",
        );
        match &doc.shapes()[0].geometry {
            Geometry::Line(line) => {
                assert_eq!(
                    (line.x1, line.y1, line.x2, line.y2),
                    (10.0, 20.0, 50.0, 60.0)
                );
            }
            _ => panic!("expected line"),
        }
        assert!(doc.shapes()[0].fill.is_none());
        let stroke = doc.shapes()[0].stroke.as_ref().expect("stroke");
        assert_eq!(stroke.style.width, 4.0);
        assert_eq!(
            doc.paints.get(stroke.paint),
            Some(&Paint::Solid(Color::black()))
        );
    }

    #[test]
    fn lowers_polyline_with_default_width() {
        let doc = lower(
            "size 200px x 200px\npolyline\n    points\n        (10px, 20px)\n        (30px, 40px)\n        (50px, 20px)\n    stroke blue\n",
        );
        match &doc.shapes()[0].geometry {
            Geometry::Polyline(polyline) => assert_eq!(polyline.points.len(), 3),
            _ => panic!("expected polyline"),
        }
        let stroke = doc.shapes()[0].stroke.as_ref().expect("stroke");
        assert_eq!(stroke.style.width, 1.0);
        assert_eq!(
            doc.paints.get(stroke.paint),
            Some(&Paint::Solid(Color::from_rgb8(0, 0, 255)))
        );
    }

    #[test]
    fn line_without_stroke_reports_s010_when_lowering_directly() {
        let source = "size 100px x 100px\nline from (0px, 0px) to (10px, 10px)\n    width 4px\n";
        let parsed = crate::parse_source(source, FileId(0));
        let document = parsed.document.expect("document");
        let bag = lower_document(&document).expect_err("missing stroke");
        assert!(
            bag.as_slice()
                .iter()
                .any(|d| d.code == codes::MISSING_STROKE)
        );
    }
    #[test]
    fn lowers_path_commands_and_paints() {
        let doc = lower(
            "size 200px x 200px\npath\n    start at (10px, 10px)\n    line to (50px, 10px)\n    curve to (80px, 40px)\n        control (60px, 10px) and (70px, 40px)\n    quad to (20px, 40px)\n        control (40px, 20px)\n    close\n    fill purple\n    stroke black\n    width 3px\n",
        );
        match &doc.shapes()[0].geometry {
            Geometry::Path(path) => {
                assert_eq!(path.commands.len(), 5);
                assert_eq!(
                    path.commands[0],
                    IrPathCommand::MoveTo(IrPoint { x: 10.0, y: 10.0 })
                );
                assert!(matches!(path.commands[2], IrPathCommand::CubicTo { .. }));
                assert!(matches!(path.commands[3], IrPathCommand::QuadTo { .. }));
                assert_eq!(path.commands[4], IrPathCommand::Close);
            }
            _ => panic!("expected path"),
        }
        assert!(doc.shapes()[0].fill.is_some());
        let stroke = doc.shapes()[0].stroke.as_ref().expect("stroke");
        assert_eq!(stroke.style.width, 3.0);
        assert_eq!(
            doc.paints.get(stroke.paint),
            Some(&Paint::Solid(Color::black()))
        );
    }

    #[test]
    fn lowers_path_with_only_fill() {
        let doc = lower(
            "size 100px x 100px\npath\n    start at (0px, 0px)\n    line to (10px, 0px)\n    close\n    fill red\n",
        );
        assert!(doc.shapes()[0].fill.is_some());
        assert!(doc.shapes()[0].stroke.is_none());
    }
    #[test]
    fn lowers_closed_shape_with_fill_and_stroke() {
        let doc = lower(
            "size 100px x 100px\nrectangle at (10px, 10px)\n    size 50px x 40px\n    fill blue\n    stroke white\n    width 4px\n",
        );
        assert!(doc.shapes()[0].fill.is_some());
        let stroke = doc.shapes()[0].stroke.as_ref().expect("stroke");
        assert_eq!(stroke.style.width, 4.0);
        assert_eq!(doc.paints.len(), 2);
        assert_eq!(
            doc.paints.get(stroke.paint),
            Some(&Paint::Solid(Color::white()))
        );
    }

    #[test]
    fn lowers_closed_shape_with_stroke_only() {
        let doc = lower(
            "size 100px x 100px\ncircle center at (50px, 50px)\n    radius 20px\n    stroke red\n",
        );
        assert!(doc.shapes()[0].fill.is_none());
        let stroke = doc.shapes()[0].stroke.as_ref().expect("stroke");
        assert_eq!(stroke.style.width, 1.0);
        assert_eq!(
            doc.paints.get(stroke.paint),
            Some(&Paint::Solid(Color::from_rgb8(255, 0, 0)))
        );
    }

    #[test]
    fn lowers_closed_shape_with_neither_fill_nor_stroke() {
        let doc =
            lower("size 100px x 100px\nellipse center at (50px, 50px)\n    size 40px x 20px\n");
        assert!(doc.shapes()[0].fill.is_none());
        assert!(doc.shapes()[0].stroke.is_none());
    }
    #[test]
    fn lowers_transform_matrix_and_opacity() {
        let doc = lower(
            "size 100px x 100px\nrectangle at (10px, 10px)\n    size 40px x 20px\n    rotate 90deg\n    scale 2 x 0.5\n    opacity 0.5\n",
        );
        let shape = &doc.shapes()[0];
        assert_eq!(shape.opacity, 0.5);
        let (x, y) = shape.transform.transform_point(1.0, 0.0);
        assert!((x - 0.0).abs() < 1e-5);
        assert!((y - 2.0).abs() < 1e-5);
    }

    #[test]
    fn defaults_to_identity_and_full_opacity() {
        let doc = lower("size 100px x 100px\ncircle center at (50px, 50px)\n    radius 10px\n");
        assert_eq!(doc.shapes()[0].transform, rupa_ir::Transform::identity());
        assert_eq!(doc.shapes()[0].opacity, 1.0);
    }
    #[test]
    fn lowers_stroke_cap_and_join() {
        let doc = lower(
            "size 100px x 100px\nline from (10px, 10px) to (90px, 90px)\n    stroke black\n    width 8px\n    cap round\n    join bevel\n",
        );
        let style = &doc.shapes()[0].stroke.as_ref().expect("stroke").style;
        assert_eq!(style.width, 8.0);
        assert_eq!(style.cap, rupa_ir::geometry::LineCap::Round);
        assert_eq!(style.join, rupa_ir::geometry::LineJoin::Bevel);
    }

    #[test]
    fn stroke_cap_and_join_default_to_butt_and_miter() {
        let doc = lower(
            "size 100px x 100px\npath\n    start at (0px, 0px)\n    line to (10px, 10px)\n    stroke black\n",
        );
        let style = &doc.shapes()[0].stroke.as_ref().expect("stroke").style;
        assert_eq!(style.cap, rupa_ir::geometry::LineCap::Butt);
        assert_eq!(style.join, rupa_ir::geometry::LineJoin::Miter);
    }
    #[test]
    fn layer_definitions_are_not_rendered_without_instances() {
        let doc = lower(
            "size 300px x 200px\nlayer \"Cloud\"\n    size 100px x 100px\n    circle center at (50px, 50px)\n        radius 20px\n        fill white\n",
        );
        assert!(doc.shapes().is_empty());
    }
    #[test]
    fn lowers_group_definitions_and_instances() {
        let doc = lower(
            "size 200px x 100px\nplace \"Dot\" at (10px, 20px)\n    scale 0.5\n    opacity 0.5\n    z-index 7\ngroup \"Dot\"\n    size 20px x 20px\n    rectangle at (0px, 0px)\n        size 20px x 20px\n        fill red\n",
        );
        assert_eq!(doc.groups.len(), 1);
        assert_eq!(doc.instances().len(), 1);
        assert_eq!(doc.groups[0].name, "Dot");
        assert_eq!(
            (doc.groups[0].bounds.width, doc.groups[0].bounds.height),
            (20.0, 20.0)
        );
        assert_eq!(doc.groups[0].children.len(), 1);
        assert!(matches!(
            doc.groups[0].children[0],
            CompositionChild::Shape(_)
        ));
        let instance = &doc.instances()[0];
        assert_eq!(instance.group, "Dot");
        assert_eq!((instance.x, instance.y), (10.0, 20.0));
        assert_eq!(instance.opacity, 0.5);
        assert_eq!(instance.z_index, 7);
    }

    #[test]
    fn lowers_root_children_in_source_order() {
        let doc = lower(
            "size 100px x 100px\nrectangle at (0px, 0px)\n    size 10px x 10px\n    fill red\nplace \"Dot\" at (0px, 0px)\n    z-index 4\ncircle center at (50px, 50px)\n    radius 5px\n    fill blue\ngroup \"Dot\"\n    size 10px x 10px\n",
        );
        use rupa_ir::document::CompositionChild;
        assert_eq!(doc.root.children.len(), 3);
        assert!(matches!(doc.root.children[0], CompositionChild::Shape(_)));
        assert!(matches!(
            doc.root.children[1],
            CompositionChild::Instance(_)
        ));
        assert!(matches!(doc.root.children[2], CompositionChild::Shape(_)));
        assert_eq!(doc.root.children[1].z_index(), 4);
    }

    #[test]
    fn lowers_nested_group_children_in_source_order() {
        let doc = lower(
            "size 100px x 100px\ngroup \"Outer\"\n    size 50px x 50px\n    place \"Inner\" at (5px, 5px)\n        z-index 3\n    rectangle at (0px, 0px)\n        size 10px x 10px\n        fill red\ngroup \"Inner\"\n    size 10px x 10px\n    circle center at (5px, 5px)\n        radius 5px\n        fill blue\n",
        );
        let outer = doc
            .groups
            .iter()
            .find(|group| group.name == "Outer")
            .expect("Outer");
        assert_eq!(outer.children.len(), 2);
        assert!(matches!(outer.children[0], CompositionChild::Instance(_)));
        assert!(matches!(outer.children[1], CompositionChild::Shape(_)));
        assert_eq!(outer.children[0].z_index(), 3);
    }

    #[test]
    fn group_without_size_uses_measured_child_bounds() {
        let doc = lower(
            "size 100px x 100px\ngroup \"Dot\"\n    circle center at (10px, 10px)\n        radius 5px\n        fill red\n",
        );
        let group = &doc.groups[0];
        assert_eq!((group.bounds.x, group.bounds.y), (5.0, 5.0));
        assert_eq!((group.bounds.width, group.bounds.height), (10.0, 10.0));
        assert!(!group.clip, "automatic bounds never clip");
    }

    #[test]
    fn group_with_declared_size_anchors_bounds_at_origin() {
        let doc = lower(
            "size 100px x 100px\ngroup \"Dot\"\n    size 40px x 20px\n    circle center at (10px, 10px)\n        radius 5px\n        fill red\n",
        );
        let group = &doc.groups[0];
        assert_eq!((group.bounds.x, group.bounds.y), (0.0, 0.0));
        assert_eq!((group.bounds.width, group.bounds.height), (40.0, 20.0));
        assert!(group.clip, "declared size clips");
    }

    #[test]
    fn layer_without_size_defaults_to_document_canvas() {
        let doc = lower("size 120px x 80px\nlayer \"Sky\"\n    z-index 2\n");
        let layer = doc
            .layers
            .iter()
            .find(|layer| layer.name == "Sky")
            .expect("Sky");
        assert_eq!((layer.width, layer.height), (120, 80));
    }

    #[test]
    fn layer_background_fill_prepends_a_full_bounds_rectangle() {
        let doc = lower(
            "size 120px x 80px\nlayer \"Sky\"\n    fill red\n    circle center at (10px, 10px)\n        radius 5px\n        fill white\n",
        );
        let layer = &doc.layers[0];
        assert_eq!(layer.children.len(), 2);
        match &layer.children[0] {
            CompositionChild::Shape(shape) => {
                assert!(matches!(
                    shape.geometry,
                    Geometry::Rectangle(IrRectangle {
                        x: 0.0,
                        y: 0.0,
                        width: 120.0,
                        height: 80.0,
                    })
                ));
                assert_eq!(
                    doc.paints.get(shape.fill.expect("background fill")),
                    Some(&Paint::Solid(Color::from_rgb8(255, 0, 0)))
                );
            }
            _ => panic!("expected background rectangle first"),
        }
        assert!(matches!(layer.children[1], CompositionChild::Shape(_)));
    }

    #[test]
    fn empty_layer_lowers_to_an_empty_plane() {
        let doc = lower("size 50px x 50px\nlayer \"Empty\"\n    z-index 1\n");
        let layer = &doc.layers[0];
        assert!(layer.children.is_empty());
        assert!(doc.root.children.iter().any(
            |child| matches!(child, CompositionChild::Plane(plane) if plane.layer == "Empty")
        ));
    }

    #[test]
    fn group_placement_before_definition_resolves_by_name() {
        let doc = lower(
            "size 100px x 100px\nplace \"Dot\" at (0px, 0px)\n    z-index 0\ngroup \"Dot\"\n    size 10px x 10px\n",
        );
        assert_eq!(doc.instances().len(), 1);
    }
}
