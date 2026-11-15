//! Abstract Syntax Tree for Rupa.

use crate::intern::IStr;
use crate::span::Span;
use serde::{Deserialize, Serialize};

/// Root document node
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    /// Source span
    pub span: Span,
    /// Document size
    pub size: Option<Size>,
    /// Background color
    pub background: Option<Background>,
    /// Document random seed
    pub seed: Option<u64>,
    /// Child elements (shapes) drawn at document root
    pub children: Vec<DocumentChild>,
    /// Reusable group definitions (drawn only when placed)
    pub groups: Vec<GroupDefinition>,
    /// Layer definitions (composited planes)
    pub layers: Vec<LayerDefinition>,
    /// Function definitions available to expressions in this document
    pub functions: Vec<Function>,
}

/// Rendering mode for a reusable layer definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LayerMode {
    /// Vector composition mode (default).
    #[default]
    Vector,
    /// Raster surface mode.
    Raster,
}

/// Reusable group definition.
///
/// A group is a visual template with its own local coordinate system. It draws
/// only when placed, and may be placed any number of times (specification §6.2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupDefinition {
    /// Group name (case-sensitive, non-empty)
    pub name: IStr,
    /// Declared size; omitted means the automatic bounding box of the children.
    pub size: Option<Size>,
    /// Shape and nested placement children in group-local coordinates
    pub children: Vec<DocumentChild>,
    /// Source span
    pub span: Span,
}

/// Layer definition: a single-use composition plane.
///
/// A layer has its own local coordinate system, a declared size, and shape
/// children. It is composited once, at its `z-index` (specification §6.4).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerDefinition {
    /// Layer name (case-sensitive, non-empty)
    pub name: IStr,
    /// Declared layer size, if provided; omitted means the document canvas.
    pub size: Option<Size>,
    /// Background fill shorthand: paints the layer bounds before its children.
    pub fill: Option<Fill>,
    /// Rendering mode; omitted means vector mode.
    pub mode: Option<LayerMode>,
    /// Whether layer content is clipped to its declared bounds.
    pub clip: Option<bool>,
    /// Ordinal against sibling composition planes; omitted means `0`.
    pub z_index: Option<f64>,
    /// Opacity of the composited plane; omitted means `1.0`.
    pub opacity: Option<f64>,
    /// Blend mode of the composited plane.
    pub blend: Option<IStr>,
    /// Ordered filter effects applied to the rendered layer.
    pub effects: Vec<LayerEffect>,
    /// Shape and nested placement children in layer-local coordinates
    pub children: Vec<DocumentChild>,
    /// Source span
    pub span: Span,
}

/// A filter effect applied to a layer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerEffect {
    /// Filter kind name.
    pub kind: IStr,
    /// Optional numeric value for the filter.
    pub value: Option<Dimension>,
    /// Optional resource path for filters that need one.
    pub resource: Option<Box<str>>,
    /// Source span.
    pub span: Span,
}

/// Document size declaration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Size {
    /// Width dimension
    pub width: Dimension,
    /// Height dimension
    pub height: Dimension,
    /// Source span
    pub span: Span,
}

/// Background declaration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Background {
    /// Background color
    pub color: Color,
    /// Source span
    pub span: Span,
}

/// Dimension with unit
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dimension {
    /// Numeric value
    pub value: f64,
    /// Unit of measurement
    pub unit: Unit,
    /// Source span
    pub span: Span,
}

/// Units supported in Rupa
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Unit {
    /// Pixels
    Px,
    /// Points
    Pt,
    /// Percentage
    Percent,
    /// Degrees
    Deg,
}

/// Color representation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Color {
    /// Named color (e.g., "red")
    Named(IStr),
    /// Hex color (e.g., "#ff0000")
    Hex(IStr),
    /// RGB color
    Rgb(u8, u8, u8),
    /// RGBA color
    Rgba(u8, u8, u8, u8),
}

/// Document children (shapes, layers, etc.)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DocumentChild {
    /// Rectangle primitive
    Rectangle(Rectangle),
    /// Circle primitive
    Circle(Circle),
    /// Ellipse primitive
    Ellipse(Ellipse),
    /// Square primitive
    Square(Square),
    /// Triangle primitive
    Triangle(Triangle),
    /// Polygon primitive
    Polygon(Polygon),
    /// Line primitive
    Line(Line),
    /// Polyline primitive
    Polyline(Polyline),
    /// Path primitive
    Path(Path),
    /// External image resource
    Picture(Picture),
    /// Text primitive
    Text(Text),
    /// Group instance placement
    Place(GroupInstance),
    /// Raster clear operation
    Clear(Clear),
    /// Raster pixel operation
    Pixel(Pixel),
    /// Raster brush stroke operation
    Brush(Brush),
    /// Variable declaration (`let name = value`)
    Let(Variable),
    /// Counted repeat loop
    Repeat(Repeat),
}

/// Rectangle primitive.
///
/// `position` is the top-left corner of the rectangle (specification §7.1).
/// Required properties are optional in the AST so that the compiler can report
/// missing properties as semantic errors instead of dropping the shape at
/// parse time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rectangle {
    /// Top-left corner of the rectangle, if declared
    pub position: Option<Position>,
    /// Size of rectangle, if declared
    pub size: Option<Size>,
    /// Optional fill paint
    pub fill: Option<Fill>,
    /// Optional stroke declaration
    pub stroke: Option<Stroke>,
    /// Optional stroke width
    pub width: Option<Dimension>,
    /// Optional transform/opacity properties
    pub transform: Option<Transform>,
    /// Source span
    pub span: Span,
}

/// Circle primitive.
///
/// `center` is the circle's center (specification §7.1). Required properties
/// are optional in the AST so that the compiler can report missing properties
/// as semantic errors.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Circle {
    /// Center position, if declared
    pub center: Option<Position>,
    /// Radius dimension, if declared
    pub radius: Option<Dimension>,
    /// Optional fill paint
    pub fill: Option<Fill>,
    /// Optional stroke declaration
    pub stroke: Option<Stroke>,
    /// Optional stroke width
    pub width: Option<Dimension>,
    /// Optional transform/opacity properties
    pub transform: Option<Transform>,
    /// Source span
    pub span: Span,
}

/// Ellipse primitive.
///
/// `center` is the ellipse's center; `size` is its full width and height
/// (specification §7.1). Required properties are optional so the compiler can
/// report missing values as semantic errors.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ellipse {
    /// Center position, if declared
    pub center: Option<Position>,
    /// Full width and height, if declared
    pub size: Option<Size>,
    /// Optional fill paint
    pub fill: Option<Fill>,
    /// Optional stroke declaration
    pub stroke: Option<Stroke>,
    /// Optional stroke width
    pub width: Option<Dimension>,
    /// Optional transform/opacity properties
    pub transform: Option<Transform>,
    /// Source span
    pub span: Span,
}

/// Square primitive.
///
/// `position` is the top-left corner and `size` is a single side length
/// (specification §7.1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Square {
    /// Top-left corner, if declared
    pub position: Option<Position>,
    /// Side length, if declared
    pub size: Option<Dimension>,
    /// Optional fill paint
    pub fill: Option<Fill>,
    /// Optional stroke declaration
    pub stroke: Option<Stroke>,
    /// Optional stroke width
    pub width: Option<Dimension>,
    /// Optional transform/opacity properties
    pub transform: Option<Transform>,
    /// Source span
    pub span: Span,
}

/// Triangle primitive.
///
/// `position` is the top-left of the bounding box and `size` is the bounding
/// box width and height (specification §7.1). The triangle shape convention is
/// recorded in DECISION-009.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Triangle {
    /// Bounding-box top-left corner, if declared
    pub position: Option<Position>,
    /// Bounding-box size, if declared
    pub size: Option<Size>,
    /// Optional fill paint
    pub fill: Option<Fill>,
    /// Optional stroke declaration
    pub stroke: Option<Stroke>,
    /// Optional stroke width
    pub width: Option<Dimension>,
    /// Optional transform/opacity properties
    pub transform: Option<Transform>,
    /// Source span
    pub span: Span,
}

/// Polygon primitive.
///
/// `points` is the ordered list of vertices; `None` means the `points` block
/// was omitted. A polygon always closes from the last vertex to the first
/// (specification §7.2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Polygon {
    /// Ordered vertices, if the `points` block was declared
    pub points: Option<Vec<Position>>,
    /// Optional fill paint
    pub fill: Option<Fill>,
    /// Optional stroke declaration
    pub stroke: Option<Stroke>,
    /// Optional stroke width
    pub width: Option<Dimension>,
    /// Optional transform/opacity properties
    pub transform: Option<Transform>,
    /// Source span
    pub span: Span,
}

/// External raster image placement (specification §10).
///
/// `at` is the top-left of the picture frame; `size` is the optional frame
/// size. `None` means the frame takes the image's natural pixel size. `fit`
/// controls how the image maps into the frame and defaults to `contain`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Picture {
    /// Resource path as written in the source
    pub path: Box<str>,
    /// Frame top-left corner, if declared
    pub position: Option<Position>,
    /// Frame size, if declared
    pub size: Option<Size>,
    /// Fit mode, if declared
    pub fit: Option<FitMode>,
    /// Optional transform/opacity properties
    pub transform: Option<Transform>,
    /// Source span
    pub span: Span,
}

/// How an image is mapped into its picture frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FitMode {
    /// Scale to fit inside the frame, preserving aspect ratio, centered
    Contain,
    /// Scale to cover the frame, preserving aspect ratio, center-cropped
    Cover,
    /// Stretch to the frame exactly
    Fill,
    /// Draw at natural size from the frame's top-left corner
    None,
}

/// Text primitive (specification §9).
///
/// `position` is the baseline origin when `centered` is false; `center` is
/// used for centered text. `fill` defaults to black and `size` to 16px when
/// omitted by the compiler.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Text {
    /// Text content (unique payload, stored once)
    pub text: Box<str>,
    /// Baseline origin, if declared with `at`
    pub position: Option<Position>,
    /// Center anchor, if declared with `center at`
    pub center: Option<Position>,
    /// Font family or resource name, if declared
    pub font: Option<IStr>,
    /// Font size dimension, if declared
    pub size: Option<Dimension>,
    /// Font weight, if declared
    pub weight: Option<FontWeight>,
    /// Optional fill paint
    pub fill: Option<Fill>,
    /// Text alignment, if declared
    pub align: Option<TextAlign>,
    /// Optional transform/opacity properties
    pub transform: Option<Transform>,
    /// Source span
    pub span: Span,
}

/// Font weight for text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FontWeight {
    /// Normal weight
    Normal,
    /// Bold weight
    Bold,
}

/// Text alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextAlign {
    /// Left alignment
    Left,
    /// Center alignment
    Center,
    /// Right alignment
    Right,
}

/// Layer instance placement.
///
/// Instances reference a definition by name and apply independent position,
/// transform, opacity, and z-index (specification §6.2). They never mutate the
/// shared definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupInstance {
    /// Referenced group definition name
    pub name: IStr,
    /// Placement of the layer's top-left local origin, if declared
    pub position: Option<Position>,
    /// Optional transform properties
    pub transform: Option<Transform>,
    /// Optional integer z-index
    pub z_index: Option<f64>,
    /// Optional blend mode
    pub blend: Option<IStr>,
    /// Source span
    pub span: Span,
}

/// Raster clear operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Clear {
    /// Color used to clear the surface
    pub color: Color,
    /// Source span
    pub span: Span,
}

/// Raster pixel operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pixel {
    /// Pixel position, if declared
    pub position: Option<Position>,
    /// Fill color, if declared
    pub fill: Option<Fill>,
    /// Source span
    pub span: Span,
}

/// Raster brush stroke operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Brush {
    /// Brush size (diameter), if declared
    pub size: Option<Dimension>,
    /// Fill color, if declared
    pub fill: Option<Fill>,
    /// Stroke points, if declared
    pub points: Vec<Position>,
    /// Source span
    pub span: Span,
}

/// Path primitive with an ordered command sequence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Path {
    /// Commands in source order
    pub commands: Vec<PathCommand>,
    /// Optional fill paint
    pub fill: Option<Fill>,
    /// Optional stroke declaration
    pub stroke: Option<Stroke>,
    /// Optional stroke width
    pub width: Option<Dimension>,
    /// Optional transform/opacity properties
    pub transform: Option<Transform>,
    /// Source span
    pub span: Span,
}

/// Path command.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PathCommand {
    /// Begin a subpath at a point
    Start {
        /// Starting point
        position: Position,
        /// Source span
        span: Span,
    },
    /// Straight segment to a point
    LineTo {
        /// End point
        to: Position,
        /// Source span
        span: Span,
    },
    /// Cubic Bézier segment
    CurveTo {
        /// End point
        to: Position,
        /// First control point
        control1: Position,
        /// Second control point
        control2: Position,
        /// Source span
        span: Span,
    },
    /// Quadratic Bézier segment
    QuadTo {
        /// End point
        to: Position,
        /// Control point
        control: Position,
        /// Source span
        span: Span,
    },
    /// Close the current subpath
    Close {
        /// Source span
        span: Span,
    },
}

/// Primitive transform properties.
///
/// Missing properties use the documented defaults: no rotation, scale 1.0,
/// opacity 1.0. The renderer applies these around the primitive's geometric
/// center (specification §11).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    /// Rotation dimension; must use degrees
    pub rotate: Option<Dimension>,
    /// Scale factors, if declared
    pub scale: Option<Scale>,
    /// Opacity in `0.0..=1.0`
    pub opacity: Option<f64>,
    /// Source span
    pub span: Span,
}

/// Scale factors for a transform.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scale {
    /// Horizontal scale
    pub x: f64,
    /// Vertical scale
    pub y: f64,
    /// Source span
    pub span: Span,
}

/// Stroke paint declaration.
///
/// `color` is `None` for an explicit `stroke none`. Width is declared
/// separately with `width <dimension>`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stroke {
    /// Stroke color, or `None` for `stroke none`
    pub color: Option<Color>,
    /// Line cap style, if declared
    pub cap: Option<StrokeCap>,
    /// Line join style, if declared
    pub join: Option<StrokeJoin>,
    /// Source span
    pub span: Span,
}

/// Line cap style for stroked geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrokeCap {
    /// Flat cap ending at the endpoint
    Butt,
    /// Semicircular cap
    Round,
    /// Square cap extending half the width
    Square,
}

/// Line join style for stroked geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrokeJoin {
    /// Sharp miter join
    Miter,
    /// Rounded join
    Round,
    /// Beveled join
    Bevel,
}

/// Line primitive with explicit endpoints.
///
/// Lines have no fill; they are drawn with a stroke (specification §7.3).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Line {
    /// Start endpoint, if declared
    pub from: Option<Position>,
    /// End endpoint, if declared
    pub to: Option<Position>,
    /// Optional stroke declaration
    pub stroke: Option<Stroke>,
    /// Optional stroke width
    pub width: Option<Dimension>,
    /// Optional transform/opacity properties
    pub transform: Option<Transform>,
    /// Source span
    pub span: Span,
}

/// Open polyline primitive.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Polyline {
    /// Ordered points, if the `points` block was declared
    pub points: Option<Vec<Position>>,
    /// Optional stroke declaration
    pub stroke: Option<Stroke>,
    /// Optional stroke width
    pub width: Option<Dimension>,
    /// Optional transform/opacity properties
    pub transform: Option<Transform>,
    /// Source span
    pub span: Span,
}

/// Position (at or center at)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Position {
    /// X coordinate
    pub x: Dimension,
    /// Y coordinate
    pub y: Dimension,
    /// Source span
    pub span: Span,
}

/// Fill paint declaration.
///
/// `color` is `None` for an explicit `fill none`, which disables fill paint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fill {
    /// Fill color, or `None` for `fill none`
    pub color: Option<Color>,
    /// Gradient fill, when `fill gradient` is used.
    pub gradient: Option<Box<GradientFill>>,
    /// Pattern fill, when `fill pattern` is used.
    pub pattern: Option<Box<PatternFill>>,
    /// Source span
    pub span: Span,
}

/// A pattern fill declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatternFill {
    /// Pattern kind: dots, grid, stripes, checkerboard.
    pub kind: IStr,
    /// Foreground color.
    pub color: Option<Color>,
    /// Background color.
    pub background: Option<Color>,
    /// Tile size.
    pub size: Option<Dimension>,
    /// Pattern angle.
    pub angle: Option<Dimension>,
    /// Source span.
    pub span: Span,
}

/// A single stop in a gradient fill.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GradientStop {
    /// Offset from 0 to 100 percent.
    pub offset: Dimension,
    /// Stop color.
    pub color: Color,
    /// Source span.
    pub span: Span,
}

/// A linear gradient fill declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GradientFill {
    /// Whether this is a radial gradient.
    pub radial: bool,
    /// Starting color for `from`/`to` shorthand.
    pub from: Option<Color>,
    /// Ending color for `from`/`to` shorthand.
    pub to: Option<Color>,
    /// Explicit gradient stops.
    pub stops: Vec<GradientStop>,
    /// Gradient angle.
    pub angle: Option<Dimension>,
    /// Source span.
    pub span: Span,
}

/// A counted repeat loop.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Repeat {
    /// Number of iterations
    pub count: Expression,
    /// Optional loop index variable name
    pub index: Option<IStr>,
    /// Repeated body children
    pub body: Vec<DocumentChild>,
    /// Source span
    pub span: Span,
}

/// A user-defined function declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Function {
    /// Function name
    pub name: IStr,
    /// Parameter names
    pub params: Vec<IStr>,
    /// Returned expression
    pub body: Expression,
    /// Source span
    pub span: Span,
}

/// A variable declaration (`let name = value`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variable {
    /// Variable name
    pub name: IStr,
    /// Declared value
    pub value: Expression,
    /// Source span
    pub span: Span,
}

/// Binary operator for expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOp {
    /// Addition
    Add,
    /// Subtraction
    Sub,
    /// Multiplication
    Mul,
    /// Division
    Div,
    /// Modulo
    Mod,
    /// Equality
    Eq,
    /// Inequality
    Ne,
    /// Less than
    Lt,
    /// Less than or equal
    Le,
    /// Greater than
    Gt,
    /// Greater than or equal
    Ge,
    /// Logical and
    And,
    /// Logical or
    Or,
}

/// Unary operator for expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOp {
    /// Numeric negation
    Neg,
    /// Logical not
    Not,
}

/// Minimal expression/value representation for variable declarations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    /// Numeric literal without a unit
    Number(f64),
    /// Dimensional value with a unit
    Dimension(Dimension),
    /// String literal (unique payload, never interned)
    String(Box<str>),
    /// Boolean literal
    Bool(bool),
    /// Identifier reference or named value
    Identifier(IStr),
    /// Coordinate pair
    Position(Box<Position>),
    /// Binary operation
    Binary {
        /// Operator
        op: BinaryOp,
        /// Left operand
        left: Box<Expression>,
        /// Right operand
        right: Box<Expression>,
        /// Source span
        span: Span,
    },
    /// Unary operation
    Unary {
        /// Operator
        op: UnaryOp,
        /// Operand
        expr: Box<Expression>,
        /// Source span
        span: Span,
    },
    /// Function call
    Call {
        /// Function name
        name: IStr,
        /// Arguments
        args: Vec<Expression>,
        /// Source span
        span: Span,
    },
}

/// Runtime value produced by expression evaluation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    /// Numeric value
    Number(f64),
    /// Dimensional value
    Dimension(Dimension),
    /// String value
    String(Box<str>),
    /// Boolean value
    Bool(bool),
    /// Coordinate pair
    Position(Position),
}

/// Alias for a variable declaration.
pub type VariableDeclaration = Variable;
/// Alias for a variable declaration.
pub type LetBinding = Variable;
/// Alias for a variable declaration.
pub type LetStatement = Variable;
/// Alias for an expression.
pub type Expr = Expression;

impl Expression {
    /// Evaluate the expression using the supplied variable environment, function
    /// definitions, and PRNG.
    pub fn evaluate(
        &self,
        env: &std::collections::HashMap<IStr, Value>,
        functions: &std::collections::HashMap<IStr, Function>,
        rng: &mut crate::Prng,
    ) -> Result<Value, String> {
        self.evaluate_with(env, functions, rng, 0)
    }

    /// Evaluate with an explicit function-call depth for recursion protection.
    pub fn evaluate_with(
        &self,
        env: &std::collections::HashMap<IStr, Value>,
        functions: &std::collections::HashMap<IStr, Function>,
        rng: &mut crate::Prng,
        depth: usize,
    ) -> Result<Value, String> {
        match self {
            Expression::Number(n) => Ok(Value::Number(*n)),
            Expression::Dimension(d) => Ok(Value::Dimension(d.clone())),
            Expression::String(s) => Ok(Value::String(s.clone())),
            Expression::Bool(b) => Ok(Value::Bool(*b)),
            Expression::Identifier(name) => env
                .get(name)
                .cloned()
                .ok_or_else(|| format!("unknown variable `{name}`")),
            Expression::Position(p) => Ok(Value::Position((**p).clone())),
            Expression::Unary { op, expr, .. } => {
                let value = expr.evaluate_with(env, functions, rng, depth)?;
                match (op, value) {
                    (UnaryOp::Neg, Value::Number(n)) => Ok(Value::Number(-n)),
                    (UnaryOp::Neg, Value::Dimension(d)) => Ok(Value::Dimension(Dimension {
                        value: -d.value,
                        unit: d.unit,
                        span: d.span,
                    })),
                    (UnaryOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
                    _ => Err("invalid unary operation".to_string()),
                }
            }
            Expression::Binary {
                op, left, right, ..
            } => {
                let l = left.evaluate_with(env, functions, rng, depth)?;
                let r = right.evaluate_with(env, functions, rng, depth)?;
                match op {
                    BinaryOp::Add => match (l, r) {
                        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                        (Value::Dimension(a), Value::Dimension(b)) if a.unit == b.unit => {
                            Ok(Value::Dimension(Dimension {
                                value: a.value + b.value,
                                unit: a.unit,
                                span: a.span,
                            }))
                        }
                        _ => Err("invalid addition operands".to_string()),
                    },
                    BinaryOp::Sub => match (l, r) {
                        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
                        (Value::Dimension(a), Value::Dimension(b)) if a.unit == b.unit => {
                            Ok(Value::Dimension(Dimension {
                                value: a.value - b.value,
                                unit: a.unit,
                                span: a.span,
                            }))
                        }
                        _ => Err("invalid subtraction operands".to_string()),
                    },
                    BinaryOp::Mul => match (l, r) {
                        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
                        (Value::Dimension(a), Value::Number(b)) => {
                            Ok(Value::Dimension(Dimension {
                                value: a.value * b,
                                unit: a.unit,
                                span: a.span,
                            }))
                        }
                        (Value::Number(a), Value::Dimension(b)) => {
                            Ok(Value::Dimension(Dimension {
                                value: a * b.value,
                                unit: b.unit,
                                span: b.span,
                            }))
                        }
                        _ => Err("invalid multiplication operands".to_string()),
                    },
                    BinaryOp::Div => match (l, r) {
                        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a / b)),
                        (Value::Dimension(a), Value::Number(b)) => {
                            Ok(Value::Dimension(Dimension {
                                value: a.value / b,
                                unit: a.unit,
                                span: a.span,
                            }))
                        }
                        (Value::Dimension(a), Value::Dimension(b)) if a.unit == b.unit => {
                            Ok(Value::Number(a.value / b.value))
                        }
                        _ => Err("invalid division operands".to_string()),
                    },
                    BinaryOp::Mod => match (l, r) {
                        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a % b)),
                        _ => Err("invalid modulo operands".to_string()),
                    },
                    BinaryOp::Eq => Ok(Value::Bool(l == r)),
                    BinaryOp::Ne => Ok(Value::Bool(l != r)),
                    BinaryOp::Lt => match (l, r) {
                        (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a < b)),
                        (Value::Dimension(a), Value::Dimension(b)) if a.unit == b.unit => {
                            Ok(Value::Bool(a.value < b.value))
                        }
                        _ => Err("invalid comparison operands".to_string()),
                    },
                    BinaryOp::Le => match (l, r) {
                        (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a <= b)),
                        (Value::Dimension(a), Value::Dimension(b)) if a.unit == b.unit => {
                            Ok(Value::Bool(a.value <= b.value))
                        }
                        _ => Err("invalid comparison operands".to_string()),
                    },
                    BinaryOp::Gt => match (l, r) {
                        (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a > b)),
                        (Value::Dimension(a), Value::Dimension(b)) if a.unit == b.unit => {
                            Ok(Value::Bool(a.value > b.value))
                        }
                        _ => Err("invalid comparison operands".to_string()),
                    },
                    BinaryOp::Ge => match (l, r) {
                        (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a >= b)),
                        (Value::Dimension(a), Value::Dimension(b)) if a.unit == b.unit => {
                            Ok(Value::Bool(a.value >= b.value))
                        }
                        _ => Err("invalid comparison operands".to_string()),
                    },
                    BinaryOp::And => match (l, r) {
                        (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a && b)),
                        _ => Err("invalid logical and operands".to_string()),
                    },
                    BinaryOp::Or => match (l, r) {
                        (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a || b)),
                        _ => Err("invalid logical or operands".to_string()),
                    },
                }
            }
            Expression::Call { name, args, .. } => {
                if name.as_str() == "random" {
                    if args.len() != 2 {
                        return Err(format!("random expects 2 arguments, found {}", args.len()));
                    }
                    let min = args[0].evaluate_with(env, functions, rng, depth)?;
                    let max = args[1].evaluate_with(env, functions, rng, depth)?;
                    match (min, max) {
                        (Value::Number(a), Value::Number(b)) => {
                            Ok(Value::Number(rng.next_f64_range(a, b)))
                        }
                        (Value::Dimension(a), Value::Dimension(b)) if a.unit == b.unit => {
                            Ok(Value::Dimension(Dimension {
                                value: rng.next_f64_range(a.value, b.value),
                                unit: a.unit,
                                span: a.span,
                            }))
                        }
                        _ => Err("invalid random() arguments".to_string()),
                    }
                } else if let Some(function) = functions.get(name) {
                    if args.len() != function.params.len() {
                        return Err(format!(
                            "function `{name}` expects {} arguments, found {}",
                            function.params.len(),
                            args.len()
                        ));
                    }
                    if depth >= 64 {
                        return Err(format!("function `{name}` exceeded maximum call depth"));
                    }
                    let mut arg_values = Vec::with_capacity(args.len());
                    for arg in args {
                        arg_values.push(arg.evaluate_with(env, functions, rng, depth)?);
                    }
                    let mut call_scope = env.clone();
                    for (param, value) in function.params.iter().zip(arg_values) {
                        call_scope.insert(*param, value);
                    }
                    function
                        .body
                        .evaluate_with(&call_scope, functions, rng, depth + 1)
                } else {
                    let evaluated: Result<Vec<Value>, String> = args
                        .iter()
                        .map(|a| a.evaluate_with(env, functions, rng, depth))
                        .collect();
                    let _evaluated = evaluated?;
                    Err(format!("unknown function `{name}`"))
                }
            }
        }
    }
}
