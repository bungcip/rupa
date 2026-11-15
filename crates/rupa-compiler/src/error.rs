//! Error types for Rupa compiler.

use rupa_syntax::diagnostic::{Diagnostic, DiagnosticBag};
use rupa_syntax::span::Span;
use thiserror::Error;

/// Result type for compiler operations
pub type CompilerResult<T> = Result<T, CompilerError>;

/// Compiler error with diagnostics
#[derive(Debug, Error)]
#[error("Compiler error: {code}")]
pub struct CompilerError {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Source span
    pub span: Span,
    /// Optional help text
    pub help: Option<String>,
}

impl CompilerError {
    /// Create a new compiler error
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
        let mut diagnostic = Diagnostic::error(&self.code, self.message.clone(), self.span);
        diagnostic.help = self.help.clone();
        diagnostic
    }
}

/// Trait for converting to diagnostic bag
pub trait IntoDiagnostics {
    /// Convert to diagnostic bag
    fn into_diagnostics(self) -> DiagnosticBag;
}

impl IntoDiagnostics for CompilerError {
    fn into_diagnostics(self) -> DiagnosticBag {
        let mut bag = DiagnosticBag::new();
        bag.push(self.to_diagnostic());
        bag
    }
}

impl IntoDiagnostics for Vec<CompilerError> {
    fn into_diagnostics(self) -> DiagnosticBag {
        self.into_iter().map(|err| err.to_diagnostic()).collect()
    }
}
