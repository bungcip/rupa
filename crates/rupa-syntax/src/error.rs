//! Error types for Rupa syntax.

use crate::diagnostic::{Diagnostic, DiagnosticBag, Severity};
use crate::span::Span;
use thiserror::Error;

/// Result type for syntax operations
pub type SyntaxResult<T> = Result<T, SyntaxError>;

/// Syntax error with diagnostics
#[derive(Debug, Error)]
#[error("Syntax error: {code}")]
pub struct SyntaxError {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Source span
    pub span: Span,
    /// Optional help text
    pub help: Option<String>,
}

impl SyntaxError {
    /// Create a new syntax error
    pub fn new(code: &str, message: impl Into<String>, span: Span) -> Self {
        Self {
            code: code.to_string(),
            message: message.into(),
            span,
            help: None,
        }
    }

    /// Add help text
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Convert to diagnostic
    pub fn to_diagnostic(&self) -> Diagnostic {
        Diagnostic {
            code: self.code.clone(),
            severity: Severity::Error,
            message: self.message.clone(),
            primary_span: self.span,
            help: self.help.clone(),
            edits: Vec::new(),
        }
    }
}

/// Trait for converting to diagnostic bag
pub trait IntoDiagnostics {
    /// Convert to diagnostic bag
    fn into_diagnostics(self) -> DiagnosticBag;
}

impl IntoDiagnostics for SyntaxError {
    fn into_diagnostics(self) -> DiagnosticBag {
        let mut bag = DiagnosticBag::new();
        bag.push(self.to_diagnostic());
        bag
    }
}

impl IntoDiagnostics for Vec<SyntaxError> {
    fn into_diagnostics(self) -> DiagnosticBag {
        let mut bag = DiagnosticBag::new();
        for err in self {
            bag.push(err.to_diagnostic());
        }
        bag
    }
}
