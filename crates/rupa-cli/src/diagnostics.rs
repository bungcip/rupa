//! Diagnostic printing for the CLI.
//!
//! Diagnostics are written to stderr with source context per specification
//! §22.3, including stable code, location, a quoted source span with carets,
//! the span label, and help text.

use rupa_syntax::diagnostic::{DiagnosticBag, render_diagnostic};
use std::path::Path;

/// Render all diagnostics in `bag` to stderr with source context.
pub fn print_diagnostics(path: &Path, source: &str, bag: &DiagnosticBag) {
    let display_path = path.display().to_string();
    for diagnostic in bag {
        eprint!("{}", render_diagnostic(&display_path, source, diagnostic));
    }
}
