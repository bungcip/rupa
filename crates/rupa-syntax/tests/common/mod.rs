//! Shared helpers for `rupa-syntax` integration tests.
#![allow(dead_code)]

use rupa_syntax::{DiagnosticBag, Document, FileId, Lexer, Parser};

/// Lex and parse `source`, merging lexer and parser diagnostics.
pub fn parse(source: &str) -> (Option<Document>, DiagnosticBag) {
    let (tokens, mut diagnostics) = Lexer::new(source.to_string(), FileId(0)).tokenize();
    let (document, parser_diagnostics) = Parser::new(tokens).parse();
    diagnostics.append(parser_diagnostics);
    (document, diagnostics)
}

/// Collect error messages for assertion output.
pub fn error_messages(bag: &DiagnosticBag) -> Vec<String> {
    bag.errors()
        .into_iter()
        .map(|diagnostic| diagnostic.message.clone())
        .collect()
}
