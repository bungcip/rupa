//! Diagnostic reporting for Rupa.
//!
//! Diagnostics carry a stable code (see [`codes`]), a severity, a message, a
//! primary source span, and optional help text. Use [`render_diagnostics`] to
//! pretty-print diagnostics with source context (the offending line and a
//! caret pointing at the span).

use crate::span::{LineIndex, Span};
use serde::{Deserialize, Serialize};

/// Known Rupa keywords used for "did you mean" suggestions.
pub const KNOWN_KEYWORDS: &[&str] = crate::metadata::ALL_KEYWORDS;

/// Edit distance between two strings (Levenshtein, iterative, O(n*m) space).
fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr = vec![0usize; b.len() + 1];
    for i in 1..=a.len() {
        curr[0] = i;
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b.len()]
}

/// Find the closest known keyword to `input`, if it is close enough to be a
/// plausible misspelling.
///
/// The threshold is proportional to the input length so short inputs are not
/// over-suggested. Ties are broken alphabetically for determinism.
pub fn did_you_mean(input: &str) -> Option<&'static str> {
    did_you_mean_from(input, KNOWN_KEYWORDS)
}

/// Like [`did_you_mean`], but against a custom candidate list.
pub fn did_you_mean_from<'a>(input: &str, candidates: &[&'a str]) -> Option<&'a str> {
    if input.is_empty() {
        return None;
    }
    let threshold = (input.chars().count() / 3).clamp(1, 3);
    candidates
        .iter()
        .copied()
        .filter(|c| edit_distance(input, c) <= threshold)
        .min_by_key(|c| (edit_distance(input, c), *c))
}

/// Severity of a diagnostic
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    /// Error severity
    Error,
    /// Warning severity
    Warning,
    /// Info severity
    Info,
    /// Help severity
    Help,
}

/// A source replacement that fixes a diagnostic deterministically.
///
/// Only diagnostics whose fix is exact carry one: a suggested keyword or a
/// suggested named colour. A host may apply the edit verbatim, while `help`
/// remains the human-readable explanation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SuggestedEdit {
    /// Span of the source text to replace.
    pub span: Span,
    /// Replacement text for that span.
    pub replacement: String,
}

/// A diagnostic message with source location
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Diagnostic code
    pub code: String,
    /// Severity level
    pub severity: Severity,
    /// Human-readable message
    pub message: String,
    /// Primary source span
    pub primary_span: Span,
    /// Optional help text
    pub help: Option<String>,
    /// Machine-applicable fixes; empty when the fix is not exact
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub edits: Vec<SuggestedEdit>,
}

impl Diagnostic {
    /// Create an error diagnostic.
    pub fn error(code: &str, message: impl Into<String>, primary_span: Span) -> Self {
        Self {
            code: code.to_string(),
            severity: Severity::Error,
            message: message.into(),
            primary_span,
            help: None,
            edits: Vec::new(),
        }
    }

    /// Create a warning diagnostic.
    pub fn warning(code: &str, message: impl Into<String>, primary_span: Span) -> Self {
        Self {
            severity: Severity::Warning,
            ..Self::error(code, message, primary_span)
        }
    }

    /// Attach help text.
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Attach a "did you mean `X`?" suggestion, appended to any existing help.
    ///
    /// The suggestion also records a [`SuggestedEdit`]: the primary span (the
    /// unknown keyword token) is replaced by the suggested keyword, so a host
    /// can fix the source without reading the help text.
    pub fn with_suggestion(mut self, input: &str) -> Self {
        if let Some(candidate) = suggest_keyword(input) {
            let text = format!("did you mean `{candidate}`?");
            self.help = match self.help {
                Some(existing) => Some(format!("{existing}; {text}")),
                None => Some(text),
            };
            self.edits.push(SuggestedEdit {
                span: self.primary_span,
                replacement: candidate.to_string(),
            });
        }
        self
    }
}

/// Metadata for one stable diagnostic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodeInfo {
    /// Stable code, e.g. `S005`.
    pub code: &'static str,
    /// Constant name in [`codes`], e.g. `MISSING_RADIUS`.
    pub name: &'static str,
    /// Short human-readable description of when the code is emitted.
    pub description: &'static str,
}

/// Define the stable diagnostic code constants and their catalog in one place.
///
/// Each entry is `NAME = "code", "description";`. The macro generates both the
/// `pub const NAME: &str` and the [`codes::ALL`] catalog slice, so a code
/// cannot be added without appearing in the machine-readable catalog (and
/// [`codes::describe`]).
macro_rules! diagnostic_codes {
    ($($name:ident = $code:literal, $description:literal;)*) => {
        /// Catalog of every canonical diagnostic code.
        ///
        /// Aliases (for example `RESOURCE_NOT_FOUND` for `MISSING_RESOURCE`)
        /// are not listed twice; they resolve to the canonical entry.
        pub const ALL: &[super::CodeInfo] = &[
            $(super::CodeInfo {
                code: $code,
                name: stringify!($name),
                description: $description,
            },)*
        ];

        /// Look up a code in the catalog.
        pub fn describe(code: &str) -> Option<&'static super::CodeInfo> {
            ALL.iter().find(|info| info.code == code)
        }

        $(
            #[doc = concat!("Stable code `", $code, "`: ", $description)]
            pub const $name: &str = $code;
        )*
    };
}

/// Stable diagnostic codes.
///
/// Codes are grouped by category prefix:
///
/// | Prefix | Category |
/// |--------|----------|
/// | `L`    | Lexer    |
/// | `P`    | Parser   |
/// | `C`    | Compiler/document |
/// | `S`    | Shapes, layers, expressions |
/// | `R`    | Resources |
/// | `T`    | Paint    |
///
/// Codes must only be added, never renumbered. See
/// `docs/decisions/DECISION-001.md`.
pub mod codes {
    diagnostic_codes! {
    // Lexer errors (Lxxx)
    INVALID_INDENTATION = "L001", "Indentation is not a multiple of four spaces.";
    TAB_CHARACTER = "L002", "A tab character was found; Rupa requires spaces.";
    UNTERMINATED_STRING = "L003", "A string literal is missing its closing quote.";
    INVALID_NUMBER = "L004", "A numeric literal is malformed.";
    INVALID_HEX_COLOR = "L005", "A hex color literal is malformed.";
    UNKNOWN_KEYWORD = "L006", "An identifier is not a known keyword, property, or name.";
    INVALID_CHARACTER = "L007", "A character is not valid in Rupa source.";
    INVALID_ESCAPE = "L008", "A string literal contains an invalid escape sequence.";
    UNKNOWN_UNIT = "L009", "A numeric suffix is not a supported unit.";
    SOURCE_TOO_LARGE = "L010", "A source file exceeds the 16 MiB - 1 byte source limit.";

    // Parser errors (Pxxx)
    EXPECTED_TOKEN = "P001", "A specific token or construct was expected.";
    MISSING_REQUIRED_PROPERTY = "P002", "A required property was not provided.";
    INVALID_PROPERTY_VALUE = "P003", "A property value has the wrong shape or type.";

    // Compiler errors (Cxxx)
    INVALID_DIMENSIONS = "C003", "A width or height is not positive.";
    NON_PIXEL_DIMENSION = "C007", "A document dimension does not use pixel units.";
    CANVAS_TOO_LARGE = "C008", "Document dimensions exceed the supported PNG canvas.";
    DUPLICATE_DOCUMENT_SIZE = "C009", "The document declares `size` more than once.";
    DUPLICATE_BACKGROUND = "C010", "The document declares `background` more than once.";
    DUPLICATE_SEED = "C011", "The document declares `seed` more than once.";

    // Shape, layer, and expression errors (Sxxx)
    NEGATIVE_DIMENSION = "S001", "A shape dimension is zero or negative.";
    NEGATIVE_RADIUS = "S002", "A radius is zero or negative.";
    MISSING_POSITION = "S003", "A shape or group instance is missing its position.";
    MISSING_SIZE = "S004", "A shape is missing its size.";
    MISSING_RADIUS = "S005", "A circle is missing its radius.";
    MISSING_FILL = "S006", "A filled shape is missing its fill paint.";
    NON_PIXEL_SHAPE_DIMENSION = "S007", "A shape dimension or point does not use pixels.";
    MISSING_POINTS = "S008", "A polygon, polyline, or brush is missing its points.";
    TOO_FEW_POINTS = "S009", "A polygon has fewer than three points.";
    MISSING_STROKE = "S010", "A stroked shape is missing its stroke declaration.";
    INVALID_STROKE_WIDTH = "S011", "A stroke width is not positive.";
    TOO_FEW_POLYLINE_POINTS = "S012", "A polyline has fewer than two points.";
    MISSING_PATH_START = "S013", "A path has no commands or does not start with `start`.";
    INVALID_TRANSFORM = "S014", "A transform value (rotation, scale, or opacity) is invalid.";
    DUPLICATE_LAYER_NAME = "S016", "Two layer definitions share the same name.";
    INVALID_LAYER_SIZE = "S017", "A layer size is zero or negative.";
    INVALID_LAYER_NAME = "S018", "A layer name is empty.";
    INVALID_Z_INDEX = "S020", "A z-index is not an integer.";
    EMPTY_RESOURCE_PATH = "S022", "A picture resource path is empty.";
    EMPTY_TEXT = "S023", "A text primitive has empty content.";
    DUPLICATE_VARIABLE_NAME = "S024", "A variable is declared twice in the same block.";
    UNKNOWN_VARIABLE = "S025", "An expression references an unknown variable.";
    INVALID_EXPRESSION = "S026", "An expression cannot be evaluated.";
    INVALID_REPEAT_COUNT = "S027", "A repeat count is not a non-negative integer.";
    INVALID_SEED = "S028", "A document seed is negative or invalid.";
    ROOT_LAYER_RESERVED = "S029", "An explicit layer or group uses the reserved implicit root name `root`.";
    DUPLICATE_GROUP_NAME = "S030", "Two group definitions share the same name.";
    INVALID_GROUP_NAME = "S031", "A group name is empty.";
    INVALID_GROUP_SIZE = "S032", "A group size is zero or negative.";
    UNKNOWN_GROUP = "S033", "A placement references a group that is not defined.";
    PLACE_TARGETS_LAYER = "S034", "A placement references a layer; placements target group definitions only.";
    RECURSIVE_GROUP_REFERENCE = "S035", "Group definitions reference each other recursively.";
    DUPLICATE_LAYER_FILL = "S036", "A layer declares its background `fill` more than once.";

    // Resource errors (Rxxx)
    MISSING_RESOURCE = "R001", "A referenced resource file could not be found.";
    UNDECODABLE_RESOURCE = "R002", "A referenced resource could not be decoded.";
    UNRESOLVED_FONT = "R003", "A requested text font could not be resolved.";

    // Paint errors (Txxx)
    INVALID_HEX_COLOR_FORMAT = "T001", "A fill or stroke hex color is malformed.";
    UNKNOWN_NAMED_COLOR = "T002", "A color name is not in the named color palette.";
    }

    /// Alias for [`MISSING_RESOURCE`].
    pub const RESOURCE_NOT_FOUND: &str = MISSING_RESOURCE;
    /// Alias for [`UNDECODABLE_RESOURCE`].
    pub const INVALID_RESOURCE: &str = UNDECODABLE_RESOURCE;
    /// Alias for [`UNDECODABLE_RESOURCE`].
    pub const RESOURCE_DECODE_ERROR: &str = UNDECODABLE_RESOURCE;
    /// Alias for [`UNRESOLVED_FONT`].
    pub const MISSING_FONT: &str = UNRESOLVED_FONT;
}

/// Common alternative spellings LLMs and humans use for Rupa keywords.
///
/// These are intentionally semantic (not just typos): for example `colour`
/// suggests `fill` because that is the Rupa property that accepts a color.
const KEYWORD_ALIASES: &[(&str, &str)] = &[
    ("color", "fill"),
    ("colour", "fill"),
    ("fill-color", "fill"),
    ("background-color", "background"),
    ("bg", "background"),
    ("rect", "rectangle"),
    ("circle-shape", "circle"),
    ("oval", "ellipse"),
    ("centre", "center"),
    ("rad", "radius"),
    ("rotation", "rotate"),
    ("rotation-angle", "rotate"),
    ("scaling", "scale"),
    ("alpha", "opacity"),
    ("zindex", "z-index"),
    ("z_index", "z-index"),
    ("zindex-order", "z-index"),
    ("txt", "text"),
    ("text-content", "content"),
    ("img", "picture"),
    ("image", "picture"),
    ("fontsize", "size"),
    ("font-size", "size"),
    ("fontsize-px", "size"),
    ("line-width", "width"),
    ("stroke-width", "width"),
    ("coords", "points"),
    ("coordinates", "points"),
    ("loop", "repeat"),
    ("var", "let"),
    ("variable", "let"),
    ("func", "fn"),
    ("function", "fn"),
    ("return-value", "return"),
    ("random-seed", "seed"),
    ("path-string", "path"),
    ("transparency", "opacity"),
    ("blend-mode", "blend"),
];

/// Suggest a keyword or property for an unknown identifier.
///
/// This first checks a curated alias table (so semantically related names such
/// as `colour`/`color` map to `fill`) and then falls back to the Levenshtein
/// [`did_you_mean`] search over [`KNOWN_KEYWORDS`].
pub fn suggest_keyword(input: &str) -> Option<&'static str> {
    let lower = input.to_ascii_lowercase();
    if let Some((_, canonical)) = KEYWORD_ALIASES.iter().find(|(alias, _)| *alias == lower) {
        return Some(canonical);
    }
    did_you_mean(&lower)
}

/// Collection of diagnostics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DiagnosticBag {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticBag {
    /// Create a new diagnostic bag
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of collected diagnostics.
    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    /// Whether no diagnostics were collected.
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    /// Borrow the diagnostics in emission order.
    pub fn as_slice(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Iterate over the diagnostics in emission order.
    pub fn iter(&self) -> std::slice::Iter<'_, Diagnostic> {
        self.diagnostics.iter()
    }

    /// Add a diagnostic
    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Add an error diagnostic.
    pub fn error(&mut self, code: &str, message: impl Into<String>, span: Span) {
        self.push(Diagnostic::error(code, message, span));
    }

    /// Add an error diagnostic with help text.
    pub fn error_with_help(
        &mut self,
        code: &str,
        message: impl Into<String>,
        span: Span,
        help: impl Into<String>,
    ) {
        self.push(Diagnostic::error(code, message, span).with_help(help));
    }

    /// Append all diagnostics from another bag.
    pub fn append(&mut self, other: DiagnosticBag) {
        self.diagnostics.extend(other.diagnostics);
    }

    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
    }

    /// Get all errors
    pub fn errors(&self) -> Vec<&Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .collect()
    }

    /// Get all warnings
    pub fn warnings(&self) -> Vec<&Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .collect()
    }

    /// Sort diagnostics by source position for stable output.
    pub fn sort(&mut self) {
        self.diagnostics.sort_by_key(|d| {
            (
                d.primary_span.file_id().index(),
                d.primary_span.start.offset(),
            )
        });
    }
}

impl IntoIterator for DiagnosticBag {
    type Item = Diagnostic;
    type IntoIter = std::vec::IntoIter<Diagnostic>;

    fn into_iter(self) -> Self::IntoIter {
        self.diagnostics.into_iter()
    }
}

impl<'a> IntoIterator for &'a DiagnosticBag {
    type Item = &'a Diagnostic;
    type IntoIter = std::slice::Iter<'a, Diagnostic>;

    fn into_iter(self) -> Self::IntoIter {
        self.diagnostics.iter()
    }
}

impl FromIterator<Diagnostic> for DiagnosticBag {
    fn from_iter<T: IntoIterator<Item = Diagnostic>>(iter: T) -> Self {
        Self {
            diagnostics: iter.into_iter().collect(),
        }
    }
}

impl Extend<Diagnostic> for DiagnosticBag {
    fn extend<T: IntoIterator<Item = Diagnostic>>(&mut self, iter: T) {
        self.diagnostics.extend(iter);
    }
}

/// Render one diagnostic with source context:
///
/// ```text
/// error[P002]: missing required property `size`
///   --> example.rupa:3:5
///    |
///  3 |     colour red
///    |     ^^^^^^ unknown keyword or property `colour` in rectangle
///    |
///    = help: did you mean `fill`?
/// ```
///
/// Spans covering more than one line show up to three source lines: the first
/// line from the start column, each middle line in full, and the final line up
/// to the end column. The diagnostic message labels the first caret run.
pub fn render_diagnostic(path: &str, source: &str, diag: &Diagnostic) -> String {
    let sev = match diag.severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
        Severity::Help => "help",
    };
    let index = LineIndex::new(source);
    let start = index.line_col(diag.primary_span.start_position());
    let end = index.line_col(diag.primary_span.end_position());
    let mut out = String::new();
    // Rendering and slicing use usize; compact offsets are widened losslessly
    // on every supported target.
    let line_no = start.line as usize;
    let col_no = start.column as usize;
    out.push_str(&format!("{sev}[{}]: {}\n", diag.code, diag.message));
    out.push_str(&format!("  --> {path}:{line_no}:{col_no}\n"));

    let lines: Vec<&str> = source.lines().collect();
    let end_line = (end.line as usize).max(line_no);
    let last_line = end_line.min(line_no.saturating_add(2));
    let gutter = last_line.to_string().len();
    let pad = " ".repeat(gutter);
    out.push_str(&format!("{pad} |\n"));

    if line_no == 0 || line_no > lines.len() {
        out.push_str(&format!("{pad} | (source line unavailable)\n"));
    } else {
        let shown_last = last_line.min(lines.len());
        // The message labels the span once, on its first line. Keep the snippet
        // valid even if a producer ever puts a newline in the message.
        let label = diag.message.lines().next().unwrap_or_default().trim();

        for current in line_no..=shown_last {
            let line = lines[current - 1];
            out.push_str(&format!("{current:>gutter$} | {line}\n"));

            let line_chars = line.chars().count();
            let first = current == line_no;
            let final_line = current == end_line;
            let start_col = if first { col_no } else { 1 };
            let end_col = if final_line {
                end.column as usize
            } else {
                line_chars.saturating_add(1)
            };

            let caret_start = start_col.saturating_sub(1).min(line_chars);
            let max_width = line_chars
                .saturating_add(1)
                .saturating_sub(caret_start)
                .max(1);
            let width = end_col.saturating_sub(start_col).max(1).min(max_width);
            let suffix = if first && !label.is_empty() {
                format!(" {label}")
            } else {
                String::new()
            };
            out.push_str(&format!(
                "{pad} | {}{}{suffix}\n",
                " ".repeat(caret_start),
                "^".repeat(width)
            ));
        }
    }

    out.push_str(&format!("{pad} |\n"));
    if let Some(help) = &diag.help {
        out.push_str(&format!("{pad} = help: {help}\n"));
    }
    out
}

/// Render all diagnostics in a bag, each with source context.
pub fn render_diagnostics(path: &str, source: &str, bag: &DiagnosticBag) -> String {
    let mut out = String::new();
    for diag in bag {
        out.push_str(&render_diagnostic(path, source, diag));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::{FileId, Location, Position};

    fn span(_start_line: u32, _start_col: u32, start_off: u32, end_off: u32) -> Span {
        Span::new(FileId(0), Position::new(start_off), Position::new(end_off))
    }

    #[test]
    fn test_render_diagnostic_format() {
        let source = "size 100px x 100px\n    colour red\n";
        // Span over "colour" on line 2 (line 2 starts at offset 19)
        let diag = Diagnostic {
            code: "L006".to_string(),
            severity: Severity::Error,
            message: "unknown keyword or property `colour` in document".to_string(),
            primary_span: span(2, 5, 23, 29),
            help: Some("did you mean `fill`?".to_string()),
            edits: Vec::new(),
        };
        let rendered = render_diagnostic("example.rupa", source, &diag);
        assert!(
            rendered.starts_with("error[L006]: unknown keyword or property `colour` in document\n")
        );
        assert!(rendered.contains("--> example.rupa:2:5\n"));
        assert!(rendered.contains("2 |     colour red\n"));
        // 6 carets under "colour", labelled with the diagnostic message.
        assert!(
            rendered.contains("|     ^^^^^^ unknown keyword or property `colour` in document\n")
        );
        assert!(rendered.contains("= help: did you mean `fill`?\n"));
    }

    #[test]
    fn test_render_warning_severity() {
        let source = "size 100px x 100px\n";
        let diag = Diagnostic {
            code: "C003".to_string(),
            severity: Severity::Warning,
            message: "invalid dimensions".to_string(),
            primary_span: span(1, 1, 0, 1),
            help: None,
            edits: Vec::new(),
        };
        let rendered = render_diagnostic("t.rupa", source, &diag);
        assert!(rendered.starts_with("warning[C003]: invalid dimensions\n"));
        assert!(!rendered.contains("= help:"));
    }

    #[test]
    fn test_render_multiline_span_shows_each_line_and_labels_once() {
        let source = "size 100px x 100px\nrectangle at (0px, 0px)\n    size 20px x 20px\n";
        let mut s = span(2, 1, 19, 21);
        s.end = Location::new(FileId(0), Position::new(59));
        let diag = Diagnostic::error("P003", "span crosses lines", s);
        let rendered = render_diagnostic("t.rupa", source, &diag);

        assert!(rendered.contains("2 | rectangle at (0px, 0px)\n"));
        assert!(rendered.contains("3 |     size 20px x 20px\n"));
        let first_width = "rectangle at (0px, 0px)".chars().count();
        assert!(rendered.contains(&format!(
            "| {} span crosses lines\n",
            "^".repeat(first_width)
        )));
        assert!(rendered.contains(&format!("| {}\n", "^".repeat(16))));
        // Header plus one span label, not a label per line.
        assert_eq!(rendered.matches("span crosses lines").count(), 2);
    }

    #[test]
    fn test_render_multiline_span_caps_context_at_three_lines() {
        let source = "one\ntwo\nthree\nfour\nfive\n";
        let mut s = span(1, 1, 0, 3);
        s.end = Location::new(FileId(0), Position::new(24));
        let diag = Diagnostic::error("P001", "long span", s);
        let rendered = render_diagnostic("t.rupa", source, &diag);

        assert!(rendered.contains("1 | one\n"));
        assert!(rendered.contains("2 | two\n"));
        assert!(rendered.contains("3 | three\n"));
        assert!(!rendered.contains("4 | four\n"));
    }

    #[test]
    fn test_render_line_out_of_range() {
        let diag = Diagnostic::error("P001", "boom", span(99, 1, 99, 100));
        let rendered = render_diagnostic("t.rupa", "one\n", &diag);
        assert!(rendered.contains("(source line unavailable)"));
    }

    #[test]
    fn test_multiple_diagnostics_rendered_in_order() {
        let source = "    foo\nbar\n";
        let mut bag = DiagnosticBag::new();
        bag.push(Diagnostic::error(
            "L006",
            "unknown `bar`",
            span(2, 1, 8, 11),
        ));
        bag.push(Diagnostic::error("L006", "unknown `foo`", span(1, 5, 4, 7)));
        // Sorting by source position gives stable, source-ordered output.
        bag.sort();
        let rendered = render_diagnostics("t.rupa", source, &bag);
        let foo_pos = rendered.find("t.rupa:1:5").unwrap();
        let bar_pos = rendered.find("t.rupa:2:1").unwrap();
        assert!(foo_pos < bar_pos);
        // Sorting by position keeps the same order.
        bag.sort();
        assert_eq!(bag.as_slice()[0].message, "unknown `foo`");
    }

    #[test]
    fn test_suggestion_for_misspelled_fill() {
        // Note: the task doc's example `colour` -> `fill` is not within any
        // reasonable edit-distance threshold (distance 5); realistic
        // misspellings such as `fil` are suggested instead.
        let diag = Diagnostic::error(
            "L006",
            "unknown keyword or property `fil`",
            span(1, 1, 0, 3),
        )
        .with_suggestion("fil");
        assert_eq!(diag.help.as_deref(), Some("did you mean `fill`?"));
    }

    #[test]
    fn test_suggestion_appends_to_existing_help() {
        let diag = Diagnostic::error("L006", "unknown", span(1, 1, 0, 6))
            .with_help("expected a keyword")
            .with_suggestion("fil");
        assert_eq!(
            diag.help.as_deref(),
            Some("expected a keyword; did you mean `fill`?")
        );
    }

    #[test]
    fn test_suggestion_records_machine_applicable_edit() {
        let target = span(2, 5, 23, 26);
        let diag = Diagnostic::error("L006", "unknown", target).with_suggestion("fil");
        assert_eq!(
            diag.edits,
            vec![SuggestedEdit {
                span: target,
                replacement: "fill".to_string(),
            }]
        );
    }

    #[test]
    fn test_no_edit_without_a_candidate() {
        let diag =
            Diagnostic::error("L006", "unknown", span(1, 1, 0, 6)).with_suggestion("zzzzzzzzzzzz");
        assert!(diag.edits.is_empty());
        assert!(diag.help.is_none());
    }

    #[test]
    fn test_no_suggestion_for_garbage() {
        assert_eq!(did_you_mean("zzzzzzzzzzzz"), None);
        assert_eq!(did_you_mean(""), None);
    }

    #[test]
    fn test_did_you_mean_from_custom_candidates() {
        let candidates = ["size", "fill", "radius"];
        assert_eq!(did_you_mean_from("fils", &candidates), Some("fill"));
        assert_eq!(did_you_mean_from("radios", &candidates), Some("radius"));
        assert_eq!(did_you_mean_from("completely-unrelated", &candidates), None);
    }

    #[test]
    fn test_did_you_mean_ties_are_deterministic() {
        // Both candidates at equal distance: alphabetical tie-break must be stable.
        let candidates = ["bat", "cat"];
        assert_eq!(did_you_mean_from("aat", &candidates), Some("bat"));
    }

    #[test]
    fn test_diagnostic_codes_are_stable() {
        // Codes are documented constants; verify a representative set.
        assert_eq!(codes::INVALID_INDENTATION, "L001");
        assert_eq!(codes::TAB_CHARACTER, "L002");
        assert_eq!(codes::UNKNOWN_KEYWORD, "L006");
        assert_eq!(codes::EXPECTED_TOKEN, "P001");
        assert_eq!(codes::INVALID_DIMENSIONS, "C003");
        assert_eq!(codes::NON_PIXEL_DIMENSION, "C007");
        assert_eq!(codes::CANVAS_TOO_LARGE, "C008");
        assert_eq!(codes::NEGATIVE_DIMENSION, "S001");
        assert_eq!(codes::NON_PIXEL_SHAPE_DIMENSION, "S007");
        assert_eq!(codes::MISSING_POINTS, "S008");
        assert_eq!(codes::TOO_FEW_POINTS, "S009");
        assert_eq!(codes::MISSING_STROKE, "S010");
        assert_eq!(codes::INVALID_STROKE_WIDTH, "S011");
        assert_eq!(codes::TOO_FEW_POLYLINE_POINTS, "S012");
        assert_eq!(codes::MISSING_PATH_START, "S013");
        assert_eq!(codes::INVALID_TRANSFORM, "S014");
        assert_eq!(codes::DUPLICATE_LAYER_NAME, "S016");
        assert_eq!(codes::INVALID_LAYER_SIZE, "S017");
        assert_eq!(codes::INVALID_LAYER_NAME, "S018");
        assert_eq!(codes::INVALID_Z_INDEX, "S020");
        assert_eq!(codes::DUPLICATE_GROUP_NAME, "S030");
        assert_eq!(codes::INVALID_GROUP_NAME, "S031");
        assert_eq!(codes::INVALID_GROUP_SIZE, "S032");
        assert_eq!(codes::UNKNOWN_GROUP, "S033");
        assert_eq!(codes::PLACE_TARGETS_LAYER, "S034");
        assert_eq!(codes::RECURSIVE_GROUP_REFERENCE, "S035");
        assert_eq!(codes::UNKNOWN_NAMED_COLOR, "T002");
    }

    #[test]
    fn test_code_catalog_is_complete_and_unique() {
        assert!(!codes::ALL.is_empty());
        let mut seen = std::collections::HashSet::new();
        for info in codes::ALL {
            assert!(
                seen.insert(info.code),
                "duplicate code in catalog: {} ({})",
                info.code,
                info.name
            );
            assert!(!info.name.is_empty());
            assert!(!info.description.is_empty());
            assert!(
                info.code.len() == 4
                    && matches!(
                        info.code.as_bytes()[0],
                        b'L' | b'P' | b'C' | b'S' | b'R' | b'T'
                    )
                    && info.code[1..].bytes().all(|b| b.is_ascii_digit()),
                "malformed diagnostic code: {}",
                info.code
            );
            let found = codes::describe(info.code).expect("catalog lookup");
            assert_eq!(found, info);
            assert_eq!(codes::describe(info.code).map(|i| i.name), Some(info.name));
        }
        // Every constant exposed by the module must be represented exactly once.
        let constants = [
            codes::INVALID_INDENTATION,
            codes::TAB_CHARACTER,
            codes::UNTERMINATED_STRING,
            codes::INVALID_NUMBER,
            codes::INVALID_HEX_COLOR,
            codes::UNKNOWN_KEYWORD,
            codes::INVALID_CHARACTER,
            codes::INVALID_ESCAPE,
            codes::UNKNOWN_UNIT,
            codes::SOURCE_TOO_LARGE,
            codes::EXPECTED_TOKEN,
            codes::MISSING_REQUIRED_PROPERTY,
            codes::INVALID_PROPERTY_VALUE,
            codes::INVALID_DIMENSIONS,
            codes::NON_PIXEL_DIMENSION,
            codes::CANVAS_TOO_LARGE,
            codes::DUPLICATE_DOCUMENT_SIZE,
            codes::DUPLICATE_BACKGROUND,
            codes::DUPLICATE_SEED,
            codes::NEGATIVE_DIMENSION,
            codes::NEGATIVE_RADIUS,
            codes::MISSING_POSITION,
            codes::MISSING_SIZE,
            codes::MISSING_RADIUS,
            codes::MISSING_FILL,
            codes::NON_PIXEL_SHAPE_DIMENSION,
            codes::MISSING_POINTS,
            codes::TOO_FEW_POINTS,
            codes::MISSING_STROKE,
            codes::INVALID_STROKE_WIDTH,
            codes::TOO_FEW_POLYLINE_POINTS,
            codes::MISSING_PATH_START,
            codes::INVALID_TRANSFORM,
            codes::DUPLICATE_LAYER_NAME,
            codes::INVALID_LAYER_SIZE,
            codes::INVALID_LAYER_NAME,
            codes::INVALID_Z_INDEX,
            codes::EMPTY_RESOURCE_PATH,
            codes::EMPTY_TEXT,
            codes::DUPLICATE_VARIABLE_NAME,
            codes::UNKNOWN_VARIABLE,
            codes::INVALID_EXPRESSION,
            codes::INVALID_REPEAT_COUNT,
            codes::INVALID_SEED,
            codes::ROOT_LAYER_RESERVED,
            codes::DUPLICATE_GROUP_NAME,
            codes::INVALID_GROUP_NAME,
            codes::INVALID_GROUP_SIZE,
            codes::DUPLICATE_LAYER_FILL,
            codes::UNKNOWN_GROUP,
            codes::PLACE_TARGETS_LAYER,
            codes::RECURSIVE_GROUP_REFERENCE,
            codes::MISSING_RESOURCE,
            codes::UNDECODABLE_RESOURCE,
            codes::UNRESOLVED_FONT,
            codes::INVALID_HEX_COLOR_FORMAT,
            codes::UNKNOWN_NAMED_COLOR,
        ];
        assert_eq!(constants.len(), codes::ALL.len());
        for code in constants {
            assert!(
                codes::describe(code).is_some(),
                "missing from catalog: {code}"
            );
        }
    }

    #[test]
    fn test_code_aliases_match_canonical_codes() {
        assert_eq!(codes::RESOURCE_NOT_FOUND, codes::MISSING_RESOURCE);
        assert_eq!(codes::INVALID_RESOURCE, codes::UNDECODABLE_RESOURCE);
        assert_eq!(codes::RESOURCE_DECODE_ERROR, codes::UNDECODABLE_RESOURCE);
        assert_eq!(codes::MISSING_FONT, codes::UNRESOLVED_FONT);
    }

    #[test]
    fn test_suggest_keyword_aliases() {
        assert_eq!(suggest_keyword("colour"), Some("fill"));
        assert_eq!(suggest_keyword("color"), Some("fill"));
        assert_eq!(suggest_keyword("Colour"), Some("fill"));
        assert_eq!(suggest_keyword("rect"), Some("rectangle"));
        assert_eq!(suggest_keyword("bg"), Some("background"));
        assert_eq!(suggest_keyword("centre"), Some("center"));
        assert_eq!(suggest_keyword("z_index"), Some("z-index"));
        assert_eq!(suggest_keyword("img"), Some("picture"));
        // Typo fallback still works through Levenshtein search.
        assert_eq!(suggest_keyword("fil"), Some("fill"));
        assert_eq!(suggest_keyword("zzzzzzzzzzzz"), None);
    }

    #[test]
    fn test_error_and_warning_constructors() {
        let d = Diagnostic::warning("C003", "invalid dimensions", span(1, 1, 0, 1));
        assert_eq!(d.severity, Severity::Warning);
        assert_eq!(d.code, "C003");
        let d = Diagnostic::error("P001", "boom", span(1, 1, 0, 1)).with_help("fix it");
        assert_eq!(d.severity, Severity::Error);
        assert_eq!(d.help.as_deref(), Some("fix it"));
    }

    #[test]
    fn test_bag_convenience_constructors_and_append() {
        let mut first = DiagnosticBag::new();
        first.error("P001", "plain", span(1, 1, 0, 1));
        first.error_with_help("P002", "with help", span(1, 1, 0, 1), "fix it");
        assert_eq!(first.len(), 2);
        assert_eq!(first.as_slice()[0].help, None);
        assert_eq!(first.as_slice()[1].help.as_deref(), Some("fix it"));

        let mut second = DiagnosticBag::new();
        second.error("P003", "second", span(1, 1, 0, 1));
        first.append(second);
        assert_eq!(first.len(), 3);
        assert_eq!(first.as_slice()[2].code, "P003");
    }

    #[test]
    fn bag_supports_iteration_and_collection() {
        let mut bag = DiagnosticBag::new();
        bag.error("P001", "one", span(1, 1, 0, 1));
        bag.error("P002", "two", span(1, 2, 0, 2));

        let codes: Vec<&str> = bag
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect();
        assert_eq!(codes, vec!["P001", "P002"]);

        let borrowed: DiagnosticBag = (&bag).into_iter().cloned().collect();
        assert_eq!(borrowed.len(), 2);

        let mut extended = DiagnosticBag::new();
        extended.extend(borrowed);
        assert_eq!(extended.len(), 2);

        let owned: Vec<Diagnostic> = bag.into_iter().collect();
        assert_eq!(owned.len(), 2);
    }

    #[test]
    fn test_bag_error_warning_filters() {
        let mut bag = DiagnosticBag::new();
        bag.push(Diagnostic::error("P001", "e", span(1, 1, 0, 1)));
        bag.push(Diagnostic::warning("C003", "w", span(1, 1, 0, 1)));
        assert!(bag.has_errors());
        assert_eq!(bag.errors().len(), 1);
        assert_eq!(bag.warnings().len(), 1);
    }
}
