//! Rupa syntax crate: lexer, parser, and AST for the Rupa language.

#![warn(missing_docs)]

pub mod ast;
pub mod diagnostic;
pub mod error;
pub mod formatter;
pub mod intern;
pub mod lexer;
pub mod metadata;
pub mod parser;
pub mod prng;
pub mod random;
pub mod rng;
pub mod span;
pub mod token;

/// Re-export commonly used types from ast
pub use ast::{
    Background, Circle, Color, Dimension, Document, DocumentChild, Ellipse, Fill, GroupInstance,
    LayerDefinition, LayerMode, Line, Path, PathCommand, Polygon, Polyline,
    Position as AstPosition, Rectangle, Scale, Size, Square, Stroke, StrokeCap, StrokeJoin,
    Transform, Triangle, Unit as AstUnit,
};
/// Re-export commonly used types from diagnostic
pub use diagnostic::{
    CodeInfo, Diagnostic, DiagnosticBag, KNOWN_KEYWORDS, Severity, SuggestedEdit, did_you_mean,
    did_you_mean_from, render_diagnostic, render_diagnostics, suggest_keyword,
};
/// Re-export commonly used types from error
pub use error::SyntaxError;
/// Re-export formatter helpers
pub use formatter::format_source;
/// Re-export the interned string handle
pub use intern::IStr;
/// Re-export commonly used types from lexer
pub use lexer::Lexer;
/// Re-export canonical language metadata
pub use metadata::{KeywordClass, KeywordSpec, PRIMITIVES, UNITS};
/// Re-export commonly used types from parser
pub use parser::Parser;
/// Re-export PRNG types
pub use prng::Prng;
/// Re-export commonly used types from span
pub use span::{FileId, Position, Span};
/// Re-export commonly used types from token
pub use token::{Token, TokenKind, Unit};
