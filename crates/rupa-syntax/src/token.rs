//! Token definitions for Rupa lexer.

use crate::intern::IStr;
use crate::span::Span;
use serde::{Deserialize, Serialize};

/// Token kind
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TokenKind {
    // Keywords
    /// `size` keyword
    Size,
    /// `background` keyword
    Background,
    /// `rectangle` keyword
    Rectangle,
    /// `circle` keyword
    Circle,
    /// `at` keyword
    At,
    /// `center` keyword
    Center,
    /// `radius` keyword
    Radius,
    /// `fill` keyword
    Fill,
    /// `stroke` keyword
    Stroke,
    /// `x` keyword
    X,
    /// `layer` keyword
    Layer,
    /// `group` keyword
    Group,
    /// `place` keyword
    Place,
    /// `z-index` keyword
    ZIndex,
    /// `scale` keyword
    Scale,
    /// `rotate` keyword
    Rotate,
    /// `opacity` keyword
    Opacity,
    /// `repeat` keyword
    Repeat,
    /// `times` keyword
    Times,
    /// `as` keyword
    As,
    /// `let` keyword
    Let,
    /// `in` keyword
    In,
    /// `gradient` keyword
    Gradient,
    /// `pattern` keyword
    Pattern,
    /// `radial` keyword
    Radial,
    /// `from` keyword
    From,
    /// `to` keyword
    To,
    /// `angle` keyword
    Angle,
    /// `stop` keyword
    Stop,
    /// `picture` keyword
    Picture,
    /// `fit` keyword
    Fit,
    /// `contain` keyword
    Contain,
    /// `cover` keyword
    Cover,
    /// `none` keyword
    None_,
    /// `text` keyword
    Text,
    /// `font` keyword
    Font,
    /// `weight` keyword
    Weight,
    /// `align` keyword
    Align,
    /// `bold` keyword
    Bold,
    /// `normal` keyword
    Normal,
    /// `left` keyword
    Left,
    /// `right` keyword
    Right,
    /// `center` keyword (alternative)
    Center_,
    /// `raster` keyword
    Raster,
    /// `mode` keyword
    Mode,
    /// `clear` keyword
    Clear,
    /// `pixel` keyword
    Pixel,
    /// `brush` keyword
    Brush,
    /// `path` keyword
    Path,
    /// `start` keyword
    Start,
    /// `line` keyword
    Line,
    /// `curve` keyword
    Curve,
    /// `quad` keyword
    Quad,
    /// `close` keyword
    Close,
    /// `to` keyword (alternative)
    To_,
    /// `control` keyword
    Control,
    /// `and` keyword (declaration or plain keyword)
    And,
    /// `ellipse` keyword
    Ellipse,
    /// `square` keyword
    Square,
    /// `triangle` keyword
    Triangle,
    /// `polygon` keyword
    Polygon,
    /// `points` keyword
    Points,
    /// `line` keyword (alternative)
    Line_,
    /// `polyline` keyword
    Polyline,
    /// `width` keyword
    Width,
    /// `cap` keyword
    Cap,
    /// `join` keyword
    Join,
    /// `dash` keyword
    Dash,
    /// `miter` keyword
    Miter,
    /// `round` keyword
    Round,
    /// `square` keyword (alternative)
    Square_,
    /// `butt` keyword
    Butt,
    /// `bevel` keyword
    Bevel,
    /// `miter-limit` keyword
    MiterLimit,
    /// `effect` keyword
    Effect,
    /// `blur` keyword
    Blur,
    /// `brightness` keyword
    Brightness,
    /// `contrast` keyword
    Contrast,
    /// `saturation` keyword
    Saturation,
    /// `grayscale` keyword
    Grayscale,
    /// `hue-rotate` keyword
    HueRotate,
    /// `sharpen` keyword
    Sharpen,
    /// `noise` keyword
    Noise,
    /// `pixelate` keyword
    Pixelate,
    /// `vignette` keyword
    Vignette,
    /// `displacement-map` keyword
    DisplacementMap,
    /// `clip` keyword
    Clip,
    /// `blend` keyword
    Blend,
    /// `true` keyword
    True,
    /// `false` keyword
    False,
    /// `fn` keyword
    Fn,
    /// `return` keyword
    Return,
    /// `random` keyword
    Random,
    /// `seed` keyword
    Seed,

    // Identifiers and literals
    /// Identifier
    Identifier(IStr),
    /// String literal (unique payload, never interned)
    String(Box<str>),
    /// Number literal
    Number(f64),

    // Units
    /// Unit suffix
    Unit(Unit),

    // Operators and punctuation
    /// `+` operator
    Plus,
    /// `-` operator
    Minus,
    /// `*` operator
    Star,
    /// `/` operator
    Slash,
    /// `%` operator
    Percent,
    /// `==` operator
    Eq,
    /// `!=` operator
    Ne,
    /// `<` operator
    Lt,
    /// `<=` operator
    Le,
    /// `>` operator
    Gt,
    /// `>=` operator
    Ge,
    /// `and` keyword operator
    AndKw,
    /// `or` keyword operator
    OrKw,
    /// `not` keyword operator
    NotKw,
    /// `(` punctuation
    LParen,
    /// `)` punctuation
    RParen,
    /// `{` punctuation
    LBrace,
    /// `}` punctuation
    RBrace,
    /// `[` punctuation
    LBracket,
    /// `]` punctuation
    RBracket,
    /// `,` punctuation
    Comma,
    /// `:` punctuation
    Colon,
    /// `=` assignment operator
    Assign,
    /// `;` punctuation
    Semicolon,
    /// `=>` punctuation
    Arrow,
    /// `@` punctuation
    AtSign,
    /// `#` punctuation
    Hash,

    // Indentation
    /// Indentation increase
    Indent,
    /// Indentation decrease
    Dedent,
    /// Newline
    Newline,
    /// End of file
    Eof,
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

/// Token with span
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Token {
    /// Token kind
    pub kind: TokenKind,
    /// Source span
    pub span: Span,
}

impl Token {
    /// Create a new token
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }

    /// Create an EOF token
    pub fn eof(span: Span) -> Self {
        Self::new(TokenKind::Eof, span)
    }

    /// Check if token is a keyword
    pub fn is_keyword(&self) -> bool {
        matches!(
            self.kind,
            TokenKind::Size
                | TokenKind::Background
                | TokenKind::Rectangle
                | TokenKind::Circle
                | TokenKind::At
                | TokenKind::Center
                | TokenKind::Radius
                | TokenKind::Fill
                | TokenKind::Stroke
                | TokenKind::X
                | TokenKind::Layer
                | TokenKind::Place
                | TokenKind::ZIndex
                | TokenKind::Scale
                | TokenKind::Rotate
                | TokenKind::Opacity
                | TokenKind::Repeat
                | TokenKind::Times
                | TokenKind::As
                | TokenKind::Let
                | TokenKind::In
                | TokenKind::Gradient
                | TokenKind::From
                | TokenKind::To
                | TokenKind::Angle
                | TokenKind::Stop
                | TokenKind::Picture
                | TokenKind::Fit
                | TokenKind::Contain
                | TokenKind::Cover
                | TokenKind::None_
                | TokenKind::Text
                | TokenKind::Font
                | TokenKind::Weight
                | TokenKind::Align
                | TokenKind::Bold
                | TokenKind::Normal
                | TokenKind::Left
                | TokenKind::Right
                | TokenKind::Center_
                | TokenKind::Raster
                | TokenKind::Mode
                | TokenKind::Clear
                | TokenKind::Pixel
                | TokenKind::Brush
                | TokenKind::Path
                | TokenKind::Start
                | TokenKind::Line
                | TokenKind::Curve
                | TokenKind::Quad
                | TokenKind::Close
                | TokenKind::To_
                | TokenKind::Control
                | TokenKind::And
                | TokenKind::Ellipse
                | TokenKind::Square
                | TokenKind::Triangle
                | TokenKind::Polygon
                | TokenKind::Points
                | TokenKind::Line_
                | TokenKind::Polyline
                | TokenKind::Width
                | TokenKind::Cap
                | TokenKind::Join
                | TokenKind::Dash
                | TokenKind::Miter
                | TokenKind::Round
                | TokenKind::Square_
                | TokenKind::Butt
                | TokenKind::Bevel
                | TokenKind::MiterLimit
                | TokenKind::Effect
                | TokenKind::Blur
                | TokenKind::Brightness
                | TokenKind::Contrast
                | TokenKind::Saturation
                | TokenKind::Grayscale
                | TokenKind::HueRotate
                | TokenKind::Sharpen
                | TokenKind::Noise
                | TokenKind::Pixelate
                | TokenKind::Vignette
                | TokenKind::DisplacementMap
                | TokenKind::Clip
                | TokenKind::True
                | TokenKind::False
                | TokenKind::Fn
                | TokenKind::Return
                | TokenKind::Random
                | TokenKind::Seed
        )
    }
}
