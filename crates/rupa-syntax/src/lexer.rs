//! Indentation-aware lexer for Rupa.

use crate::diagnostic::{DiagnosticBag, codes};
use crate::intern::IStr;
use crate::span::{FileId, MAX_SOURCE_BYTES, Position, Span};
use crate::token::{Token, TokenKind};

/// A `//` comment, preserved as a side channel for the formatter.
///
/// Comments never reach the parser; they are kept separately so `rupa fmt` can
/// re-emit them instead of dropping them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comment {
    /// Span covering the `//` and the comment text up to the line break.
    pub span: Span,
    /// Comment text as written, including the leading `//`.
    pub text: String,
}

/// Lexer for Rupa source code
pub struct Lexer {
    /// Source code string
    source: String,
    /// File identifier
    file_id: FileId,
    /// Current position in source (byte offset)
    pos: usize,
    /// Indentation level stack
    indent_stack: Vec<usize>,
    /// Whether at start of line (for indentation handling)
    at_line_start: bool,
    /// Whether we're at the very beginning of the file
    at_file_start: bool,
    /// Generated tokens
    tokens: Vec<Token>,
    /// Comments encountered while scanning, in source order
    comments: Vec<Comment>,
    /// Collected diagnostics
    diagnostics: DiagnosticBag,
}

impl Lexer {
    /// Create a new lexer
    pub fn new(source: String, file_id: FileId) -> Self {
        Self {
            source,
            file_id,
            pos: 0,
            indent_stack: vec![0],
            at_line_start: true,
            at_file_start: true,
            tokens: Vec::new(),
            comments: Vec::new(),
            diagnostics: DiagnosticBag::new(),
        }
    }

    /// Tokenize the source code.
    pub fn tokenize(self) -> (Vec<Token>, DiagnosticBag) {
        let (tokens, _comments, diagnostics) = self.tokenize_with_comments();
        (tokens, diagnostics)
    }

    /// Tokenize the source code, also returning the comments in source order.
    ///
    /// The formatter uses the comments to preserve them; the parser ignores
    /// them entirely.
    pub fn tokenize_with_comments(mut self) -> (Vec<Token>, Vec<Comment>, DiagnosticBag) {
        if self.source.len() > MAX_SOURCE_BYTES as usize {
            let mut offset = MAX_SOURCE_BYTES as usize;
            while offset > 0 && !self.source.is_char_boundary(offset) {
                offset -= 1;
            }
            let span = Span::single(self.file_id, Self::position(offset));
            self.diagnostics.error(
                codes::SOURCE_TOO_LARGE,
                format!(
                    "source file is {} bytes; the maximum is {} bytes (16 MiB - 1)",
                    self.source.len(),
                    MAX_SOURCE_BYTES
                ),
                span,
            );
            self.tokens.push(Token::eof(span));
            return (self.tokens, self.comments, self.diagnostics);
        }

        while !self.is_eof() {
            if self.at_line_start {
                self.handle_indentation();
            }
            self.scan_token();
        }
        self.emit_dedents();
        self.tokens.push(Token::eof(self.current_span()));
        (self.tokens, self.comments, self.diagnostics)
    }

    fn position(offset: usize) -> Position {
        Position::new(u32::try_from(offset).unwrap_or(MAX_SOURCE_BYTES))
    }

    fn current_span(&self) -> Span {
        Span::single(self.file_id, Self::position(self.pos))
    }

    fn span_from_start(&self, start_offset: usize) -> Span {
        Span::new(
            self.file_id,
            Self::position(start_offset),
            Self::position(self.pos),
        )
    }

    fn push_token(&mut self, kind: TokenKind, start_offset: usize) {
        let span = self.span_from_start(start_offset);
        self.tokens.push(Token::new(kind, span));
    }

    /// Advance over a token of `width` characters and emit it.
    fn emit(&mut self, kind: TokenKind, start_offset: usize, width: usize) {
        for _ in 0..width {
            self.advance();
        }
        self.push_token(kind, start_offset);
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.source.len()
    }

    fn current_char(&self) -> Option<char> {
        self.source[self.pos..].chars().next()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.current_char();
        if let Some(ch) = ch {
            self.pos += ch.len_utf8();
            self.at_line_start = ch == '\n';
        }
        ch
    }

    /// Peek at the next character (after the current one)
    fn peek_next(&self) -> Option<char> {
        let ch = self.current_char()?;
        let next_pos = self.pos + ch.len_utf8();
        self.source[next_pos..].chars().next()
    }

    fn handle_indentation(&mut self) {
        // Skip blank lines and comment-only lines - they don't affect indentation
        // Loop until we find a non-blank line or reach EOF
        loop {
            if self.is_eof() {
                return;
            }
            if self.is_blank_or_comment_line() {
                // Comment-only lines do not affect indentation, but the
                // formatter still needs their text.
                self.record_comment_only_line();
                self.skip_line();
                // Continue to next line
                continue;
            }
            break;
        }

        let start_offset = self.pos;

        let mut spaces = 0;
        while matches!(self.current_char(), Some(' ')) {
            self.advance();
            spaces += 1;
        }

        // Check for tabs in indentation
        if matches!(self.current_char(), Some('\t')) {
            let span = self.span_from_start(start_offset);
            self.diagnostics.error_with_help(
                codes::TAB_CHARACTER,
                "Tab characters are not allowed",
                span,
                "Use 4 spaces for indentation",
            );
            // Skip the tab
            self.advance();
        }

        if spaces % 4 != 0 {
            let span = self.span_from_start(start_offset);
            self.diagnostics.error_with_help(
                codes::INVALID_INDENTATION,
                format!(
                    "Invalid indentation: {} spaces (must be multiple of 4)",
                    spaces
                ),
                span,
                "Use 4 spaces per indentation level",
            );
        }

        let level = spaces / 4;
        let current_level = *self.indent_stack.last().unwrap_or(&0);

        if level > current_level {
            if level > current_level + 1 {
                let span = self.span_from_start(start_offset);
                self.diagnostics.error_with_help(
                    codes::INVALID_INDENTATION,
                    format!(
                        "Indentation increase too large: {} -> {}",
                        current_level, level
                    ),
                    span,
                    "Increase indentation by exactly one level (4 spaces)",
                );
            }
            self.indent_stack.push(level);
            self.push_token(TokenKind::Indent, start_offset);
        } else {
            let mut current_level = *self.indent_stack.last().unwrap_or(&0);
            while level < current_level {
                self.indent_stack.pop();
                self.push_token(TokenKind::Dedent, start_offset);
                current_level = *self.indent_stack.last().unwrap_or(&0);
            }
        }

        self.at_line_start = false;
        self.at_file_start = false;
    }

    fn is_blank_or_comment_line(&self) -> bool {
        let remaining = &self.source[self.pos..];
        let line = remaining
            .split_once('\n')
            .map_or(remaining, |(line, _)| line);
        let line = line.trim_start_matches([' ', '\t', '\r']);
        line.is_empty() || line.starts_with("//")
    }

    /// Record a comment that occupies its whole line and nothing else.
    ///
    /// Comment-only lines are skipped for indentation purposes, so they never
    /// pass through `scan_token`; capture the comment here instead.
    fn record_comment_only_line(&mut self) {
        while matches!(self.current_char(), Some(' ')) {
            self.advance();
        }
        if self.current_char() == Some('/') && self.peek_next() == Some('/') {
            self.scan_comment();
        }
    }

    fn skip_line(&mut self) {
        while let Some(ch) = self.current_char() {
            if ch == '\n' {
                self.advance();
                break;
            }
            self.advance();
        }
        self.at_line_start = true;
    }

    fn emit_dedents(&mut self) {
        // Use the current position for dedents at EOF
        let span = self.current_span();
        while self.indent_stack.len() > 1 {
            self.indent_stack.pop();
            self.tokens.push(Token::new(TokenKind::Dedent, span));
        }
    }

    fn scan_token(&mut self) {
        let start_offset = self.pos;

        let ch = match self.current_char() {
            Some(c) => c,
            None => return,
        };

        match ch {
            ' ' | '\r' => {
                self.advance();
            }
            '\t' => {
                let span = self.span_from_start(start_offset);
                self.diagnostics.error_with_help(
                    codes::TAB_CHARACTER,
                    "Tab characters are not allowed",
                    span,
                    "Use 4 spaces for indentation",
                );
                self.advance();
            }
            '\n' => self.emit(TokenKind::Newline, start_offset, 1),
            '/' if self.peek_next() == Some('/') => self.scan_comment(),
            '/' => self.emit(TokenKind::Slash, start_offset, 1),
            '"' => self.scan_string(start_offset),
            '0'..='9' => self.scan_number(start_offset),
            'a'..='z' | 'A'..='Z' | '_' => self.scan_identifier(start_offset),
            '+' => self.emit(TokenKind::Plus, start_offset, 1),
            '-' => self.emit(TokenKind::Minus, start_offset, 1),
            '*' => self.emit(TokenKind::Star, start_offset, 1),
            '%' => self.emit(TokenKind::Percent, start_offset, 1),
            '=' if self.peek_next() == Some('=') => self.emit(TokenKind::Eq, start_offset, 2),
            '=' => self.emit(TokenKind::Assign, start_offset, 1),
            '!' if self.peek_next() == Some('=') => self.emit(TokenKind::Ne, start_offset, 2),
            '!' => {
                self.advance();
                let span = self.span_from_start(start_offset);
                self.diagnostics.error_with_help(
                    codes::INVALID_CHARACTER,
                    "Unexpected character '!', did you mean '!='?",
                    span,
                    "Use '!=' for not-equal comparison",
                );
            }
            '<' if self.peek_next() == Some('=') => self.emit(TokenKind::Le, start_offset, 2),
            '<' => self.emit(TokenKind::Lt, start_offset, 1),
            '>' if self.peek_next() == Some('=') => self.emit(TokenKind::Ge, start_offset, 2),
            '>' => self.emit(TokenKind::Gt, start_offset, 1),
            '(' => self.emit(TokenKind::LParen, start_offset, 1),
            ')' => self.emit(TokenKind::RParen, start_offset, 1),
            '{' => self.emit(TokenKind::LBrace, start_offset, 1),
            '}' => self.emit(TokenKind::RBrace, start_offset, 1),
            '[' => self.emit(TokenKind::LBracket, start_offset, 1),
            ']' => self.emit(TokenKind::RBracket, start_offset, 1),
            ',' => self.emit(TokenKind::Comma, start_offset, 1),
            ':' => self.emit(TokenKind::Colon, start_offset, 1),
            ';' => self.emit(TokenKind::Semicolon, start_offset, 1),
            '#' => self.emit(TokenKind::Hash, start_offset, 1),
            '@' => self.emit(TokenKind::AtSign, start_offset, 1),
            _ => {
                // Unknown character
                let span = self.span_from_start(start_offset);
                self.diagnostics.error(
                    codes::INVALID_CHARACTER,
                    format!("Invalid character: '{ch}'"),
                    span,
                );
                self.advance();
            }
        }
    }

    fn scan_comment(&mut self) {
        let start_offset = self.pos;

        // Skip "//"
        self.advance();
        self.advance();
        while let Some(ch) = self.current_char() {
            if ch == '\n' {
                break;
            }
            self.advance();
        }

        let span = self.span_from_start(start_offset);
        let text = self.source[start_offset..self.pos].trim_end().to_string();
        self.comments.push(Comment { span, text });
    }

    fn scan_string(&mut self, start_offset: usize) {
        self.advance(); // Skip opening quote
        let mut value = String::new();

        while let Some(ch) = self.current_char() {
            if ch == '"' {
                self.advance(); // Skip closing quote
                self.push_token(TokenKind::String(value.into_boxed_str()), start_offset);
                return;
            } else if ch == '\\' {
                self.advance();
                if let Some(escaped) = self.current_char() {
                    match escaped {
                        'n' => value.push('\n'),
                        't' => value.push('\t'),
                        'r' => value.push('\r'),
                        '\\' => value.push('\\'),
                        '"' => value.push('"'),
                        _ => {
                            let span = self.current_span();
                            self.diagnostics.error_with_help(
                                codes::INVALID_ESCAPE,
                                format!("Invalid escape sequence: '\\{}'", escaped),
                                span,
                                "Valid escapes: \\n, \\t, \\r, \\\\, \\\"",
                            );
                            value.push(escaped);
                        }
                    }
                    self.advance();
                }
            } else if ch == '\n' {
                self.finish_unterminated_string(start_offset, value);
                return;
            } else {
                value.push(ch);
                self.advance();
            }
        }
        // EOF without closing quote
        self.finish_unterminated_string(start_offset, value);
    }

    fn finish_unterminated_string(&mut self, start_offset: usize, value: String) {
        let span = self.current_span();
        self.diagnostics.error_with_help(
            codes::UNTERMINATED_STRING,
            "Unterminated string literal",
            span,
            "Close the string with a double quote",
        );
        self.push_token(TokenKind::String(value.into_boxed_str()), start_offset);
    }

    fn scan_number(&mut self, start_offset: usize) {
        let mut value = String::new();
        let mut is_float = false;

        while let Some(ch) = self.current_char() {
            if ch.is_ascii_digit() {
                value.push(ch);
                self.advance();
            } else if ch == '.' && !is_float {
                // Check if next char is a digit (to avoid consuming . as part of .. or similar)
                if let Some(next_ch) = self.peek_next() {
                    if next_ch.is_ascii_digit() {
                        is_float = true;
                        value.push(ch);
                        self.advance();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Check for unit suffix.
        let mut unit = None;
        if let Some(ch) = self.current_char()
            && (ch.is_ascii_alphabetic() || ch == '%')
        {
            let unit_start_offset = self.pos;
            let mut unit_str = String::new();
            while let Some(ch) = self.current_char() {
                if ch.is_ascii_alphabetic() || ch == '%' {
                    unit_str.push(ch);
                    self.advance();
                } else {
                    break;
                }
            }
            let span = Span::new(
                self.file_id,
                Self::position(unit_start_offset),
                Self::position(self.pos),
            );
            match crate::metadata::unit_token(&unit_str) {
                Some(kind) => unit = Some((kind, span)),
                None => self.diagnostics.error_with_help(
                    codes::UNKNOWN_UNIT,
                    format!("Unknown unit: '{}'", unit_str),
                    span,
                    "Valid units: px, pt, deg, %",
                ),
            }
        }

        let num_value: f64 = value.parse().unwrap_or(0.0);
        self.push_token(TokenKind::Number(num_value), start_offset);
        if let Some((unit, span)) = unit {
            self.tokens.push(Token::new(TokenKind::Unit(unit), span));
        }
    }

    fn scan_identifier(&mut self, start_offset: usize) {
        while let Some(ch) = self.current_char() {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                self.advance();
            } else {
                break;
            }
        }

        // Slice the already-owned source instead of building a temporary
        // `String`; only the intern table stores the identifier payload.
        let value = &self.source[start_offset..self.pos];
        let kind = self.keyword_or_ident(value);
        self.push_token(kind, start_offset);
    }

    fn keyword_or_ident(&self, value: &str) -> TokenKind {
        crate::metadata::keyword_token(value)
            .unwrap_or_else(|| TokenKind::Identifier(IStr::new(value)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::{FileId, LineIndex};
    use crate::token::Unit;

    fn lex(input: &str) -> (Vec<Token>, DiagnosticBag) {
        let lexer = Lexer::new(input.to_string(), FileId(0));
        lexer.tokenize()
    }

    macro_rules! lex_ok {
        ($source:expr) => {{
            let (tokens, diags) = lex($source);
            assert!(!diags.has_errors(), "Diagnostics: {:?}", diags.as_slice());
            tokens
        }};
    }

    macro_rules! assert_tokens {
        ($source:expr, [$($kind:expr),* $(,)?]) => {{
            let tokens = lex_ok!($source);
            let kinds: Vec<_> = tokens.iter().map(|token| &token.kind).collect();
            assert_eq!(kinds, vec![$(&$kind),*]);
        }};
    }

    #[test]
    fn source_over_the_16_mib_limit_reports_l010_and_stops() {
        let source = " ".repeat(MAX_SOURCE_BYTES as usize + 1);
        let (tokens, diags) = lex(&source);

        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0].kind, TokenKind::Eof));
        assert_eq!(diags.as_slice().len(), 1);
        let diagnostic = &diags.as_slice()[0];
        assert_eq!(diagnostic.code, codes::SOURCE_TOO_LARGE);
        assert_eq!(diagnostic.primary_span.start.offset(), MAX_SOURCE_BYTES);
    }

    #[test]
    fn test_simple_keywords() {
        assert_tokens!(
            "size background rectangle circle",
            [
                TokenKind::Size,
                TokenKind::Background,
                TokenKind::Rectangle,
                TokenKind::Circle,
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_shape_keywords() {
        assert_tokens!(
            "rectangle circle at center radius fill x",
            [
                TokenKind::Rectangle,
                TokenKind::Circle,
                TokenKind::At,
                TokenKind::Center,
                TokenKind::Radius,
                TokenKind::Fill,
                TokenKind::X,
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_indentation() {
        assert_tokens!(
            "rectangle\n    size 100px x 100px",
            [
                TokenKind::Rectangle,
                TokenKind::Newline,
                TokenKind::Indent,
                TokenKind::Size,
                TokenKind::Number(100.0),
                TokenKind::Unit(Unit::Px),
                TokenKind::X,
                TokenKind::Number(100.0),
                TokenKind::Unit(Unit::Px),
                TokenKind::Dedent,
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_invalid_indentation_not_multiple_of_4() {
        let (tokens, diags) = lex("rectangle\n   size 100px x 100px");
        assert!(diags.has_errors());
        let kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
        // Should still produce tokens but with error
        assert_eq!(kinds[0], &TokenKind::Rectangle);
    }

    #[test]
    fn invalid_sources_report_lex_errors() {
        let cases = [
            (
                "indentation increase too large",
                "rectangle\n        size 100px x 100px",
            ),
            ("tab at line start", "rectangle\n\tsize 100px x 100px"),
            ("tab mid-line", "size\t100px"),
            ("unknown unit", "100em"),
            ("invalid escape", r#""hello\z""#),
            ("unterminated string", r#""hello"#),
            ("single exclamation", "!"),
            ("invalid character", "$"),
        ];

        for (name, source) in cases {
            let (_tokens, diags) = lex(source);
            assert!(diags.has_errors(), "expected lexer errors for {name}");
        }
    }

    #[test]
    fn test_numbers_with_units() {
        assert_tokens!(
            "100px 50.5pt 30deg 100%",
            [
                TokenKind::Number(100.0),
                TokenKind::Unit(Unit::Px),
                TokenKind::Number(50.5),
                TokenKind::Unit(Unit::Pt),
                TokenKind::Number(30.0),
                TokenKind::Unit(Unit::Deg),
                TokenKind::Number(100.0),
                TokenKind::Unit(Unit::Percent),
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_string_literals() {
        assert_tokens!(
            r#""hello" "world""#,
            [
                TokenKind::String("hello".into()),
                TokenKind::String("world".into()),
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_string_escapes() {
        let tokens = lex_ok!(r#""hello\nworld\t\r\\""#);
        let TokenKind::String(value) = &tokens[0].kind else {
            panic!("Expected string token");
        };
        assert_eq!(value.as_ref(), "hello\nworld\t\r\\");
    }

    #[test]
    fn test_operators() {
        assert_tokens!(
            "+ - * / % == != < <= > >=",
            [
                TokenKind::Plus,
                TokenKind::Minus,
                TokenKind::Star,
                TokenKind::Slash,
                TokenKind::Percent,
                TokenKind::Eq,
                TokenKind::Ne,
                TokenKind::Lt,
                TokenKind::Le,
                TokenKind::Gt,
                TokenKind::Ge,
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_assign_vs_eq() {
        assert_tokens!("= ==", [TokenKind::Assign, TokenKind::Eq, TokenKind::Eof]);
    }

    #[test]
    fn test_punctuation() {
        assert_tokens!(
            "( ) { } [ ] , : ; # @",
            [
                TokenKind::LParen,
                TokenKind::RParen,
                TokenKind::LBrace,
                TokenKind::RBrace,
                TokenKind::LBracket,
                TokenKind::RBracket,
                TokenKind::Comma,
                TokenKind::Colon,
                TokenKind::Semicolon,
                TokenKind::Hash,
                TokenKind::AtSign,
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_comments_skipped() {
        assert_tokens!(
            "size // comment\n100px",
            [
                TokenKind::Size,
                TokenKind::Newline,
                TokenKind::Number(100.0),
                TokenKind::Unit(Unit::Px),
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_blank_lines_ignored_for_indentation() {
        let tokens = lex_ok!("rectangle\n\n    size 100px x 100px\n\n");
        let kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
        // Should have Indent after blank line, not Dedent/Indent for blank line
        assert!(kinds.iter().any(|k| matches!(k, TokenKind::Indent)));
        assert!(kinds.iter().any(|k| matches!(k, TokenKind::Dedent)));
    }

    #[test]
    fn test_comment_only_lines_ignored_for_indentation() {
        let tokens = lex_ok!("rectangle\n// comment\n    size 100px x 100px");
        let has_indent = tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::Indent));
        assert!(has_indent);
    }

    #[test]
    fn test_identifiers() {
        assert_tokens!(
            "myVar _private var123",
            [
                TokenKind::Identifier(IStr::new("myVar")),
                TokenKind::Identifier(IStr::new("_private")),
                TokenKind::Identifier(IStr::new("var123")),
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_boolean_literals() {
        assert_tokens!(
            "true false",
            [TokenKind::True, TokenKind::False, TokenKind::Eof]
        );
    }

    #[test]
    fn test_dedents_at_eof() {
        let tokens = lex_ok!("rectangle\n    size 100px x 100px\n    background white");
        let dedent_count = tokens
            .iter()
            .filter(|token| matches!(token.kind, TokenKind::Dedent))
            .count();
        assert_eq!(dedent_count, 1);
    }

    #[test]
    fn test_nested_indentation() {
        let tokens = lex_ok!(
            "layer \"L\"\n    rectangle at (0px, 0px)\n        size 100px x 100px\n        fill red"
        );
        let indent_count = tokens
            .iter()
            .filter(|token| matches!(token.kind, TokenKind::Indent))
            .count();
        let dedent_count = tokens
            .iter()
            .filter(|token| matches!(token.kind, TokenKind::Dedent))
            .count();
        assert_eq!(indent_count, 2);
        assert_eq!(dedent_count, 2);
    }

    #[test]
    fn test_spans_use_byte_offsets_and_character_columns() {
        let tokens = lex_ok!("\"é\" size");

        let index = LineIndex::new("\"é\" size");
        let string = &tokens[0];
        assert!(matches!(string.kind, TokenKind::String(_)));
        assert_eq!(index.line_col(string.span.start_position()).column, 1);
        assert_eq!(string.span.start.offset(), 0);
        assert_eq!(index.line_col(string.span.end_position()).column, 4);
        assert_eq!(string.span.end.offset(), 4);

        let size = &tokens[1];
        assert_eq!(size.kind, TokenKind::Size);
        assert_eq!(index.line_col(size.span.start_position()).column, 5);
        assert_eq!(size.span.start.offset(), 5);
    }

    #[test]
    fn test_keyword_with_hyphen() {
        assert_tokens!(
            "z-index miter-limit hue-rotate displacement-map",
            [
                TokenKind::ZIndex,
                TokenKind::MiterLimit,
                TokenKind::HueRotate,
                TokenKind::DisplacementMap,
                TokenKind::Eof
            ]
        );
    }
}
