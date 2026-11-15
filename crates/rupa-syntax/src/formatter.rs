//! Canonical formatter for Rupa source.

use crate::diagnostic::{DiagnosticBag, Severity};
use crate::lexer::{Comment, Lexer};
use crate::span::{FileId, LineIndex};
use crate::token::TokenKind;

/// Format a Rupa source string into its canonical representation.
///
/// Returns `Ok(formatted)` on success, or the lexer diagnostics when the
/// source cannot be tokenized.
pub fn format_source(source: &str) -> Result<String, DiagnosticBag> {
    let (_, diagnostics) = Lexer::new(source.to_string(), FileId(0)).tokenize();
    if diagnostics.iter().any(|d| d.severity == Severity::Error) {
        return Err(diagnostics);
    }
    // File-level document settings are canonicalized before token formatting:
    // `size`, `background`, `seed`, followed by the rest of the file. Only
    // column-zero settings are moved; indented properties are left inside their
    // blocks.
    let canonical = canonicalize_document_settings(source);
    let (tokens, comments, diagnostics) =
        Lexer::new(canonical.clone(), FileId(0)).tokenize_with_comments();
    if diagnostics.iter().any(|d| d.severity == Severity::Error) {
        return Err(diagnostics);
    }
    let source = canonical.as_str();
    let line_index = LineIndex::new(source);

    let mut out = String::new();
    let mut indent: usize = 0;
    let mut at_line_start = true;
    let mut prev_char: Option<char> = None;
    // Tracks the previous significant token so a unary `-` can stay attached
    // to its operand (`rotate -8deg`) while a binary `-` keeps its spaces
    // (`10px - 5px`).
    let mut prev_kind: Option<TokenKind> = None;
    let mut minus_is_unary = false;
    // Tracks whether a blank line should be inserted before the next
    // top-level (indent 0) block. A single blank line separates consecutive
    // top-level blocks; consecutive document settings (size/background/seed)
    // stay adjacent.
    let mut first_line = true;
    let mut prev_was_setting = false;
    // Bracket depth, so an inline property break never splits `(x, y)`.
    let mut paren_depth: usize = 0;
    // Comments are emitted lazily, in source order, before the next token.
    let mut comment_index = 0usize;
    // A comment flush ends the current line, so the following newline token is
    // redundant and would otherwise produce a blank line (or grow on re-format).
    let mut comment_ended_line = false;

    for token in tokens {
        let flushed = flush_comments(
            &mut out,
            &comments,
            &mut comment_index,
            token.span.start.offset(),
            &line_index,
            &mut at_line_start,
        );
        if flushed {
            prev_char = None;
            prev_kind = None;
            minus_is_unary = false;
            first_line = false;
            comment_ended_line = true;
        }

        // An inline property list is expanded to the canonical layout: one
        // property per indented line (specification §2.5).
        if !at_line_start && paren_depth == 0 && is_property_start(&token.kind) {
            out.push('\n');
            for _ in 0..=indent {
                out.push_str("    ");
            }
            prev_char = None;
            prev_kind = None;
            minus_is_unary = false;
        }

        match token.kind {
            TokenKind::Indent => {
                indent += 1;
            }
            TokenKind::Dedent => {
                indent = indent.saturating_sub(1);
            }
            TokenKind::Newline => {
                if comment_ended_line {
                    comment_ended_line = false;
                } else {
                    out.push('\n');
                }
                at_line_start = true;
                prev_kind = None;
                minus_is_unary = false;
            }
            TokenKind::Eof => break,
            _ => {
                let text_owned: Option<String> = match &token.kind {
                    TokenKind::Number(n) => Some(format!("{}", n)),
                    TokenKind::Unit(u) => Some(match u {
                        crate::token::Unit::Px => "px".to_string(),
                        crate::token::Unit::Pt => "pt".to_string(),
                        crate::token::Unit::Percent => "%".to_string(),
                        crate::token::Unit::Deg => "deg".to_string(),
                    }),
                    _ => None,
                };
                let start = token.span.start.offset_usize();
                let end = token.span.end.offset_usize();
                let borrowed = match (start, end) {
                    (Some(start), Some(end)) if start < end && end <= source.len() => {
                        &source[start..end]
                    }
                    _ => "",
                };
                let text = text_owned.as_deref().unwrap_or(borrowed);

                if at_line_start {
                    let is_top_level = indent == 0;
                    let is_setting = matches!(
                        token.kind,
                        TokenKind::Size | TokenKind::Background | TokenKind::Seed
                    );
                    if is_top_level && !first_line && !(prev_was_setting && is_setting) {
                        // Preserve a single blank line between top-level blocks.
                        if !out.ends_with("\n\n") {
                            out.push('\n');
                        }
                    }
                    for _ in 0..indent {
                        out.push_str("    ");
                    }
                    at_line_start = false;
                    first_line = false;
                    prev_was_setting = is_top_level && is_setting;
                } else {
                    let no_space_before = minus_is_unary
                        || matches!(
                            token.kind,
                            TokenKind::RParen
                                | TokenKind::RBracket
                                | TokenKind::RBrace
                                | TokenKind::Comma
                                | TokenKind::Colon
                                | TokenKind::Semicolon
                                | TokenKind::Unit(_)
                        );
                    let prev_no_space = prev_char
                        .map(|c| c == '(' || c == '[' || c == '{' || c == '#' || c == '\n')
                        .unwrap_or(true);
                    if !no_space_before && !prev_no_space {
                        out.push(' ');
                    }
                }

                out.push_str(text);
                // The comment already closed its line; this token starts a new one.
                comment_ended_line = false;
                prev_char = text.chars().last();
                match token.kind {
                    TokenKind::LParen | TokenKind::LBracket => paren_depth += 1,
                    TokenKind::RParen | TokenKind::RBracket => {
                        paren_depth = paren_depth.saturating_sub(1);
                    }
                    _ => {}
                }
                let unary_minus = matches!(token.kind, TokenKind::Minus)
                    && !prev_kind.as_ref().is_some_and(is_value_token);
                prev_kind = Some(token.kind.clone());
                minus_is_unary = unary_minus;
            }
        }
    }

    // Comments after the last token (end of file) are still preserved.
    flush_comments(
        &mut out,
        &comments,
        &mut comment_index,
        u32::MAX,
        &line_index,
        &mut at_line_start,
    );

    // Trim trailing whitespace on each line and trailing blank lines.
    let mut result = String::new();
    for line in out.lines() {
        result.push_str(line.trim_end());
        result.push('\n');
    }
    while result.ends_with("\n\n") {
        result.pop();
    }
    if !result.ends_with('\n') && !result.is_empty() {
        result.push('\n');
    }
    Ok(result)
}

/// Convenience: parse-free formatting entry point used by the CLI.
pub fn format_file(source: &str) -> Result<String, DiagnosticBag> {
    format_source(source)
}

/// Emit pending comments whose position precedes `before_offset`.
///
/// Returns whether anything was written. Comment-only lines are indented to the
/// current block depth; trailing comments stay at the end of their line.
fn flush_comments(
    out: &mut String,
    comments: &[Comment],
    index: &mut usize,
    before_offset: u32,
    line_index: &LineIndex<'_>,
    at_line_start: &mut bool,
) -> bool {
    let mut wrote = false;
    while *index < comments.len() && comments[*index].span.start.offset() < before_offset {
        let comment = &comments[*index];
        if *at_line_start {
            // Keep the authored nesting level: a comment inside a block is
            // flushed before the Indent token, so block depth is not available.
            let depth = line_index
                .line_col(comment.span.start_position())
                .column
                .saturating_sub(1)
                / 4;
            for _ in 0..depth {
                out.push_str("    ");
            }
        } else {
            out.push(' ');
        }
        out.push_str(&comment.text);
        out.push('\n');
        *at_line_start = true;
        *index += 1;
        wrote = true;
    }
    wrote
}

/// Whether a token starts a property that the canonical layout puts on its own
/// line.
///
/// Header keywords (`at`, `center`, `from`, `to`) are deliberately excluded so
/// `rectangle at (10px, 10px)` and `line from … to …` stay on one line.
fn is_property_start(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Size
            | TokenKind::Radius
            | TokenKind::Fill
            | TokenKind::Stroke
            | TokenKind::Width
            | TokenKind::Cap
            | TokenKind::Join
            | TokenKind::Rotate
            | TokenKind::Scale
            | TokenKind::Opacity
            | TokenKind::Font
            | TokenKind::Weight
            | TokenKind::Align
            | TokenKind::Fit
            | TokenKind::ZIndex
            | TokenKind::Blend
            | TokenKind::Effect
            | TokenKind::Mode
            | TokenKind::Clip
            | TokenKind::Points
    )
}

/// Whether a token can end a value, making a following `-` binary.
fn is_value_token(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Number(_)
            | TokenKind::Unit(_)
            | TokenKind::String(_)
            | TokenKind::Identifier(_)
            | TokenKind::RParen
            | TokenKind::RBracket
            | TokenKind::True
            | TokenKind::False
    )
}

/// Move top-level document settings into canonical order.
fn canonicalize_document_settings(source: &str) -> String {
    let mut size = Vec::new();
    let mut background = Vec::new();
    let mut seed = Vec::new();
    let mut rest = Vec::new();

    for line in source.lines() {
        let top_level = line
            .chars()
            .next()
            .is_some_and(|character| !character.is_whitespace());
        let keyword = if top_level {
            line.split_ascii_whitespace().next().unwrap_or("")
        } else {
            ""
        };
        match keyword {
            "size" => size.push(line),
            "background" => background.push(line),
            "seed" => seed.push(line),
            _ => rest.push(line),
        }
    }

    if size.is_empty() && background.is_empty() && seed.is_empty() {
        return source.to_string();
    }

    let mut out = String::new();
    for line in size.into_iter().chain(background).chain(seed) {
        out.push_str(line);
        out.push('\n');
    }

    let first_content = rest
        .iter()
        .position(|line| !line.trim().is_empty())
        .unwrap_or(rest.len());
    let rest = &rest[first_content..];
    if !rest.is_empty() {
        out.push('\n');
        for line in rest {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_file_level_source() {
        let source = "size  100px x 100px\nbackground white\n\nrectangle at (10px, 10px)\n    size 20px x 20px\n    fill red\n";
        let formatted = format_source(source).expect("format");
        assert!(formatted.starts_with("size 100px x 100px\n"));
        assert!(formatted.contains("\nrectangle at (10px, 10px)\n"));
        assert_eq!(format_source(&formatted).expect("idempotent"), formatted);
    }

    #[test]
    fn canonicalizes_document_setting_order() {
        let source = "background white\nrectangle at (0px, 0px)\n    size 10px x 10px\n    fill red\nsize 100px x 100px\n";
        let formatted = format_source(source).expect("format");
        assert!(formatted.starts_with("size 100px x 100px\nbackground white\n\nrectangle"));
        assert_eq!(format_source(&formatted).expect("idempotent"), formatted);
    }

    #[test]
    fn does_not_reorder_indented_properties_named_like_settings() {
        let source =
            "size 100px x 100px\nrectangle at (0px, 0px)\n    size 10px x 10px\n    fill red\n";
        let formatted = format_source(source).expect("format");
        assert!(formatted.starts_with("size 100px x 100px\n\nrectangle"));
        assert!(formatted.contains("\n    size 10px x 10px\n"));
    }

    #[test]
    fn preserves_group_and_layer_declarations() {
        let source = "size 200px x 100px\n\ngroup \"Tile\"\n    size 20px x 20px\n    rectangle at (0px, 0px)\n        size 20px x 20px\n        fill red\n\nlayer \"Sky\"\n    fill blue\n\n    place \"Tile\" at (10px, 10px)\n        z-index 1\n";
        let formatted = format_source(source).expect("format");
        assert!(formatted.contains("group \"Tile\"\n"));
        assert!(formatted.contains("    size 20px x 20px\n"));
        assert!(formatted.contains("layer \"Sky\"\n"));
        assert!(formatted.contains("place \"Tile\" at (10px, 10px)\n"));
        assert!(formatted.contains("        z-index 1\n"));
        assert_eq!(format_source(&formatted).expect("idempotent"), formatted);
    }

    #[test]
    fn keeps_unary_minus_attached_to_its_operand() {
        let source = "size 100px x 100px\nplace \"Dot\" at (0px, 0px)\n    rotate -8deg\n    z-index -1\n    scale -0.5\n";
        let formatted = format_source(source).expect("format");
        assert!(formatted.contains("rotate -8deg\n"), "{formatted}");
        assert!(formatted.contains("z-index -1\n"), "{formatted}");
        assert!(formatted.contains("scale -0.5\n"), "{formatted}");
        assert!(!formatted.contains("- 8"), "{formatted}");
        assert_eq!(format_source(&formatted).expect("idempotent"), formatted);
    }
}
