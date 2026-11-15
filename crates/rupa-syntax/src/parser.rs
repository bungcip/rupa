//! Parser for Rupa.

use crate::ast::*;
use crate::diagnostic::{Diagnostic, DiagnosticBag, codes};
use crate::intern::IStr;
use crate::span::{FileId, Span};
use crate::token::{Token, TokenKind, Unit as TokenUnit};

/// Parser for Rupa source code
pub struct Parser {
    /// List of tokens
    tokens: Vec<Token>,
    /// Current token position
    pos: usize,
    /// Collected diagnostics
    diagnostics: DiagnosticBag,
}

/// Common shape properties parsed by every primitive that supports them.
#[derive(Default)]
struct ShapeProperties {
    fill: Option<Fill>,
    stroke: Option<Stroke>,
    width: Option<Dimension>,
    transform: Option<Transform>,
}

/// Where a drawing primitive's properties are being parsed from.
///
/// A primitive may write its properties inline on the header line, inside an
/// indented block, or inline first and then continue in an indented block
/// (specification §7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BodyState {
    /// Properties continue on the primitive's own line.
    Inline,
    /// Properties are inside an indented block.
    Block,
    /// No body follows; only primitives with optional properties use this.
    Empty,
}

impl Parser {
    /// Create a new parser
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            diagnostics: DiagnosticBag::new(),
        }
    }

    /// Parse tokens into AST.
    ///
    /// A `.rupa` file is exactly one document, so parsing always produces a
    /// `Document` (possibly with missing required settings, reported by
    /// semantic validation).
    pub fn parse(mut self) -> (Option<Document>, DiagnosticBag) {
        let mut document = Document {
            span: self.current_span(),
            size: None,
            background: None,
            seed: None,
            children: Vec::new(),
            groups: Vec::new(),
            layers: Vec::new(),
            functions: Vec::new(),
        };

        loop {
            while self.check(TokenKind::Newline) {
                self.advance();
            }
            if self.check_eof() {
                break;
            }

            if self.check(TokenKind::Size) {
                if document.size.is_some() {
                    self.duplicate_document_setting(codes::DUPLICATE_DOCUMENT_SIZE, "size");
                    self.parse_size();
                } else {
                    document.size = self.parse_size();
                }
            } else if self.check(TokenKind::Background) {
                if document.background.is_some() {
                    self.duplicate_document_setting(codes::DUPLICATE_BACKGROUND, "background");
                    self.parse_background();
                } else {
                    document.background = self.parse_background();
                }
            } else if self.check(TokenKind::Seed) {
                if document.seed.is_some() {
                    self.duplicate_document_setting(codes::DUPLICATE_SEED, "seed");
                    self.parse_seed();
                } else {
                    document.seed = self.parse_seed();
                }
            } else if self.check(TokenKind::Layer) {
                if let Some(layer) = self.parse_layer() {
                    document.layers.push(layer);
                }
            } else if self.check(TokenKind::Group) {
                if let Some(group) = self.parse_group() {
                    document.groups.push(group);
                }
            } else if self.check(TokenKind::Fn) {
                if let Some(function) = self.parse_function() {
                    document.functions.push(function);
                }
            } else if let Some(child) = self.parse_shared_child(false) {
                document.children.push(child);
            } else {
                let token = self.current_token().cloned();
                if let Some(tok) = token {
                    let diag = if let TokenKind::Identifier(name) = &tok.kind {
                        Diagnostic::error(
                            codes::UNKNOWN_KEYWORD,
                            format!("unknown keyword or property `{name}` at file level"),
                            tok.span,
                        )
                        .with_help("Expected size, background, seed, layer, group, place, rectangle, circle, ellipse, square, triangle, polygon, line, polyline, or path")
                        .with_suggestion(name)
                    } else {
                        Diagnostic::error(
                            codes::EXPECTED_TOKEN,
                            format!("Unexpected top-level token: {:?}", tok.kind),
                            tok.span,
                        )
                        .with_help(
                            "Expected size, background, seed, layer, place, or a drawing primitive",
                        )
                    };
                    self.diagnostics.push(diag);
                    self.advance();
                }
            }
        }

        (Some(document), self.diagnostics)
    }

    fn duplicate_document_setting(&mut self, code: &str, name: &str) {
        let span = self.current_span();
        self.diagnostics.push(
            Diagnostic::error(
                code,
                format!("duplicate document `{name}` declaration"),
                span,
            )
            .with_help("keep at most one declaration per document setting"),
        );
    }

    fn parse_place(&mut self) -> Option<GroupInstance> {
        let place_token = self.advance()?; // consume "place"
        let name = self.expect_istr()?;
        let position = self.parse_at_keyword();

        let mut instance = GroupInstance {
            name,
            position,
            transform: None,
            z_index: None,
            blend: None,
            span: place_token.span,
        };

        // A placement's properties are optional, so `place "Dot" at (0px, 0px)`
        // is valid on its own and a sibling declaration may follow directly.
        let mut body = self.enter_optional_body();

        while self.body_continue(&mut body) {
            if self.check(TokenKind::ZIndex) {
                self.advance();
                instance.z_index = self.parse_signed_number();
            } else if self.check(TokenKind::Blend) {
                self.advance(); // consume "blend"
                if let Some(TokenKind::Identifier(name)) =
                    self.current_token().map(|t| t.kind.clone())
                {
                    instance.blend = Some(name);
                    self.advance();
                }
            } else if !self.parse_transform_property(&mut instance.transform)? {
                self.push_shape_unexpected(
                    "place",
                    place_token.span,
                    "z-index, rotate, scale, or opacity",
                );
            }
        }
        if matches!(body, BodyState::Block) {
            self.expect_dedent()?;
        }
        Some(instance)
    }

    fn parse_child_shape(&mut self) -> Option<DocumentChild> {
        if self.check(TokenKind::Rectangle) {
            self.parse_rectangle().map(DocumentChild::Rectangle)
        } else if self.check(TokenKind::Circle) {
            self.parse_circle().map(DocumentChild::Circle)
        } else if self.check(TokenKind::Ellipse) {
            self.parse_ellipse().map(DocumentChild::Ellipse)
        } else if self.check(TokenKind::Square) {
            self.parse_square().map(DocumentChild::Square)
        } else if self.check(TokenKind::Triangle) {
            self.parse_triangle().map(DocumentChild::Triangle)
        } else if self.check(TokenKind::Polygon) {
            self.parse_polygon().map(DocumentChild::Polygon)
        } else if self.check(TokenKind::Line) {
            self.parse_line().map(DocumentChild::Line)
        } else if self.check(TokenKind::Polyline) {
            self.parse_polyline().map(DocumentChild::Polyline)
        } else if self.check(TokenKind::Path) {
            self.parse_path().map(DocumentChild::Path)
        } else if self.check(TokenKind::Picture) {
            self.parse_picture().map(DocumentChild::Picture)
        } else if self.check(TokenKind::Text) {
            self.parse_text().map(DocumentChild::Text)
        } else {
            None
        }
    }

    /// Parse a declaration shared by document, layer, group, and repeat
    /// bodies. `allow_raster` keeps clear/pixel/brush out of vector-only
    /// contexts.
    fn parse_shared_child(&mut self, allow_raster: bool) -> Option<DocumentChild> {
        if self.check(TokenKind::Place) {
            self.parse_place().map(DocumentChild::Place)
        } else if allow_raster && self.check(TokenKind::Clear) {
            self.parse_clear().map(DocumentChild::Clear)
        } else if allow_raster && self.check(TokenKind::Pixel) {
            self.parse_pixel().map(DocumentChild::Pixel)
        } else if allow_raster && self.check(TokenKind::Brush) {
            self.parse_brush().map(DocumentChild::Brush)
        } else if self.check(TokenKind::Let) {
            self.parse_variable().map(DocumentChild::Let)
        } else if self.check(TokenKind::Repeat) {
            self.parse_repeat().map(DocumentChild::Repeat)
        } else {
            self.parse_child_shape()
        }
    }

    fn parse_variable(&mut self) -> Option<Variable> {
        let let_token = self.advance()?; // consume "let"
        let name = self.expect_variable_name(let_token.span)?;
        if !self.check(TokenKind::Assign) {
            let span = self
                .current_token()
                .map(|t| t.span)
                .unwrap_or(let_token.span);
            self.diagnostics.error(
                codes::EXPECTED_TOKEN,
                "Expected '=' after variable name",
                span,
            );
            return None;
        }
        self.advance(); // consume '='
        let value = self.parse_expression()?;
        let span = let_token.span.merge(self.previous_span());
        Some(Variable { name, value, span })
    }

    fn parse_function(&mut self) -> Option<Function> {
        let fn_token = self.advance()?; // consume "fn"
        let name = self.expect_identifier("Expected function name after 'fn'", fn_token.span)?;

        if !self.check(TokenKind::LParen) {
            let span = self
                .current_token()
                .map(|t| t.span)
                .unwrap_or(fn_token.span);
            self.diagnostics.error(
                codes::EXPECTED_TOKEN,
                "Expected '(' after function name",
                span,
            );
            return None;
        }
        self.advance(); // consume '('

        let mut params = Vec::new();
        if !self.check(TokenKind::RParen) {
            loop {
                params.push(self.expect_identifier("Expected parameter name", fn_token.span)?);
                if self.check(TokenKind::Comma) {
                    self.advance();
                    continue;
                }
                break;
            }
        }
        if !self.check(TokenKind::RParen) {
            let span = self
                .current_token()
                .map(|t| t.span)
                .unwrap_or(fn_token.span);
            self.diagnostics
                .error(codes::EXPECTED_TOKEN, "Expected ')' after parameters", span);
            return None;
        }
        self.advance(); // consume ')'

        // Optional same-line return.
        if self.check(TokenKind::Return) {
            self.advance();
            let body = self.parse_expression()?;
            let span = fn_token.span.merge(self.previous_span());
            return Some(Function {
                name,
                params,
                body,
                span,
            });
        }

        self.expect_indent()?;
        let mut body = None;
        while !self.check_dedent() && !self.check_eof() {
            if self.check_newline() {
                self.advance();
                continue;
            }
            if self.check(TokenKind::Return) {
                self.advance();
                body = self.parse_expression();
            } else {
                let token = self.current_token().cloned();
                if let Some(tok) = token {
                    self.diagnostics.error(
                        codes::EXPECTED_TOKEN,
                        format!("Expected 'return' in function body, found {:?}", tok.kind),
                        tok.span,
                    );
                    self.advance();
                }
            }
        }
        self.expect_dedent()?;

        let body = body.unwrap_or(Expression::Number(0.0));
        let span = fn_token.span.merge(self.previous_span());
        Some(Function {
            name,
            params,
            body,
            span,
        })
    }

    fn parse_expression(&mut self) -> Option<Expression> {
        self.parse_or()
    }

    /// Parse a bare identifier or a call sharing the identifier grammar.
    fn parse_named_or_call(&mut self, name: IStr, span: Span) -> Expression {
        if !self.check(TokenKind::LParen) {
            return Expression::Identifier(name);
        }
        self.advance();
        let mut args = Vec::new();
        if !self.check(TokenKind::RParen) {
            while let Some(arg) = self.parse_expression() {
                args.push(arg);
                if !self.check(TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
        }
        self.expect(TokenKind::RParen);
        Expression::Call { name, args, span }
    }

    fn parse_or(&mut self) -> Option<Expression> {
        self.parse_binary(Self::parse_and, |kind| {
            matches!(kind, TokenKind::OrKw).then_some(BinaryOp::Or)
        })
    }

    fn parse_and(&mut self) -> Option<Expression> {
        self.parse_binary(Self::parse_equality, |kind| match kind {
            TokenKind::And | TokenKind::AndKw => Some(BinaryOp::And),
            _ => None,
        })
    }

    fn parse_equality(&mut self) -> Option<Expression> {
        self.parse_binary(Self::parse_comparison, |kind| match kind {
            TokenKind::Eq => Some(BinaryOp::Eq),
            TokenKind::Ne => Some(BinaryOp::Ne),
            _ => None,
        })
    }

    fn parse_comparison(&mut self) -> Option<Expression> {
        self.parse_binary(Self::parse_additive, |kind| match kind {
            TokenKind::Lt => Some(BinaryOp::Lt),
            TokenKind::Le => Some(BinaryOp::Le),
            TokenKind::Gt => Some(BinaryOp::Gt),
            TokenKind::Ge => Some(BinaryOp::Ge),
            _ => None,
        })
    }

    fn parse_additive(&mut self) -> Option<Expression> {
        self.parse_binary(Self::parse_multiplicative, |kind| match kind {
            TokenKind::Plus => Some(BinaryOp::Add),
            TokenKind::Minus => Some(BinaryOp::Sub),
            _ => None,
        })
    }

    fn parse_multiplicative(&mut self) -> Option<Expression> {
        self.parse_binary(Self::parse_unary, |kind| match kind {
            TokenKind::Star => Some(BinaryOp::Mul),
            TokenKind::Slash => Some(BinaryOp::Div),
            TokenKind::Percent => Some(BinaryOp::Mod),
            _ => None,
        })
    }

    /// Parse one left-associative precedence level.
    fn parse_binary(
        &mut self,
        parse_operand: fn(&mut Self) -> Option<Expression>,
        operator: fn(&TokenKind) -> Option<BinaryOp>,
    ) -> Option<Expression> {
        let mut lhs = parse_operand(self)?;
        while let Some(op) = self.current_token().and_then(|token| operator(&token.kind)) {
            let op_span = self.advance()?.span;
            let rhs = parse_operand(self)?;
            lhs = Expression::Binary {
                op,
                left: Box::new(lhs),
                right: Box::new(rhs),
                span: op_span,
            };
        }
        Some(lhs)
    }

    fn parse_unary(&mut self) -> Option<Expression> {
        if self.check(TokenKind::Minus) {
            let op_span = self.advance()?.span;
            let expr = self.parse_unary()?;
            return Some(Expression::Unary {
                op: UnaryOp::Neg,
                expr: Box::new(expr),
                span: op_span,
            });
        }
        if self.check(TokenKind::NotKw) {
            let op_span = self.advance()?.span;
            let expr = self.parse_unary()?;
            return Some(Expression::Unary {
                op: UnaryOp::Not,
                expr: Box::new(expr),
                span: op_span,
            });
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Option<Expression> {
        let token = self.current_token().cloned()?;
        match token.kind {
            TokenKind::String(value) => {
                self.advance();
                Some(Expression::String(value))
            }
            TokenKind::True => {
                self.advance();
                Some(Expression::Bool(true))
            }
            TokenKind::False => {
                self.advance();
                Some(Expression::Bool(false))
            }
            TokenKind::Number(_) => {
                let value = self.expect_number()?;
                if self.check_unit() {
                    Some(Expression::Dimension(Dimension {
                        value,
                        unit: self.parse_unit(),
                        span: self.previous_span(),
                    }))
                } else {
                    Some(Expression::Number(value))
                }
            }
            TokenKind::Identifier(value) => {
                self.advance();
                Some(self.parse_named_or_call(value, token.span))
            }
            TokenKind::Radius => {
                self.advance();
                Some(Expression::Identifier(IStr::new("radius")))
            }
            TokenKind::Random => {
                let span = token.span;
                self.advance();
                Some(self.parse_named_or_call(IStr::new("random"), span))
            }
            TokenKind::X => {
                self.advance();
                Some(Expression::Identifier(IStr::new("x")))
            }
            TokenKind::LParen => {
                self.advance();
                let first = self.parse_expression()?;
                if self.check(TokenKind::Comma) {
                    self.advance();
                    let second = self.parse_expression()?;
                    self.expect(TokenKind::RParen);
                    // Best-effort conversion to a concrete position when both
                    // operands are dimensions/numbers.
                    let x = expression_dimension(first, token.span);
                    let y = expression_dimension(second, token.span);
                    Some(Expression::Position(Box::new(Position {
                        x,
                        y,
                        span: token.span,
                    })))
                } else {
                    self.expect(TokenKind::RParen);
                    Some(first)
                }
            }
            TokenKind::NotKw => {
                self.advance();
                let expr = self.parse_unary()?;
                Some(Expression::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(expr),
                    span: token.span,
                })
            }
            _ => {
                self.diagnostics.error(
                    codes::EXPECTED_TOKEN,
                    format!("Expected expression, found {:?}", token.kind),
                    token.span,
                );
                None
            }
        }
    }

    fn parse_repeat(&mut self) -> Option<Repeat> {
        let repeat_token = self.advance()?; // consume "repeat"
        let count = self.parse_expression()?;
        if !self.check(TokenKind::Times) {
            let span = self
                .current_token()
                .map(|t| t.span)
                .unwrap_or(repeat_token.span);
            self.diagnostics.error(
                codes::EXPECTED_TOKEN,
                "Expected 'times' after repeat count",
                span,
            );
            return None;
        }
        self.advance(); // consume "times"
        let index =
            if self.check(TokenKind::As) {
                self.advance(); // consume "as"
                Some(self.expect_identifier(
                    "Expected index variable name after 'as'",
                    repeat_token.span,
                )?)
            } else {
                None
            };
        self.expect_indent()?;

        let mut body = Vec::new();
        while !self.check_dedent() && !self.check_eof() {
            if self.check_newline() {
                self.advance();
                continue;
            }
            if let Some(child) = self.parse_shared_child(true) {
                body.push(child);
            } else {
                let token = self.current_token().cloned();
                if let Some(tok) = token {
                    self.diagnostics.error(
                        codes::EXPECTED_TOKEN,
                        format!("Unexpected token in repeat: {:?}", tok.kind),
                        tok.span,
                    );
                    self.advance();
                }
            }
        }
        self.expect_dedent()?;
        let span = repeat_token.span.merge(self.previous_span());
        Some(Repeat {
            count,
            index,
            body,
            span,
        })
    }

    /// Parse fill/stroke/width/cap/join/transform/opacity properties shared by
    /// drawing primitives.
    ///
    /// Returns `Some(true)` when a property was consumed, `Some(false)` when
    /// the current token is not a common property, and `None` when a property
    /// was recognized but could not be parsed.
    fn parse_common_shape_property(
        &mut self,
        properties: &mut ShapeProperties,
        allow_fill: bool,
        allow_stroke: bool,
    ) -> Option<bool> {
        if allow_fill && self.check(TokenKind::Fill) {
            properties.fill = Some(self.parse_fill()?);
        } else if allow_stroke && self.check(TokenKind::Stroke) {
            set_stroke(&mut properties.stroke, self.parse_stroke()?);
        } else if allow_stroke && self.check(TokenKind::Width) {
            self.advance();
            properties.width = self.parse_dimension();
        } else if allow_stroke && self.check(TokenKind::Cap) {
            let span = self.advance()?.span;
            let cap = self.parse_stroke_cap()?;
            stroke_slot(&mut properties.stroke, span).cap = Some(cap);
        } else if allow_stroke && self.check(TokenKind::Join) {
            let span = self.advance()?.span;
            let join = self.parse_stroke_join()?;
            stroke_slot(&mut properties.stroke, span).join = Some(join);
        } else {
            return self.parse_transform_property(&mut properties.transform);
        }
        Some(true)
    }

    /// Parse a rotate/scale/opacity property into a shared transform slot.
    fn parse_transform_property(&mut self, transform: &mut Option<Transform>) -> Option<bool> {
        if self.check(TokenKind::Rotate) {
            let span = self.advance()?.span;
            let angle = self.parse_dimension()?;
            transform_slot(transform, span).rotate = Some(angle);
        } else if self.check(TokenKind::Scale) {
            let span = self.advance()?.span;
            let scale = self.parse_scale(span)?;
            transform_slot(transform, span).scale = Some(scale);
        } else if self.check(TokenKind::Opacity) {
            let span = self.advance()?.span;
            let opacity = self.parse_signed_number()?;
            transform_slot(transform, span).opacity = Some(opacity);
        } else {
            return Some(false);
        }
        Some(true)
    }

    /// Parse a primitive body with shape-specific and shared properties.
    ///
    /// The callback handles properties unique to one primitive and returns
    /// `Some(true)` when it consumed one, `Some(false)` when it did not, and
    /// `None` after reporting a fatal parse error. Shared fill/stroke/transform
    /// properties are handled here, as are body termination and the final
    /// `Dedent`.
    fn parse_shape_body<F>(
        &mut self,
        shape: &str,
        fallback_span: Span,
        expected: &str,
        allow_fill: bool,
        allow_stroke: bool,
        mut specific: F,
    ) -> Option<ShapeProperties>
    where
        F: FnMut(&mut Self, &mut ShapeProperties) -> Option<bool>,
    {
        let mut body = self.enter_body()?;
        let mut properties = ShapeProperties::default();

        while self.body_continue(&mut body) {
            if specific(self, &mut properties)? {
                continue;
            }
            if self.parse_common_shape_property(&mut properties, allow_fill, allow_stroke)? {
                continue;
            }
            self.push_shape_unexpected(shape, fallback_span, expected);
        }
        if matches!(body, BodyState::Block) {
            self.expect_dedent()?;
        }
        Some(properties)
    }

    fn parse_picture(&mut self) -> Option<Picture> {
        let picture_token = self.advance()?; // consume "picture"
        let path = self.expect_string()?;
        // `at` may be on the header or as a property line.
        let position = self.parse_at_keyword();

        let mut picture = Picture {
            path,
            position,
            size: None,
            fit: None,
            transform: None,
            span: picture_token.span,
        };
        let properties = self.parse_shape_body(
            "picture",
            picture_token.span,
            "at, size, fit, rotate, scale, or opacity",
            false,
            false,
            |parser, _| {
                if parser.check(TokenKind::At) {
                    picture.position = parser.parse_at_keyword();
                    Some(true)
                } else if parser.check(TokenKind::Size) {
                    picture.size = parser.parse_size();
                    Some(true)
                } else if parser.check(TokenKind::Fit) {
                    parser.advance();
                    picture.fit = Some(parser.parse_fit_mode()?);
                    Some(true)
                } else {
                    Some(false)
                }
            },
        )?;
        picture.transform = properties.transform;
        Some(picture)
    }

    fn parse_text(&mut self) -> Option<Text> {
        let text_token = self.advance()?; // consume "text"
        let content = self.expect_string()?;
        // `at` or `center at` may be on the header or as a property line.
        let mut position = None;
        let mut center = None;
        if self.check(TokenKind::Center) {
            self.advance();
            self.expect(TokenKind::At)?;
            center = self.parse_position();
        } else if self.check(TokenKind::At) {
            position = self.parse_at_keyword();
        }

        let mut text = Text {
            text: content,
            position,
            center,
            font: None,
            size: None,
            weight: None,
            fill: None,
            align: None,
            transform: None,
            span: text_token.span,
        };
        let properties = self.parse_shape_body(
            "text",
            text_token.span,
            "at, center at, font, size, weight, fill, align, rotate, scale, or opacity",
            true,
            false,
            |parser, _| {
                if parser.check(TokenKind::At) {
                    text.position = parser.parse_at_keyword();
                    text.center = None;
                    Some(true)
                } else if parser.check(TokenKind::Center) {
                    // support `center at` as a property line
                    parser.advance();
                    if parser.check(TokenKind::At) {
                        text.center = parser.parse_at_keyword();
                        text.position = None;
                    }
                    Some(true)
                } else if parser.check(TokenKind::Font) {
                    parser.advance();
                    text.font = parser.expect_istr();
                    Some(true)
                } else if parser.check(TokenKind::Size) {
                    parser.advance();
                    text.size = parser.parse_dimension();
                    Some(true)
                } else if parser.check(TokenKind::Weight) {
                    parser.advance();
                    text.weight = parser.parse_font_weight();
                    Some(true)
                } else if parser.check(TokenKind::Align) {
                    parser.advance();
                    text.align = parser.parse_text_align();
                    Some(true)
                } else {
                    Some(false)
                }
            },
        )?;
        text.fill = properties.fill;
        text.transform = properties.transform;
        Some(text)
    }

    fn parse_font_weight(&mut self) -> Option<FontWeight> {
        let token = self.current_token().cloned()?;
        let weight = match token.kind {
            TokenKind::Bold => FontWeight::Bold,
            TokenKind::Normal => FontWeight::Normal,
            _ => {
                self.diagnostics.error_with_help(
                    codes::INVALID_PROPERTY_VALUE,
                    format!("invalid font weight: {:?}", token.kind),
                    token.span,
                    "use 'normal' or 'bold'",
                );
                self.advance();
                return None;
            }
        };
        self.advance();
        Some(weight)
    }

    fn parse_text_align(&mut self) -> Option<TextAlign> {
        let token = self.current_token().cloned()?;
        let align = match token.kind {
            TokenKind::Left => TextAlign::Left,
            TokenKind::Center => TextAlign::Center,
            TokenKind::Right => TextAlign::Right,
            _ => {
                self.diagnostics.error_with_help(
                    codes::INVALID_PROPERTY_VALUE,
                    format!("invalid text align: {:?}", token.kind),
                    token.span,
                    "use 'left', 'center', or 'right'",
                );
                self.advance();
                return None;
            }
        };
        self.advance();
        Some(align)
    }

    fn parse_fit_mode(&mut self) -> Option<FitMode> {
        let token = self.current_token().cloned()?;
        let mode = match token.kind {
            TokenKind::Contain => FitMode::Contain,
            TokenKind::Cover => FitMode::Cover,
            TokenKind::Fill => FitMode::Fill,
            TokenKind::None_ => FitMode::None,
            _ => {
                self.diagnostics.error_with_help(
                    codes::EXPECTED_TOKEN,
                    format!("Expected fit mode, found {:?}", token.kind),
                    token.span,
                    "Expected contain, cover, fill, or none",
                );
                return None;
            }
        };
        self.advance();
        Some(mode)
    }

    fn parse_layer(&mut self) -> Option<LayerDefinition> {
        let layer_token = self.advance()?; // consume "layer"
        let name = self.expect_istr()?;
        self.expect_indent()?;

        let mut layer = LayerDefinition {
            name,
            size: None,
            fill: None,
            mode: None,
            clip: None,
            z_index: None,
            opacity: None,
            blend: None,
            effects: Vec::new(),
            children: Vec::new(),
            span: layer_token.span,
        };

        while !self.check_dedent() && !self.check_eof() {
            if self.check_newline() {
                self.advance();
                continue;
            }
            if self.check(TokenKind::Size) {
                layer.size = self.parse_size();
            } else if self.check(TokenKind::Fill) {
                // `fill` directly inside a layer is the background shorthand,
                // not a primitive (specification §6.4).
                if let Some(fill) = self.parse_fill() {
                    if layer.fill.is_some() {
                        self.diagnostics.push(
                            Diagnostic::error(
                                codes::DUPLICATE_LAYER_FILL,
                                "a layer declares its background `fill` more than once",
                                fill.span,
                            )
                            .with_help(
                                "keep a single `fill` line before the layer's other content",
                            ),
                        );
                    } else {
                        layer.fill = Some(fill);
                    }
                }
            } else if self.check(TokenKind::ZIndex) {
                self.advance();
                layer.z_index = self.parse_signed_number();
            } else if self.check(TokenKind::Opacity) {
                self.advance();
                layer.opacity = self.parse_signed_number();
            } else if self.check(TokenKind::Blend) {
                self.advance();
                if let Some(TokenKind::Identifier(name)) =
                    self.current_token().map(|t| t.kind.clone())
                {
                    layer.blend = Some(name);
                    self.advance();
                }
            } else if self.check(TokenKind::Mode) {
                layer.mode = self.parse_layer_mode();
            } else if self.check(TokenKind::Clip) {
                layer.clip = self.parse_layer_clip();
            } else if self.check(TokenKind::Effect) {
                if let Some(effect) = self.parse_layer_effect() {
                    layer.effects.push(effect);
                }
            } else if let Some(child) = self.parse_shared_child(true) {
                layer.children.push(child);
            } else {
                let token = self.current_token().cloned();
                if let Some(tok) = token {
                    let diag = if let TokenKind::Identifier(name) = &tok.kind {
                        Diagnostic::error(
                            codes::UNKNOWN_KEYWORD,
                            format!("unknown keyword or property `{name}` in layer"),
                            tok.span,
                        )
                        .with_help("Expected size, a drawing primitive, or 'place'")
                        .with_suggestion(name)
                    } else {
                        Diagnostic::error(
                            codes::EXPECTED_TOKEN,
                            format!("Unexpected token in layer: {:?}", tok.kind),
                            tok.span,
                        )
                        .with_help("Expected size, a drawing primitive, or 'place'")
                    };
                    self.diagnostics.push(diag);
                    self.advance();
                }
            }
        }
        self.expect_dedent()?;
        Some(layer)
    }

    /// Parse a reusable `group "Name"` definition block.
    ///
    /// A group holds drawing primitives and nested `place` declarations. It
    /// accepts an optional `size`; omitting it selects the automatic bounding
    /// box of its children (specification §6.2).
    fn parse_group(&mut self) -> Option<GroupDefinition> {
        let group_token = self.advance()?; // consume "group"
        let name = self.expect_istr()?;
        self.expect_indent()?;

        let mut group = GroupDefinition {
            name,
            size: None,
            children: Vec::new(),
            span: group_token.span,
        };

        while !self.check_dedent() && !self.check_eof() {
            if self.check_newline() {
                self.advance();
                continue;
            }
            if self.check(TokenKind::Size) {
                group.size = self.parse_size();
            } else if let Some(child) = self.parse_shared_child(false) {
                group.children.push(child);
            } else {
                let token = self.current_token().cloned();
                if let Some(tok) = token {
                    let diag = if let TokenKind::Identifier(name) = &tok.kind {
                        Diagnostic::error(
                            codes::UNKNOWN_KEYWORD,
                            format!("unknown keyword or property `{name}` in group"),
                            tok.span,
                        )
                        .with_help("Expected size, a drawing primitive, or 'place'")
                        .with_suggestion(name)
                    } else {
                        Diagnostic::error(
                            codes::EXPECTED_TOKEN,
                            format!("Unexpected token in group: {:?}", tok.kind),
                            tok.span,
                        )
                        .with_help("Expected size, a drawing primitive, or 'place'")
                    };
                    self.diagnostics.push(diag);
                    self.advance();
                }
            }
        }
        self.expect_dedent()?;
        Some(group)
    }

    fn parse_clear(&mut self) -> Option<Clear> {
        let clear_token = self.advance()?; // consume "clear"
        let color = self.parse_color()?;
        Some(Clear {
            color,
            span: clear_token.span,
        })
    }

    fn parse_pixel(&mut self) -> Option<Pixel> {
        let pixel_token = self.advance()?; // consume "pixel"
        let position = self.parse_at_keyword();
        let mut body = self.enter_body()?;

        let mut pixel = Pixel {
            position,
            fill: None,
            span: pixel_token.span,
        };

        while self.body_continue(&mut body) {
            if self.check(TokenKind::Fill) {
                pixel.fill = Some(self.parse_fill()?);
            } else {
                self.push_shape_unexpected("pixel", pixel_token.span, "at or fill");
            }
        }
        if matches!(body, BodyState::Block) {
            self.expect_dedent()?;
        }
        Some(pixel)
    }

    fn parse_brush(&mut self) -> Option<Brush> {
        let brush_token = self.advance()?; // consume "brush"
        let mut brush = Brush {
            size: None,
            fill: None,
            points: Vec::new(),
            span: brush_token.span,
        };

        // Optional inline size: brush size 12px
        if self.check(TokenKind::Size) {
            self.advance();
            brush.size = self.parse_dimension();
        }

        let mut body = self.enter_body()?;

        while self.body_continue(&mut body) {
            if self.check(TokenKind::Size) {
                self.advance();
                brush.size = self.parse_dimension();
            } else if self.check(TokenKind::Fill) {
                brush.fill = Some(self.parse_fill()?);
            } else if self.check(TokenKind::Stroke) {
                self.advance();
                if let Some(points) = self.parse_points_block("brush") {
                    brush.points = points;
                }
            } else {
                self.push_shape_unexpected("brush", brush_token.span, "size, fill, or stroke");
            }
        }
        if matches!(body, BodyState::Block) {
            self.expect_dedent()?;
        }
        Some(brush)
    }

    fn parse_layer_clip(&mut self) -> Option<bool> {
        let _clip_token = self.advance()?; // consume "clip"
        let value = self.current_token().cloned()?;
        let result = match value.kind {
            TokenKind::True => true,
            TokenKind::False => false,
            _ => {
                self.diagnostics.error_with_help(
                    codes::EXPECTED_TOKEN,
                    format!("Expected true or false, found {:?}", value.kind),
                    value.span,
                    "Expected `true` or `false`",
                );
                return None;
            }
        };
        self.advance();
        Some(result)
    }

    fn parse_layer_effect(&mut self) -> Option<LayerEffect> {
        let effect_token = self.advance()?; // consume "effect"
        let kind_token = self.current_token().cloned()?;
        let kind = match &kind_token.kind {
            TokenKind::Blur => IStr::new("blur"),
            TokenKind::Brightness => IStr::new("brightness"),
            TokenKind::Contrast => IStr::new("contrast"),
            TokenKind::Saturation => IStr::new("saturation"),
            TokenKind::Grayscale => IStr::new("grayscale"),
            TokenKind::HueRotate => IStr::new("hue-rotate"),
            TokenKind::Sharpen => IStr::new("sharpen"),
            TokenKind::Noise => IStr::new("noise"),
            TokenKind::Pixelate => IStr::new("pixelate"),
            TokenKind::Vignette => IStr::new("vignette"),
            TokenKind::DisplacementMap => IStr::new("displacement-map"),
            TokenKind::Identifier(s) => *s,
            _ => {
                self.diagnostics.error_with_help(
                    codes::INVALID_PROPERTY_VALUE,
                    format!("invalid filter type: {:?}", kind_token.kind),
                    kind_token.span,
                    "Expected a filter name such as blur, brightness, contrast, ...",
                );
                self.advance();
                return None;
            }
        };
        self.advance();

        let mut value = None;
        let mut resource = None;
        if let Some(TokenKind::String(s)) = self.current_token().map(|t| t.kind.clone()) {
            resource = Some(s);
            self.advance();
        } else if self.check(TokenKind::Minus)
            || self
                .current_token()
                .map(|t| matches!(t.kind, TokenKind::Number(_)))
                .unwrap_or(false)
        {
            value = self.parse_dimension();
        }

        Some(LayerEffect {
            kind,
            value,
            resource,
            span: effect_token.span,
        })
    }

    fn parse_layer_mode(&mut self) -> Option<LayerMode> {
        let _mode_token = self.advance()?; // consume "mode"
        let value = self.current_token().cloned()?;
        let mode = match value.kind {
            TokenKind::Raster => LayerMode::Raster,
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("vector") => LayerMode::Vector,
            _ => {
                self.diagnostics.error_with_help(
                    codes::EXPECTED_TOKEN,
                    format!("Expected layer mode, found {:?}", value.kind),
                    value.span,
                    "Expected `raster`",
                );
                return None;
            }
        };
        self.advance();
        Some(mode)
    }

    fn parse_size(&mut self) -> Option<Size> {
        let size_token = self.advance()?; // consume "size"
        let width = self.parse_dimension()?;
        self.expect(TokenKind::X)?;
        let height = self.parse_dimension()?;
        Some(Size {
            width,
            height,
            span: size_token.span,
        })
    }

    fn parse_seed(&mut self) -> Option<u64> {
        let seed_token = self.advance()?; // consume "seed"
        let value = self.parse_signed_number()?;
        if value < 0.0 {
            let span = seed_token.span;
            self.diagnostics
                .error(codes::INVALID_SEED, "seed must be non-negative", span);
            return None;
        }
        Some(value as u64)
    }

    fn parse_background(&mut self) -> Option<Background> {
        let bg_token = self.advance()?; // consume "background"
        let color = self.parse_color()?;
        Some(Background {
            color,
            span: bg_token.span,
        })
    }

    fn parse_rectangle(&mut self) -> Option<Rectangle> {
        let rect_token = self.advance()?; // consume "rectangle"
        // `at` is required by the language, but a missing position is reported
        // by semantic validation (S003) so the shape is still retained here.
        let position = if self.check(TokenKind::At) {
            self.advance();
            self.parse_position()
        } else {
            None
        };

        let mut rect = Rectangle {
            position,
            size: None,
            fill: None,
            stroke: None,
            width: None,
            transform: None,
            span: rect_token.span,
        };
        let properties = self.parse_shape_body(
            "rectangle",
            rect_token.span,
            "at, size, fill, stroke, width, rotate, scale, or opacity",
            true,
            true,
            |parser, _| {
                if parser.check(TokenKind::Size) {
                    rect.size = parser.parse_size();
                    Some(true)
                } else {
                    Some(false)
                }
            },
        )?;
        rect.fill = properties.fill;
        rect.stroke = properties.stroke;
        rect.width = properties.width;
        rect.transform = properties.transform;
        Some(rect)
    }

    fn parse_circle(&mut self) -> Option<Circle> {
        let circle_token = self.advance()?; // consume "circle"
        // `center at` is required by the language, but a missing center is
        // reported by semantic validation (S003) so the shape is retained.
        let center = if self.check(TokenKind::Center) {
            self.advance();
            self.expect(TokenKind::At)?;
            self.parse_position()
        } else {
            None
        };

        let mut circle = Circle {
            center,
            radius: None,
            fill: None,
            stroke: None,
            width: None,
            transform: None,
            span: circle_token.span,
        };
        let properties = self.parse_shape_body(
            "circle",
            circle_token.span,
            "center at, radius, fill, stroke, width, rotate, scale, or opacity",
            true,
            true,
            |parser, _| {
                if parser.check(TokenKind::Radius) {
                    parser.advance(); // consume "radius"
                    circle.radius = parser.parse_dimension();
                    Some(true)
                } else {
                    Some(false)
                }
            },
        )?;
        circle.fill = properties.fill;
        circle.stroke = properties.stroke;
        circle.width = properties.width;
        circle.transform = properties.transform;
        Some(circle)
    }

    fn parse_ellipse(&mut self) -> Option<Ellipse> {
        let ellipse_token = self.advance()?; // consume "ellipse"
        let center = self.parse_center_keyword();

        let mut ellipse = Ellipse {
            center,
            size: None,
            fill: None,
            stroke: None,
            width: None,
            transform: None,
            span: ellipse_token.span,
        };
        let properties = self.parse_shape_body(
            "ellipse",
            ellipse_token.span,
            "size, fill, stroke, or width",
            true,
            true,
            |parser, _| {
                if parser.check(TokenKind::Size) {
                    ellipse.size = parser.parse_size();
                    Some(true)
                } else {
                    Some(false)
                }
            },
        )?;
        ellipse.fill = properties.fill;
        ellipse.stroke = properties.stroke;
        ellipse.width = properties.width;
        ellipse.transform = properties.transform;
        Some(ellipse)
    }

    fn parse_square(&mut self) -> Option<Square> {
        let square_token = self.advance()?; // consume "square"
        let position = self.parse_at_keyword();

        let mut square = Square {
            position,
            size: None,
            fill: None,
            stroke: None,
            width: None,
            transform: None,
            span: square_token.span,
        };
        let properties = self.parse_shape_body(
            "square",
            square_token.span,
            "size, fill, stroke, or width",
            true,
            true,
            |parser, _| {
                if parser.check(TokenKind::Size) {
                    parser.advance(); // consume "size"
                    square.size = parser.parse_dimension();
                    Some(true)
                } else {
                    Some(false)
                }
            },
        )?;
        square.fill = properties.fill;
        square.stroke = properties.stroke;
        square.width = properties.width;
        square.transform = properties.transform;
        Some(square)
    }

    fn parse_triangle(&mut self) -> Option<Triangle> {
        let triangle_token = self.advance()?; // consume "triangle"
        let position = self.parse_at_keyword();

        let mut triangle = Triangle {
            position,
            size: None,
            fill: None,
            stroke: None,
            width: None,
            transform: None,
            span: triangle_token.span,
        };
        let properties = self.parse_shape_body(
            "triangle",
            triangle_token.span,
            "size, fill, stroke, or width",
            true,
            true,
            |parser, _| {
                if parser.check(TokenKind::Size) {
                    triangle.size = parser.parse_size();
                    Some(true)
                } else {
                    Some(false)
                }
            },
        )?;
        triangle.fill = properties.fill;
        triangle.stroke = properties.stroke;
        triangle.width = properties.width;
        triangle.transform = properties.transform;
        Some(triangle)
    }

    fn parse_polygon(&mut self) -> Option<Polygon> {
        let polygon_token = self.advance()?; // consume "polygon"

        let mut polygon = Polygon {
            points: None,
            fill: None,
            stroke: None,
            width: None,
            transform: None,
            span: polygon_token.span,
        };
        let properties = self.parse_shape_body(
            "polygon",
            polygon_token.span,
            "points, fill, stroke, or width",
            true,
            true,
            |parser, _| {
                if parser.check(TokenKind::Points) {
                    parser.advance(); // consume "points"
                    polygon.points = Some(parser.parse_points_block("polygon")?);
                    Some(true)
                } else {
                    Some(false)
                }
            },
        )?;
        polygon.fill = properties.fill;
        polygon.stroke = properties.stroke;
        polygon.width = properties.width;
        polygon.transform = properties.transform;
        Some(polygon)
    }

    fn parse_points_block(&mut self, shape: &str) -> Option<Vec<Position>> {
        self.expect_indent()?;
        let mut points = Vec::new();
        while !self.check_dedent() && !self.check_eof() {
            if self.check_newline() {
                self.advance();
                continue;
            }
            if self.check(TokenKind::LParen) {
                if let Some(position) = self.parse_position() {
                    points.push(position);
                }
            } else {
                let token = self.current_token().cloned();
                if let Some(tok) = token {
                    self.diagnostics.error_with_help(
                        codes::EXPECTED_TOKEN,
                        format!("Unexpected token in {shape} points: {:?}", tok.kind),
                        tok.span,
                        "Expected a point '(x, y)'",
                    );
                    self.advance();
                }
            }
        }
        self.expect_dedent()?;
        Some(points)
    }

    fn parse_line(&mut self) -> Option<Line> {
        let line_token = self.advance()?; // consume "line"
        let from = if self.check(TokenKind::From) {
            self.advance();
            self.parse_position()
        } else {
            None
        };
        let to = if self.check(TokenKind::To) {
            self.advance();
            self.parse_position()
        } else {
            None
        };

        let mut line = Line {
            from,
            to,
            stroke: None,
            width: None,
            transform: None,
            span: line_token.span,
        };
        let properties = self.parse_shape_body(
            "line",
            line_token.span,
            "stroke or width",
            false,
            true,
            |_, _| Some(false),
        )?;
        line.stroke = properties.stroke;
        line.width = properties.width;
        line.transform = properties.transform;
        Some(line)
    }

    fn parse_polyline(&mut self) -> Option<Polyline> {
        let polyline_token = self.advance()?; // consume "polyline"

        let mut polyline = Polyline {
            points: None,
            stroke: None,
            width: None,
            transform: None,
            span: polyline_token.span,
        };
        let properties = self.parse_shape_body(
            "polyline",
            polyline_token.span,
            "points, stroke, or width",
            false,
            true,
            |parser, _| {
                if parser.check(TokenKind::Points) {
                    parser.advance();
                    polyline.points = Some(parser.parse_points_block("polyline")?);
                    Some(true)
                } else {
                    Some(false)
                }
            },
        )?;
        polyline.stroke = properties.stroke;
        polyline.width = properties.width;
        polyline.transform = properties.transform;
        Some(polyline)
    }

    fn parse_path(&mut self) -> Option<Path> {
        let path_token = self.advance()?; // consume "path"

        let mut path = Path {
            commands: Vec::new(),
            fill: None,
            stroke: None,
            width: None,
            transform: None,
            span: path_token.span,
        };
        let properties = self.parse_shape_body(
            "path",
            path_token.span,
            "start, line, curve, quad, close, fill, stroke, or width",
            true,
            true,
            |parser, _| {
                if parser.check(TokenKind::Start) {
                    let span = parser.advance()?.span;
                    parser.expect(TokenKind::At)?;
                    let position = parser.parse_position()?;
                    path.commands.push(PathCommand::Start { position, span });
                    Some(true)
                } else if parser.check(TokenKind::Line) {
                    let span = parser.advance()?.span;
                    parser.expect(TokenKind::To)?;
                    let to = parser.parse_position()?;
                    path.commands.push(PathCommand::LineTo { to, span });
                    Some(true)
                } else if parser.check(TokenKind::Curve) {
                    let span = parser.advance()?.span;
                    parser.expect(TokenKind::To)?;
                    let to = parser.parse_position()?;
                    let (control1, control2) = parser.parse_control_pair()?;
                    path.commands.push(PathCommand::CurveTo {
                        to,
                        control1,
                        control2,
                        span,
                    });
                    Some(true)
                } else if parser.check(TokenKind::Quad) {
                    let span = parser.advance()?.span;
                    parser.expect(TokenKind::To)?;
                    let to = parser.parse_position()?;
                    let control = parser.parse_single_control()?;
                    path.commands
                        .push(PathCommand::QuadTo { to, control, span });
                    Some(true)
                } else if parser.check(TokenKind::Close) {
                    let span = parser.advance()?.span;
                    path.commands.push(PathCommand::Close { span });
                    Some(true)
                } else {
                    Some(false)
                }
            },
        )?;
        path.fill = properties.fill;
        path.stroke = properties.stroke;
        path.width = properties.width;
        path.transform = properties.transform;
        Some(path)
    }

    fn parse_control_pair(&mut self) -> Option<(Position, Position)> {
        self.expect_indent()?;
        while self.check_newline() {
            self.advance();
        }
        self.expect(TokenKind::Control)?;
        let control1 = self.parse_position()?;
        self.expect(TokenKind::And)?;
        let control2 = self.parse_position()?;
        while self.check_newline() {
            self.advance();
        }
        self.expect_dedent()?;
        Some((control1, control2))
    }

    fn parse_single_control(&mut self) -> Option<Position> {
        self.expect_indent()?;
        while self.check_newline() {
            self.advance();
        }
        self.expect(TokenKind::Control)?;
        let control = self.parse_position()?;
        while self.check_newline() {
            self.advance();
        }
        self.expect_dedent()?;
        Some(control)
    }

    fn parse_stroke(&mut self) -> Option<Stroke> {
        let stroke_token = self.advance()?; // consume "stroke"
        if self.check(TokenKind::None_) {
            self.advance();
            return Some(Stroke {
                color: None,
                cap: None,
                join: None,
                span: stroke_token.span,
            });
        }
        let color = self.parse_color()?;
        Some(Stroke {
            color: Some(color),
            cap: None,
            join: None,
            span: stroke_token.span,
        })
    }

    fn parse_center_keyword(&mut self) -> Option<Position> {
        if self.check(TokenKind::Center) {
            self.advance();
            self.expect(TokenKind::At)?;
            self.parse_position()
        } else {
            None
        }
    }

    fn parse_at_keyword(&mut self) -> Option<Position> {
        if self.check(TokenKind::At) {
            self.advance();
            self.parse_position()
        } else {
            None
        }
    }

    fn push_shape_unexpected(&mut self, shape: &str, fallback_span: Span, expected: &str) {
        let token = self.current_token().cloned();
        let diagnostic = match &token {
            Some(tok) => {
                let help = format!("Expected {expected}");
                match &tok.kind {
                    TokenKind::Identifier(name) => Diagnostic::error(
                        codes::UNKNOWN_KEYWORD,
                        format!("unknown keyword or property `{name}` in {shape}"),
                        tok.span,
                    )
                    .with_help(help)
                    .with_suggestion(name),
                    _ => Diagnostic::error(
                        codes::EXPECTED_TOKEN,
                        format!("Unexpected token in {shape}: {:?}", tok.kind),
                        tok.span,
                    )
                    .with_help(help),
                }
            }
            None => Diagnostic::error(
                codes::EXPECTED_TOKEN,
                format!("Unexpected end of input in {shape}"),
                fallback_span,
            )
            .with_help(format!("Expected {expected}")),
        };
        self.diagnostics.push(diagnostic);
        self.advance();
    }

    fn parse_position(&mut self) -> Option<Position> {
        self.expect(TokenKind::LParen)?;
        let x = self.parse_dimension()?;
        let x_span = x.span;
        self.expect(TokenKind::Comma)?;
        let y = self.parse_dimension()?;
        let y_span = y.span;
        self.expect(TokenKind::RParen)?;
        Some(Position {
            x,
            y,
            span: x_span.merge(y_span),
        })
    }

    fn parse_dimension(&mut self) -> Option<Dimension> {
        // A leading minus is part of the numeric literal so that negative
        // dimensions/radii parse and are rejected by semantic validation
        // (S001/S002) rather than producing a confusing syntax error.
        let num_value = self.parse_signed_number()?;
        let unit = self.parse_unit();
        Some(Dimension {
            value: num_value,
            unit,
            span: self.previous_span(),
        })
    }

    fn parse_stroke_cap(&mut self) -> Option<StrokeCap> {
        let token = self.current_token().cloned()?;
        let cap = match token.kind {
            TokenKind::Butt => StrokeCap::Butt,
            TokenKind::Round => StrokeCap::Round,
            TokenKind::Square | TokenKind::Square_ => StrokeCap::Square,
            _ => {
                self.diagnostics.error_with_help(
                    codes::INVALID_PROPERTY_VALUE,
                    format!("invalid line cap: {:?}", token.kind),
                    token.span,
                    "use 'butt', 'round', or 'square'",
                );
                self.advance();
                return None;
            }
        };
        self.advance();
        Some(cap)
    }

    fn parse_stroke_join(&mut self) -> Option<StrokeJoin> {
        let token = self.current_token().cloned()?;
        let join = match token.kind {
            TokenKind::Miter => StrokeJoin::Miter,
            TokenKind::Round => StrokeJoin::Round,
            TokenKind::Bevel => StrokeJoin::Bevel,
            _ => {
                self.diagnostics.error_with_help(
                    codes::INVALID_PROPERTY_VALUE,
                    format!("invalid line join: {:?}", token.kind),
                    token.span,
                    "use 'miter', 'round', or 'bevel'",
                );
                self.advance();
                return None;
            }
        };
        self.advance();
        Some(join)
    }

    fn parse_signed_number(&mut self) -> Option<f64> {
        let negative = if self.check(TokenKind::Minus) {
            self.advance();
            true
        } else {
            false
        };
        let value = self.expect_number()?;
        Some(if negative { -value } else { value })
    }

    fn parse_scale(&mut self, span: Span) -> Option<Scale> {
        let x = self.parse_signed_number()?;
        let y = if self.check(TokenKind::X) {
            self.advance();
            self.parse_signed_number()?
        } else {
            x
        };
        Some(Scale { x, y, span })
    }

    fn parse_color(&mut self) -> Option<Color> {
        let token = self.current_token()?;
        match &token.kind {
            TokenKind::String(s) => {
                let s = IStr::new(s);
                self.advance();
                if s.starts_with('#') {
                    Some(Color::Hex(s))
                } else {
                    Some(Color::Named(s))
                }
            }
            TokenKind::Identifier(s) => {
                let s = *s;
                self.advance();
                Some(Color::Named(s))
            }
            _ => {
                self.diagnostics.error_with_help(
                    codes::EXPECTED_TOKEN,
                    format!(
                        "Expected color (named color or hex string), found {:?}",
                        token.kind
                    ),
                    token.span,
                    "Provide a color name (e.g., red) or hex string (e.g., \"#FF0000\")",
                );
                None
            }
        }
    }

    fn parse_fill(&mut self) -> Option<Fill> {
        let fill_token = self.advance()?; // consume "fill"
        if self.check(TokenKind::None_) {
            self.advance();
            return Some(Fill {
                color: None,
                gradient: None,
                pattern: None,
                span: fill_token.span,
            });
        }
        let (is_gradient, radial) = if self.check(TokenKind::Radial) {
            self.advance(); // consume "radial"
            if self.check(TokenKind::Gradient) {
                self.advance(); // consume optional "gradient"
            }
            (true, true)
        } else if self.check(TokenKind::Gradient) {
            self.advance(); // consume "gradient"
            let radial = if self.check(TokenKind::Radial) {
                self.advance(); // consume optional "radial"
                true
            } else {
                false
            };
            (true, radial)
        } else {
            (false, false)
        };
        if is_gradient {
            self.expect_indent()?;
            let mut gradient = GradientFill {
                radial,
                from: None,
                to: None,
                stops: Vec::new(),
                angle: None,
                span: fill_token.span,
            };
            while !self.check_dedent() && !self.check_eof() {
                if self.check_newline() {
                    self.advance();
                    continue;
                }
                if self.check(TokenKind::From) {
                    self.advance();
                    gradient.from = self.parse_color();
                } else if self.check(TokenKind::To) {
                    self.advance();
                    gradient.to = self.parse_color();
                } else if self.check(TokenKind::Stop) {
                    self.advance();
                    let offset = self.parse_dimension().unwrap_or(Dimension {
                        value: 0.0,
                        unit: Unit::Percent,
                        span: fill_token.span,
                    });
                    let color = self
                        .parse_color()
                        .unwrap_or(Color::Named(IStr::new("black")));
                    gradient.stops.push(GradientStop {
                        offset,
                        color,
                        span: fill_token.span,
                    });
                } else if self.check(TokenKind::Angle) {
                    self.advance();
                    gradient.angle = self.parse_dimension();
                } else {
                    let token = self.current_token().cloned();
                    if let Some(tok) = token {
                        self.diagnostics.error(
                            codes::EXPECTED_TOKEN,
                            format!("Unexpected token in gradient: {:?}", tok.kind),
                            tok.span,
                        );
                        self.advance();
                    }
                }
            }
            self.expect_dedent()?;
            gradient.span = fill_token.span.merge(self.previous_span());
            return Some(Fill {
                color: None,
                gradient: Some(Box::new(gradient)),
                pattern: None,
                span: fill_token.span,
            });
        }
        if self.check(TokenKind::Pattern) {
            self.advance(); // consume "pattern"
            let mut pattern = PatternFill {
                kind: IStr::new("stripes"),
                color: None,
                background: None,
                size: None,
                angle: None,
                span: fill_token.span,
            };
            // Optional kind on the same line.
            if let Some(TokenKind::Identifier(name)) = self.current_token().map(|t| t.kind.clone())
                && ["dots", "grid", "stripes", "checkerboard"].contains(&name.as_str())
            {
                pattern.kind = name;
                self.advance();
            }
            self.expect_indent()?;
            while !self.check_dedent() && !self.check_eof() {
                if self.check_newline() {
                    self.advance();
                    continue;
                }
                if let Some(TokenKind::Identifier(prop)) =
                    self.current_token().map(|t| t.kind.clone())
                {
                    match prop.as_str() {
                        "kind" => {
                            self.advance();
                            if let Some(TokenKind::Identifier(name)) =
                                self.current_token().map(|t| t.kind.clone())
                            {
                                pattern.kind = name;
                                self.advance();
                            }
                        }
                        "color" => {
                            self.advance();
                            pattern.color = self.parse_color();
                        }
                        "background" => {
                            self.advance();
                            pattern.background = self.parse_color();
                        }
                        "size" => {
                            self.advance();
                            pattern.size = self.parse_dimension();
                        }
                        "angle" => {
                            self.advance();
                            pattern.angle = self.parse_dimension();
                        }
                        _ => {
                            self.advance();
                        }
                    }
                } else if self.check(TokenKind::Background) {
                    self.advance();
                    pattern.background = self.parse_color();
                } else if self.check(TokenKind::Size) {
                    self.advance();
                    pattern.size = self.parse_dimension();
                } else if self.check(TokenKind::Angle) {
                    self.advance();
                    pattern.angle = self.parse_dimension();
                } else {
                    let token = self.current_token().cloned();
                    if let Some(tok) = token {
                        self.diagnostics.error(
                            codes::EXPECTED_TOKEN,
                            format!("Unexpected token in pattern: {:?}", tok.kind),
                            tok.span,
                        );
                        self.advance();
                    }
                }
            }
            self.expect_dedent()?;
            pattern.span = fill_token.span.merge(self.previous_span());
            return Some(Fill {
                color: None,
                gradient: None,
                pattern: Some(Box::new(pattern)),
                span: fill_token.span,
            });
        }

        let color = self.parse_color()?;
        Some(Fill {
            color: Some(color),
            gradient: None,
            pattern: None,
            span: fill_token.span,
        })
    }

    fn expect(&mut self, kind: TokenKind) -> Option<Token> {
        if self.check(kind.clone()) {
            self.advance()
        } else {
            let token = self.current_token().cloned();
            self.diagnostics.error(
                codes::EXPECTED_TOKEN,
                format!(
                    "Expected {:?}, found {:?}",
                    kind,
                    token.as_ref().map(|t| &t.kind)
                ),
                token.map(|t| t.span).unwrap_or_else(|| self.current_span()),
            );
            None
        }
    }

    /// Whether the current token cannot be part of an inline property list.
    fn at_body_boundary(&self) -> bool {
        matches!(
            self.current_token().map(|token| &token.kind),
            Some(TokenKind::Newline | TokenKind::Indent | TokenKind::Dedent | TokenKind::Eof)
        )
    }

    /// Enter a drawing primitive's body.
    ///
    /// Properties may continue on the header line (inline) or live in an
    /// indented block. A missing body is a syntax error.
    fn enter_body(&mut self) -> Option<BodyState> {
        if !self.at_body_boundary() {
            return Some(BodyState::Inline);
        }
        if self.open_indented_body() {
            return Some(BodyState::Block);
        }
        self.diagnostics.error_with_help(
            codes::EXPECTED_TOKEN,
            "Expected an indented block or inline properties",
            self.current_span(),
            "write the properties on this line or indent them one level",
        );
        None
    }

    /// Enter an optional body; an absent body is not an error.
    fn enter_optional_body(&mut self) -> BodyState {
        if !self.at_body_boundary() {
            return BodyState::Inline;
        }
        let start = self.pos;
        if self.open_indented_body() {
            BodyState::Block
        } else {
            self.pos = start;
            BodyState::Empty
        }
    }

    /// Skip blank lines and open an indented body when one follows.
    ///
    /// Returns whether an `Indent` token was consumed; the caller decides
    /// whether an absent block is an error.
    fn open_indented_body(&mut self) -> bool {
        while self.check_newline() {
            self.advance();
        }
        if self.check(TokenKind::Indent) {
            self.advance();
            return true;
        }
        false
    }

    /// Advance to the next property position, returning false at the body end.
    ///
    /// An inline body ends at the end of its line. If the following line is
    /// indented, that block continues the same primitive.
    fn body_continue(&mut self, state: &mut BodyState) -> bool {
        match *state {
            BodyState::Empty => false,
            BodyState::Inline => {
                if self.check_newline() {
                    let mut probe = self.pos;
                    while self
                        .tokens
                        .get(probe)
                        .is_some_and(|token| matches!(token.kind, TokenKind::Newline))
                    {
                        probe += 1;
                    }
                    if self
                        .tokens
                        .get(probe)
                        .is_some_and(|token| matches!(token.kind, TokenKind::Indent))
                    {
                        self.pos = probe + 1;
                        *state = BodyState::Block;
                        true
                    } else {
                        false
                    }
                } else {
                    !self.check_dedent() && !self.check_eof()
                }
            }
            BodyState::Block => {
                while self.check_newline() {
                    self.advance();
                }
                !self.check_dedent() && !self.check_eof()
            }
        }
    }

    fn expect_indent(&mut self) -> Option<()> {
        // Skip newlines before expecting indent
        while self.check_newline() {
            self.advance();
        }
        if self.check(TokenKind::Indent) {
            self.advance();
            Some(())
        } else {
            self.diagnostics.error_with_help(
                codes::EXPECTED_TOKEN,
                "Expected indented block",
                self.current_span(),
                "Add indentation after the declaration",
            );
            None
        }
    }

    fn expect_dedent(&mut self) -> Option<()> {
        if self.check(TokenKind::Dedent) {
            self.advance();
            Some(())
        } else {
            self.diagnostics.error(
                codes::EXPECTED_TOKEN,
                "Expected dedent",
                self.current_span(),
            );
            None
        }
    }

    fn expect_identifier(&mut self, message: &str, fallback_span: Span) -> Option<IStr> {
        if let Some(TokenKind::Identifier(name)) = self.current_token().map(|t| t.kind.clone()) {
            self.advance();
            Some(name)
        } else {
            let span = self
                .current_token()
                .map(|t| t.span)
                .unwrap_or(fallback_span);
            self.diagnostics.error(codes::EXPECTED_TOKEN, message, span);
            None
        }
    }

    fn expect_variable_name(&mut self, fallback_span: Span) -> Option<IStr> {
        let name = match self.current_token().map(|t| t.kind.clone()) {
            Some(TokenKind::Identifier(name)) => name,
            Some(TokenKind::Radius) => IStr::new("radius"),
            Some(TokenKind::X) => IStr::new("x"),
            _ => {
                let span = self
                    .current_token()
                    .map(|t| t.span)
                    .unwrap_or(fallback_span);
                self.diagnostics.error_with_help(
                    codes::EXPECTED_TOKEN,
                    "Expected variable name after 'let'",
                    span,
                    "use an identifier such as `primary`",
                );
                return None;
            }
        };
        self.advance();
        Some(name)
    }

    fn expect_istr(&mut self) -> Option<IStr> {
        self.expect_string().map(|value| IStr::new(&value))
    }

    fn expect_string(&mut self) -> Option<Box<str>> {
        if let Some(token) = self.current_token()
            && let TokenKind::String(s) = &token.kind
        {
            let s = s.clone();
            self.advance();
            return Some(s);
        }
        let token = self.current_token().cloned();
        self.diagnostics.error_with_help(
            codes::EXPECTED_TOKEN,
            format!(
                "Expected string, found {:?}",
                token.as_ref().map(|t| &t.kind)
            ),
            token.map(|t| t.span).unwrap_or_else(|| self.current_span()),
            "Provide a string value in double quotes",
        );
        None
    }

    fn expect_number(&mut self) -> Option<f64> {
        if let Some(token) = self.current_token()
            && let TokenKind::Number(n) = token.kind
        {
            self.advance();
            return Some(n);
        }
        let token = self.current_token().cloned();
        self.diagnostics.error_with_help(
            codes::EXPECTED_TOKEN,
            format!(
                "Expected number, found {:?}",
                token.as_ref().map(|t| &t.kind)
            ),
            token.map(|t| t.span).unwrap_or_else(|| self.current_span()),
            "Provide a numeric value",
        );
        None
    }

    fn check(&self, kind: TokenKind) -> bool {
        self.current_token()
            .map(|t| std::mem::discriminant(&t.kind) == std::mem::discriminant(&kind))
            .unwrap_or(false)
    }

    fn check_dedent(&self) -> bool {
        self.check(TokenKind::Dedent)
    }

    fn check_eof(&self) -> bool {
        // Running past the final token must also count as end of file: a
        // primitive that fails before consuming its body can leave `pos` one
        // past `Eof`, and the document loop must still terminate.
        self.current_token().is_none() || self.check(TokenKind::Eof)
    }

    fn check_newline(&self) -> bool {
        self.check(TokenKind::Newline)
    }

    fn check_unit(&self) -> bool {
        self.check(TokenKind::Unit(TokenUnit::Px))
    }

    fn parse_unit(&mut self) -> Unit {
        let Some(TokenKind::Unit(unit)) = self.current_token().map(|t| t.kind.clone()) else {
            return Unit::Px;
        };
        self.advance();
        match unit {
            TokenUnit::Px => Unit::Px,
            TokenUnit::Pt => Unit::Pt,
            TokenUnit::Percent => Unit::Percent,
            TokenUnit::Deg => Unit::Deg,
        }
    }

    fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            let token = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(token)
        } else {
            None
        }
    }

    fn previous_span(&self) -> Span {
        if self.pos > 0 {
            self.tokens[self.pos - 1].span
        } else {
            Span::single(FileId(0), crate::span::Position::new(0))
        }
    }

    fn current_span(&self) -> Span {
        self.current_token()
            .map(|t| t.span)
            .or_else(|| self.tokens.last().map(|t| t.span))
            .unwrap_or_else(|| Span::single(FileId(0), crate::span::Position::new(0)))
    }
}

fn expression_dimension(expression: Expression, fallback_span: Span) -> Dimension {
    match expression {
        Expression::Dimension(dimension) => dimension,
        Expression::Number(value) => Dimension {
            value,
            unit: Unit::Px,
            span: fallback_span,
        },
        _ => Dimension {
            value: 0.0,
            unit: Unit::Px,
            span: fallback_span,
        },
    }
}

fn set_stroke(slot: &mut Option<Stroke>, parsed: Stroke) {
    if let Some(existing) = slot {
        existing.color = parsed.color;
        existing.span = parsed.span;
    } else {
        *slot = Some(parsed);
    }
}

fn stroke_slot(slot: &mut Option<Stroke>, span: Span) -> &mut Stroke {
    slot.get_or_insert_with(|| Stroke {
        color: None,
        cap: None,
        join: None,
        span,
    })
}

fn transform_slot(slot: &mut Option<Transform>, span: Span) -> &mut Transform {
    slot.get_or_insert_with(|| Transform {
        rotate: None,
        scale: None,
        opacity: None,
        span,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::span::FileId;

    macro_rules! parse_document {
        ($source:expr) => {{
            let (ast, diags) = parse($source);
            assert!(!diags.has_errors(), "Diagnostics: {:?}", diags.as_slice());
            ast.expect("document")
        }};
    }

    macro_rules! parse_errors {
        ($source:expr) => {{
            let (_ast, diags) = parse($source);
            assert!(diags.has_errors(), "expected parse errors");
            diags
        }};
    }

    macro_rules! child {
        ($children:expr, $variant:ident) => {
            child!($children, 0, $variant)
        };
        ($children:expr, $index:expr, $variant:ident) => {{
            match $children.get($index) {
                Some(DocumentChild::$variant(value)) => value,
                other => panic!(
                    "expected DocumentChild::{} at index {}, got {other:?}",
                    stringify!($variant),
                    $index
                ),
            }
        }};
    }

    macro_rules! find_diagnostic {
        ($diags:expr, $needle:expr) => {
            $diags
                .as_slice()
                .iter()
                .find(|diagnostic| diagnostic.message.contains($needle))
                .unwrap_or_else(|| {
                    panic!(
                        "no diagnostic containing {:?}: {:?}",
                        $needle,
                        $diags.as_slice()
                    )
                })
        };
    }

    fn parse(input: &str) -> (Option<Document>, DiagnosticBag) {
        let lexer = Lexer::new(input.to_string(), FileId(0));
        let (tokens, lexer_diags) = lexer.tokenize();
        let parser = Parser::new(tokens);
        let (ast, parser_diags) = parser.parse();
        let mut combined = lexer_diags;
        combined.append(parser_diags);
        (ast, combined)
    }

    #[test]
    fn test_unknown_property_suggestion() {
        let source = r#"fil red"#;
        let diags = parse_errors!(source);
        let d = find_diagnostic!(diags, "fil");
        assert_eq!(d.code, "L006");
        assert!(d.message.contains("unknown keyword or property `fil`"));
        assert!(
            d.help
                .as_deref()
                .unwrap_or_default()
                .contains("did you mean `fill`?")
        );
        assert_eq!(d.edits.len(), 1);
        assert_eq!(d.edits[0].replacement, "fill");
        assert_eq!(d.edits[0].span, d.primary_span);
    }

    #[test]
    fn test_document_size_and_background_are_optional() {
        let source = r#"circle center at (128px, 128px)
    radius 20px
    fill red"#;
        let doc = parse_document!(source);
        assert!(doc.size.is_none());
        assert!(doc.background.is_none());
    }

    #[test]
    fn test_file_level_document_settings_any_order() {
        let source = r#"background white
size 100px x 100px"#;
        let doc = parse_document!(source);
        assert!(doc.size.is_some());
        assert!(doc.background.is_some());
    }

    #[test]
    fn test_duplicate_document_settings_report_stable_codes() {
        let source = r#"size 100px x 100px
size 200px x 200px
background white
background black
seed 1
seed 2
"#;
        let diags = parse_errors!(source);
        let codes: Vec<&str> = diags
            .as_slice()
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect();
        assert_eq!(
            codes,
            vec![
                codes::DUPLICATE_DOCUMENT_SIZE,
                codes::DUPLICATE_BACKGROUND,
                codes::DUPLICATE_SEED,
            ]
        );
    }

    #[test]
    fn test_common_properties_are_shared_across_primitives() {
        let source = r#"size 400px x 300px
rectangle at (10px, 10px)
    size 20px x 20px
    fill red
    stroke black
    width 2px
    cap round
    join bevel
    rotate 5deg
    scale 1.5
    opacity 0.8
circle center at (50px, 50px)
    radius 10px
    fill blue
    stroke white
    width 1px
ellipse center at (80px, 80px)
    size 20px x 10px
    fill green
    stroke black
square at (100px, 10px)
    size 20px
    fill orange
    stroke black
triangle at (130px, 10px)
    size 20px x 30px
    fill purple
    stroke black
polygon
    points
        (160px, 10px)
        (180px, 10px)
        (170px, 30px)
    fill yellow
    stroke black
line from (10px, 100px) to (50px, 100px)
    stroke black
    width 2px
polyline
    points
        (10px, 120px)
        (30px, 110px)
        (50px, 120px)
    stroke blue
path
    start at (10px, 150px)
    line to (50px, 150px)
    fill red
    stroke black
picture "image.png"
    at (10px, 180px)
    size 20px x 20px
    fit contain
    rotate 5deg
    scale 0.5
    opacity 0.9
text "Hello"
    at (10px, 220px)
    size 12px
    fill black
    rotate 5deg
    scale 0.5
    opacity 0.9
"#;
        let doc = parse_document!(source);
        assert_eq!(doc.children.len(), 11);
    }

    #[test]
    fn invalid_sources_report_parse_errors() {
        let cases = [
            (
                "line rejects fill",
                "size 100px x 100px\nline from (0px, 0px) to (10px, 10px)\n    fill red\n",
            ),
            (
                "invalid cap",
                "size 100px x 100px\nline from (0px, 0px) to (10px, 10px)\n    stroke black\n    cap triangle\n",
            ),
            (
                "invalid join",
                "size 100px x 100px\nrectangle at (0px, 0px)\n    size 10px x 10px\n    stroke black\n    join dashed\n",
            ),
            ("missing size separator", "size 100px 100px"),
            ("invalid indentation", "   size 100px x 100px"),
            ("indentation increase too large", "    size 100px x 100px"),
            (
                "unexpected token in document",
                "size 100px x 100px\nunknown_keyword\n",
            ),
            (
                "unexpected token in rectangle",
                "size 100px x 100px\nrectangle at (10px, 10px)\n    size 50px x 50px\n    unknown_property red\n",
            ),
            (
                "missing closing paren",
                "size 100px x 100px\nrectangle at (10px, 10px\n    size 50px x 50px\n    fill red\n",
            ),
            (
                "missing comma in position",
                "size 100px x 100px\nrectangle at (10px 10px)\n    size 50px x 50px\n    fill red\n",
            ),
            (
                "missing indent after rectangle",
                "size 100px x 100px\nrectangle at (10px, 10px)\nsize 50px x 50px\nfill red\n",
            ),
            (
                "background without color",
                "size 100px x 100px\nbackground\n",
            ),
            ("dimension without number", "size px x 100px"),
            (
                "nested shape",
                "size 100px x 100px\nrectangle at (10px, 10px)\n    size 50px x 50px\n    fill red\n    circle center at (25px, 25px)\n        radius 10px\n        fill blue\n",
            ),
            (
                "invalid fit mode",
                "size 100px x 100px\npicture \"photo.png\"\n    at (0px, 0px)\n    fit stretch\n",
            ),
        ];

        for (name, source) in cases {
            let (_ast, diags) = parse(source);
            assert!(diags.has_errors(), "expected parse errors for {name}");
        }
    }

    #[test]
    fn test_parse_minimal_document() {
        let source = r#"size 200px x 100px
background white

rectangle at (10px, 10px)
    size 80px x 40px
    fill red

circle center at (150px, 50px)
    radius 30px
    fill blue

"#;
        let doc = parse_document!(source);
        assert!(doc.size.is_some());
        assert!(doc.background.is_some());
        assert_eq!(doc.children.len(), 2);
    }

    #[test]
    fn test_legacy_document_and_export_are_unknown_keywords() {
        for source in [
            "document \"Old\"\n    size 100px x 100px\n",
            "size 100px x 100px\nexport \"out.png\"\n",
        ] {
            let diags = parse_errors!(source);
            assert!(
                diags
                    .as_slice()
                    .iter()
                    .any(|diagnostic| diagnostic.code == codes::UNKNOWN_KEYWORD),
                "Diagnostics: {:?}",
                diags.as_slice()
            );
        }
    }

    #[test]
    fn test_parse_ellipse() {
        let source = r#"size 200px x 200px
ellipse center at (100px, 80px)
    size 80px x 40px
    fill orange"#;
        let doc = parse_document!(source);
        let ellipse = child!(doc.children, Ellipse);
        let center = ellipse.center.as_ref().expect("center");
        assert_eq!((center.x.value, center.y.value), (100.0, 80.0));
        let size = ellipse.size.as_ref().expect("size");
        assert_eq!((size.width.value, size.height.value), (80.0, 40.0));
        assert!(ellipse.fill.is_some());
    }

    #[test]
    fn test_parse_square_with_single_size() {
        let source = r#"size 200px x 200px
square at (10px, 20px)
    size 50px
    fill red"#;
        let doc = parse_document!(source);
        let square = child!(doc.children, Square);
        let position = square.position.as_ref().expect("position");
        assert_eq!((position.x.value, position.y.value), (10.0, 20.0));
        assert_eq!(square.size.as_ref().expect("size").value, 50.0);
        assert!(square.fill.is_some());
    }

    #[test]
    fn test_parse_triangle() {
        let source = r#"size 200px x 200px
triangle at (10px, 20px)
    size 50px x 60px
    fill purple"#;
        let doc = parse_document!(source);
        let triangle = child!(doc.children, Triangle);
        let position = triangle.position.as_ref().expect("position");
        assert_eq!((position.x.value, position.y.value), (10.0, 20.0));
        let size = triangle.size.as_ref().expect("size");
        assert_eq!((size.width.value, size.height.value), (50.0, 60.0));
        assert!(triangle.fill.is_some());
    }

    #[test]
    fn test_missing_required_properties_are_retained() {
        let source = r#"rectangle
    fill red
circle
    fill blue
ellipse
    size 10px x 10px
square at (0px, 0px)
    fill red
triangle
    size 10px x 10px"#;
        let doc = parse_document!(source);
        let rect = child!(doc.children, Rectangle);
        assert!(rect.position.is_none());
        assert!(rect.size.is_none());
        assert!(rect.fill.is_some());

        let circle = child!(doc.children, 1, Circle);
        assert!(circle.center.is_none());
        assert!(circle.radius.is_none());
        assert!(circle.fill.is_some());

        let ellipse = child!(doc.children, 2, Ellipse);
        assert!(ellipse.center.is_none());
        assert!(ellipse.fill.is_none());

        let square = child!(doc.children, 3, Square);
        assert!(square.size.is_none());
        assert!(square.fill.is_some());

        let triangle = child!(doc.children, 4, Triangle);
        assert!(triangle.position.is_none());
        assert!(triangle.fill.is_none());
    }

    #[test]
    fn test_parse_polygon_points() {
        let source = r#"size 400px x 300px
polygon
    points
        (100px, 100px)
        (200px, 50px)
        (300px, 100px)
        (250px, 200px)
    fill purple"#;
        let doc = parse_document!(source);
        let polygon = child!(doc.children, Polygon);
        let points = polygon.points.as_ref().expect("points");
        assert_eq!(points.len(), 4);
        assert_eq!((points[1].x.value, points[1].y.value), (200.0, 50.0));
        assert!(polygon.fill.is_some());
    }

    #[test]
    fn test_polygon_without_points_is_retained() {
        let source = r#"size 200px x 200px
polygon
    fill purple"#;
        let doc = parse_document!(source);
        let polygon = child!(doc.children, Polygon);
        assert!(polygon.points.is_none());
    }

    #[test]
    fn test_polygon_with_two_points_is_retained() {
        let source = r#"size 200px x 200px
polygon
    points
        (0px, 0px)
        (10px, 0px)
    fill purple"#;
        let doc = parse_document!(source);
        let polygon = child!(doc.children, Polygon);
        assert_eq!(polygon.points.as_ref().expect("points").len(), 2);
    }

    #[test]
    fn test_parse_stroke_cap_and_join() {
        let source = r#"size 200px x 200px
line from (10px, 10px) to (90px, 90px)
    cap round
    join bevel
    stroke black
    width 8px"#;
        let doc = parse_document!(source);
        let line = child!(doc.children, Line);
        let stroke = line.stroke.as_ref().expect("stroke");
        assert_eq!(stroke.cap, Some(StrokeCap::Round));
        assert_eq!(stroke.join, Some(StrokeJoin::Bevel));
    }

    #[test]
    fn test_parse_line_with_stroke_and_width() {
        let source = r#"size 400px x 300px
line from (50px, 60px) to (300px, 200px)
    stroke black
    width 4px"#;
        let doc = parse_document!(source);
        let line = child!(doc.children, Line);
        let from = line.from.as_ref().expect("from");
        let to = line.to.as_ref().expect("to");
        assert_eq!((from.x.value, from.y.value), (50.0, 60.0));
        assert_eq!((to.x.value, to.y.value), (300.0, 200.0));
        assert!(line.stroke.is_some());
        assert_eq!(line.width.as_ref().expect("width").value, 4.0);
    }

    #[test]
    fn test_line_retains_missing_endpoints() {
        let source = r#"size 200px x 200px
line
    stroke black"#;
        let doc = parse_document!(source);
        let line = child!(doc.children, Line);
        assert!(line.from.is_none());
        assert!(line.to.is_none());
        assert!(line.stroke.is_some());
    }

    #[test]
    fn test_parse_polyline() {
        let source = r#"size 400px x 300px
polyline
    points
        (50px, 300px)
        (150px, 200px)
        (250px, 300px)
    stroke blue
    width 5px"#;
        let doc = parse_document!(source);
        let polyline = child!(doc.children, Polyline);
        assert_eq!(polyline.points.as_ref().expect("points").len(), 3);
        assert!(polyline.stroke.is_some());
        assert_eq!(polyline.width.as_ref().expect("width").value, 5.0);
    }

    #[test]
    fn test_unexpected_polyline_point_names_the_shape() {
        let source = r#"size 200px x 200px
polyline
    points
        (10px, 10px)
        size 5px
    stroke black"#;
        let diags = parse_errors!(source);
        find_diagnostic!(diags, "Unexpected token in polyline points");
    }

    #[test]
    fn test_parse_stroke_none() {
        let source = r#"size 100px x 100px
line from (0px, 0px) to (10px, 10px)
    stroke none"#;
        let doc = parse_document!(source);
        let line = child!(doc.children, Line);
        let stroke = line.stroke.as_ref().expect("stroke");
        assert!(stroke.color.is_none());
    }

    #[test]
    fn test_parse_path_commands() {
        let source = r#"size 600px x 400px
path
    start at (100px, 300px)
    line to (200px, 100px)
    curve to (400px, 100px)
        control (250px, 50px) and (350px, 50px)
    quad to (500px, 300px)
        control (450px, 150px)
    close
    fill purple
    stroke black
    width 3px"#;
        let doc = parse_document!(source);
        let path = child!(doc.children, Path);
        assert_eq!(path.commands.len(), 5);
        assert!(matches!(path.commands[0], PathCommand::Start { .. }));
        assert!(matches!(path.commands[1], PathCommand::LineTo { .. }));
        assert!(matches!(path.commands[2], PathCommand::CurveTo { .. }));
        assert!(matches!(path.commands[3], PathCommand::QuadTo { .. }));
        assert!(matches!(path.commands[4], PathCommand::Close { .. }));
        assert!(path.fill.is_some());
        assert!(path.stroke.is_some());
        assert_eq!(path.width.as_ref().expect("width").value, 3.0);
    }

    #[test]
    fn test_path_without_commands_is_retained() {
        let source = r#"size 100px x 100px
path
    fill purple"#;
        let doc = parse_document!(source);
        let path = child!(doc.children, Path);
        assert!(path.commands.is_empty());
    }

    #[test]
    fn test_path_command_order_is_retained() {
        let source = r#"size 100px x 100px
path
    line to (10px, 10px)
    start at (0px, 0px)"#;
        let doc = parse_document!(source);
        let path = child!(doc.children, Path);
        assert!(matches!(path.commands[0], PathCommand::LineTo { .. }));
        assert!(matches!(path.commands[1], PathCommand::Start { .. }));
    }

    #[test]
    fn test_parse_closed_shape_fill_and_stroke() {
        let source = r#"size 200px x 200px
rectangle at (10px, 10px)
    size 50px x 40px
    fill blue
    stroke white
    width 4px"#;
        let doc = parse_document!(source);
        let rect = child!(doc.children, Rectangle);
        assert!(rect.fill.is_some());
        assert!(rect.stroke.is_some());
        assert_eq!(rect.width.as_ref().expect("width").value, 4.0);
    }

    #[test]
    fn test_parse_transform_properties() {
        let source = r#"size 200px x 200px
rectangle at (10px, 10px)
    size 50px x 40px
    fill blue
    rotate 90deg
    scale 2 x 0.5
    opacity 0.8"#;
        let doc = parse_document!(source);
        let rect = child!(doc.children, Rectangle);
        let transform = rect.transform.as_ref().expect("transform");
        let rotate = transform.rotate.as_ref().expect("rotate");
        assert_eq!(rotate.value, 90.0);
        assert_eq!(rotate.unit, Unit::Deg);
        let scale = transform.scale.as_ref().expect("scale");
        assert_eq!((scale.x, scale.y), (2.0, 0.5));
        assert_eq!(transform.opacity, Some(0.8));
    }

    #[test]
    fn test_parse_uniform_scale() {
        let source = r#"size 100px x 100px
circle center at (50px, 50px)
    radius 10px
    scale 0.5"#;
        let doc = parse_document!(source);
        let circle = child!(doc.children, Circle);
        let scale = circle
            .transform
            .as_ref()
            .and_then(|transform| transform.scale.as_ref())
            .expect("scale");
        assert_eq!((scale.x, scale.y), (0.5, 0.5));
    }

    #[test]
    fn test_parse_nested_group_instance() {
        let source = r#"size 200px x 200px
group "Outer"
    size 100px x 100px
    place "Inner" at (10px, 20px)
        scale 0.5
        z-index 2
group "Inner"
    size 20px x 20px"#;
        let doc = parse_document!(source);
        assert_eq!(doc.groups[0].children.len(), 1);
        let instance = child!(doc.groups[0].children, Place);
        assert_eq!(instance.name, "Inner");
        let position = instance.position.as_ref().expect("position");
        assert_eq!((position.x.value, position.y.value), (10.0, 20.0));
        assert!(
            instance
                .transform
                .as_ref()
                .expect("transform")
                .scale
                .is_some()
        );
        assert_eq!(instance.z_index, Some(2.0));
    }

    #[test]
    fn test_parse_group_instance() {
        let source = r#"size 300px x 200px
group "Cloud"
    size 20px x 20px
place "Cloud" at (10px, 20px)
    scale 0.8
    rotate 5deg
    opacity 0.9
    z-index 10"#;
        let doc = parse_document!(source);
        let instance = child!(doc.children, Place);
        assert_eq!(instance.name, "Cloud");
        let position = instance.position.as_ref().expect("position");
        assert_eq!((position.x.value, position.y.value), (10.0, 20.0));
        let transform = instance.transform.as_ref().expect("transform");
        assert_eq!(transform.opacity, Some(0.9));
        assert!(transform.scale.is_some());
        assert!(transform.rotate.is_some());
        assert_eq!(instance.z_index, Some(10.0));
    }

    #[test]
    fn test_place_missing_position_is_retained() {
        let source = r#"size 100px x 100px
place "Cloud"
    z-index 0"#;
        let doc = parse_document!(source);
        let instance = child!(doc.children, Place);
        assert!(instance.position.is_none());
    }

    #[test]
    fn test_parse_layer_definition() {
        let source = r#"size 300px x 200px
layer "Cloud"
    size 240px x 120px
    circle center at (60px, 60px)
        radius 40px
        fill white
    rectangle at (60px, 60px)
        size 120px x 40px
        fill white"#;
        let doc = parse_document!(source);
        assert!(doc.children.is_empty());
        assert_eq!(doc.layers.len(), 1);
        let layer = &doc.layers[0];
        assert_eq!(layer.name, "Cloud");
        let size = layer.size.as_ref().expect("size");
        assert_eq!((size.width.value, size.height.value), (240.0, 120.0));
        assert_eq!(layer.children.len(), 2);
        assert!(matches!(layer.children[0], DocumentChild::Circle(_)));
        assert!(matches!(layer.children[1], DocumentChild::Rectangle(_)));
    }

    #[test]
    fn test_layer_without_size_is_retained() {
        let source = r#"size 300px x 200px
layer "Cloud"
    circle center at (10px, 10px)
        radius 5px"#;
        let doc = parse_document!(source);
        assert!(doc.layers[0].size.is_none());
    }

    #[test]
    fn test_nested_layer_declaration_is_rejected() {
        let source = r#"size 100px x 100px
layer "Outer"
    layer "Inner"
        size 10px x 10px"#;
        let (ast, diags) = parse(source);
        assert!(diags.has_errors(), "nested layers must be rejected");
        let doc = ast.unwrap();
        assert_eq!(doc.layers.len(), 1);
        assert!(doc.layers[0].children.is_empty());
    }

    #[test]
    fn test_layer_background_fill_is_parsed() {
        let source = r##"size 300px x 200px
layer "Sky"
    fill "#87CEEB"
    circle center at (10px, 10px)
        radius 5px"##;
        let doc = parse_document!(source);
        assert!(doc.layers[0].size.is_none());
        assert!(doc.layers[0].fill.is_some());
        assert_eq!(doc.layers[0].children.len(), 1);
    }

    #[test]
    fn test_parse_rectangle() {
        let source = r#"size 200px x 200px
rectangle at (10px, 20px)
    size 50px x 60px
    fill red"#;
        let doc = parse_document!(source);
        assert_eq!(doc.children.len(), 1);
        let rect = child!(doc.children, Rectangle);
        let position = rect.position.as_ref().expect("position");
        assert_eq!(position.x.value, 10.0);
        assert_eq!(position.y.value, 20.0);
        let size = rect.size.as_ref().expect("size");
        assert_eq!(size.width.value, 50.0);
        assert_eq!(size.height.value, 60.0);
        assert!(rect.fill.is_some());
    }

    #[test]
    fn test_parse_circle() {
        let source = r#"size 200px x 200px
circle center at (100px, 100px)
    radius 50px
    fill blue"#;
        let doc = parse_document!(source);
        assert_eq!(doc.children.len(), 1);
        let circle = child!(doc.children, Circle);
        let center = circle.center.as_ref().expect("center");
        assert_eq!(center.x.value, 100.0);
        assert_eq!(center.y.value, 100.0);
        assert_eq!(circle.radius.as_ref().expect("radius").value, 50.0);
        assert!(circle.fill.is_some());
    }

    #[test]
    fn test_parse_hex_color() {
        let source = r##"size 100px x 100px
background "#FF0000"
rectangle at (0px, 0px)
    size 50px x 50px
    fill "#00FF0080""##;
        let doc = parse_document!(source);
        assert!(matches!(doc.background.unwrap().color, Color::Hex(ref s) if s == "#FF0000"));
        let rect = child!(doc.children, Rectangle);
        assert!(
            matches!(rect.fill.as_ref().unwrap().color.as_ref(), Some(Color::Hex(s)) if s == "#00FF0080")
        );
    }

    #[test]
    fn test_parse_named_colors() {
        let source = r#"size 100px x 100px
rectangle at (0px, 0px)
    size 50px x 50px
    fill red
circle center at (50px, 50px)
    radius 25px
    fill transparent"#;
        let doc = parse_document!(source);
        let rect = child!(doc.children, Rectangle);
        assert!(
            matches!(rect.fill.as_ref().unwrap().color.as_ref(), Some(Color::Named(s)) if s == "red")
        );
        let circle = child!(doc.children, 1, Circle);
        assert!(
            matches!(circle.fill.as_ref().unwrap().color.as_ref(), Some(Color::Named(s)) if s == "transparent")
        );
    }

    #[test]
    fn test_parse_fill_none() {
        let source = r#"size 100px x 100px
rectangle at (0px, 0px)
    size 50px x 50px
    fill none"#;
        let doc = parse_document!(source);
        let rect = child!(doc.children, Rectangle);
        let fill = rect.fill.as_ref().expect("fill declaration");
        assert!(fill.color.is_none());
    }

    #[test]
    fn test_parse_different_units() {
        let source = r#"size 200px x 200px
rectangle at (10pt, 20pt)
    size 50px x 60px
    fill red
circle center at (100%, 50%)
    radius 30deg
    fill blue"#;
        let doc = parse_document!(source);
        let rect = child!(doc.children, Rectangle);
        let position = rect.position.as_ref().expect("position");
        assert_eq!(position.x.unit, Unit::Pt);
        assert_eq!(position.y.unit, Unit::Pt);

        let circle = child!(doc.children, 1, Circle);
        let center = circle.center.as_ref().expect("center");
        assert_eq!(center.x.unit, Unit::Percent);
        assert_eq!(center.y.unit, Unit::Percent);
        assert_eq!(circle.radius.as_ref().expect("radius").unit, Unit::Deg);
    }

    #[test]
    fn test_parse_negative_dimension() {
        let source = r#"size 100px x 100px
rectangle at (-10px, 20px)
    size -50px x 60px
    fill red"#;
        let doc = parse_document!(source);
        let rect = child!(doc.children, Rectangle);
        let position = rect.position.as_ref().expect("position");
        assert_eq!(position.x.value, -10.0);
        let size = rect.size.as_ref().expect("size");
        assert_eq!(size.width.value, -50.0);
    }

    #[test]
    fn test_unknown_shape_property_suggests_fill_for_colour() {
        let source = r#"size 100px x 100px
rectangle at (10px, 10px)
    size 50px x 50px
    colour red"#;
        let diags = parse_errors!(source);
        let diagnostic = find_diagnostic!(diags, "colour");
        assert_eq!(diagnostic.code, "L006");
        assert!(
            diagnostic
                .help
                .as_deref()
                .unwrap_or_default()
                .contains("did you mean `fill`?")
        );
    }

    #[test]
    fn test_unknown_circle_property_suggests_alias() {
        let source = r#"size 100px x 100px
circle center at (50px, 50px)
    rad 25px
    fill blue"#;
        let diags = parse_errors!(source);
        let diagnostic = find_diagnostic!(diags, "rad");
        assert!(
            diagnostic
                .help
                .as_deref()
                .unwrap_or_default()
                .contains("did you mean `radius`?")
        );
    }

    #[test]
    fn test_empty_document_block() {
        let source = r#"size 100px x 100px"#;
        let doc = parse_document!(source);
        assert!(doc.children.is_empty());
    }

    #[test]
    fn test_blank_lines_in_document() {
        let source = r#"
size 100px x 100px

rectangle at (10px, 10px)

    size 50px x 50px

    fill red"#;
        let doc = parse_document!(source);
        assert_eq!(doc.children.len(), 1);
    }

    #[test]
    fn test_parse_picture() {
        let source = r#"size 400px x 300px
picture "photo.png"
    at (100px, 100px)
    size 400px x 300px
    fit cover"#;
        let doc = parse_document!(source);
        let picture = child!(doc.children, Picture);
        assert_eq!(picture.path.as_ref(), "photo.png");
        let position = picture.position.as_ref().expect("position");
        assert_eq!((position.x.value, position.y.value), (100.0, 100.0));
        let size = picture.size.as_ref().expect("size");
        assert_eq!((size.width.value, size.height.value), (400.0, 300.0));
        assert_eq!(picture.fit, Some(FitMode::Cover));
    }

    #[test]
    fn test_picture_position_and_fit_on_header() {
        let source = r#"size 100px x 100px
picture "photo.png" at (5px, 6px)
    fit none"#;
        let doc = parse_document!(source);
        let picture = child!(doc.children, Picture);
        let position = picture.position.as_ref().expect("position");
        assert_eq!((position.x.value, position.y.value), (5.0, 6.0));
        assert_eq!(picture.fit, Some(FitMode::None));
        assert!(picture.size.is_none());
    }
}
