//! Full front-end compilation: lex, parse, validate, and lower to IR.

use crate::lower::lower_document;
use crate::validation::Validator;
use rupa_ir::document::GraphicsDocument;
use rupa_syntax::ast::Document;
use rupa_syntax::diagnostic::DiagnosticBag;
use rupa_syntax::span::FileId;
use rupa_syntax::{Lexer, Parser};

/// Result of lexing and parsing a source file.
#[derive(Debug, Clone)]
pub struct ParsedSource {
    /// Parsed file-level document.
    pub document: Option<Document>,
    /// Lexer and parser diagnostics.
    pub diagnostics: DiagnosticBag,
}

impl ParsedSource {
    /// Whether any diagnostic has error severity.
    pub fn has_errors(&self) -> bool {
        self.diagnostics.has_errors()
    }
}

/// Result of compiling a source file through lowering.
#[derive(Debug, Clone)]
pub struct CompiledSource {
    /// Parsed document, if available.
    pub document: Option<Document>,
    /// Lowered graphics document, produced only when there are no errors.
    pub graphics: Option<GraphicsDocument>,
    /// All diagnostics from lexing through lowering.
    pub diagnostics: DiagnosticBag,
}

impl CompiledSource {
    /// Whether any diagnostic has error severity.
    pub fn has_errors(&self) -> bool {
        self.diagnostics.has_errors()
    }
}

/// Lex and parse Rupa source as one file-level document.
///
/// A `.rupa` file is exactly one document. Missing or duplicate document
/// settings are reported by semantic validation.
pub fn parse_source(source: impl Into<String>, file_id: FileId) -> ParsedSource {
    let lexer = Lexer::new(source.into(), file_id);
    let (tokens, mut diagnostics) = lexer.tokenize();

    let parser = Parser::new(tokens);
    let (document, parser_diagnostics) = parser.parse();
    diagnostics.append(parser_diagnostics);

    ParsedSource {
        document,
        diagnostics,
    }
}

/// Compile Rupa source through validation and lowering.
///
/// Lowering runs only when the front end produced no errors.
pub fn compile_source(source: impl Into<String>, file_id: FileId) -> CompiledSource {
    let parsed = parse_source(source, file_id);
    let mut diagnostics = parsed.diagnostics;

    let graphics = parsed.document.as_ref().and_then(|document| {
        diagnostics.append(Validator::validate(document));
        if diagnostics.has_errors() {
            return None;
        }
        match lower_document(document) {
            Ok(graphics) => Some(graphics),
            Err(lowering_diagnostics) => {
                diagnostics.append(lowering_diagnostics);
                None
            }
        }
    });

    CompiledSource {
        document: parsed.document,
        graphics,
        diagnostics,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn compile(source: &str) -> CompiledSource {
        compile_source(source, FileId(0))
    }

    #[test]
    fn valid_source_compiles_to_graphics_ir() {
        let compiled = compile(
            "size 200px x 100px\nbackground white\nrectangle at (10px, 10px)\n    size 20px x 30px\n    fill red\n",
        );
        assert!(
            !compiled.has_errors(),
            "diagnostics: {:?}",
            compiled.diagnostics.as_slice()
        );
        let graphics = compiled.graphics.expect("graphics");
        assert_eq!((graphics.width, graphics.height), (200, 100));
        assert_eq!(graphics.shapes().len(), 1);
    }

    #[test]
    fn invalid_source_produces_no_graphics() {
        let compiled = compile("size 0px x 100px\n");
        assert!(compiled.has_errors());
        assert!(compiled.graphics.is_none());
    }

    #[test]
    fn invalid_shape_paint_produces_no_graphics() {
        let compiled = compile(
            "size 100px x 100px\nrectangle at (0px, 0px)\n    size 10px x 10px\n    fill colour\n",
        );
        assert!(compiled.has_errors());
        assert!(compiled.graphics.is_none());
    }

    #[test]
    fn parse_source_treats_file_as_document() {
        let parsed = parse_source("size 10px x 10px\n", FileId(0));
        let document = parsed.document.as_ref().expect("file is a document");
        assert!(document.size.is_some());
        assert!(
            !parsed.has_errors(),
            "diagnostics: {:?}",
            parsed.diagnostics
        );
    }

    #[test]
    fn picture_lowers_to_picture_node_with_defaults() {
        let compiled = compile(
            "size 200px x 100px\npicture \"photo.png\"\n    at (10px, 20px)\n    size 80px x 40px\n",
        );
        assert!(
            !compiled.has_errors(),
            "diagnostics: {:?}",
            compiled.diagnostics.as_slice()
        );
        let graphics = compiled.graphics.expect("graphics");
        assert_eq!(graphics.pictures().len(), 1);
        let picture = graphics.pictures()[0];
        assert_eq!(picture.path, "photo.png");
        assert_eq!((picture.x, picture.y), (10.0, 20.0));
        assert_eq!(picture.width, Some(80.0));
        assert_eq!(picture.height, Some(40.0));
        assert_eq!(picture.fit, rupa_ir::document::FitMode::Contain);
        assert_eq!(picture.opacity, 1.0);
    }

    #[test]
    fn picture_without_position_is_rejected() {
        let compiled = compile("size 200px x 100px\npicture \"photo.png\"\n    size 80px x 40px\n");
        assert!(compiled.has_errors());
        assert!(compiled.graphics.is_none());
    }
}
