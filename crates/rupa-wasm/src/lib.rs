//! Browser (WebAssembly) bindings for the Rupa compiler and renderer.
//!
//! The module keeps the browser contract small and explicit:
//!
//! - [`render_source`] compiles and rasterizes one source string and returns a
//!   [`Frame`] (metadata JSON plus PNG bytes). It is plain safe Rust, so the
//!   same code is exercised by native tests.
//! - [`format_source_text`] returns the same canonical text `rupa fmt` writes,
//!   or the diagnostics that prevented formatting.
//! - [`metadata_json`] returns the canonical keyword, colour, unit, and
//!   primitive tables the playground uses for syntax highlighting.
//! - [`encode_frame`], [`encode_format_frame`], and [`encode_metadata_frame`]
//!   serialize those results into the single-buffer wire format the JavaScript
//!   host reads.
//! - [`abi`] exposes the C ABI used by the host to pass memory in and out of
//!   the module (see that module's safety documentation).
//!
//! Rendering is deterministic: the same source produces byte-identical PNG
//! output for the same renderer build, matching `rupa render`.

#![warn(missing_docs)]

pub mod abi;

use rupa_compiler::compile_source;
use rupa_image::encode::{encode_rgba_png_to_writer, unpremultiply_rgba};
use rupa_renderer::{RenderError, RenderOutput, render_document};
use rupa_syntax::diagnostic::{Diagnostic, Severity, codes};
use rupa_syntax::span::{FileId, LineIndex};
use serde_json::{Value, json};

/// Number of bytes the frame header occupies: total length and metadata length.
pub const FRAME_HEADER_BYTES: usize = 8;

/// One rendered frame: metadata plus PNG bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    /// JSON metadata: status, canvas size, diagnostics, and render errors.
    pub meta: String,
    /// PNG bytes (empty when rendering failed).
    pub png: Vec<u8>,
}

impl Frame {
    /// Build a failure frame.
    fn failure(diagnostics: Vec<Value>, render_error: Option<String>) -> Self {
        Self {
            meta: json!({
                "status": "error",
                "width": Value::Null,
                "height": Value::Null,
                "diagnostics": diagnostics,
                "renderError": render_error,
            })
            .to_string(),
            png: Vec::new(),
        }
    }

    /// Whether this frame rendered successfully.
    pub fn is_ok(&self) -> bool {
        self.meta.contains("\"status\":\"ok\"")
    }
}

/// One formatted document: metadata plus canonical source text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatFrame {
    /// JSON metadata: status and diagnostics.
    pub meta: String,
    /// Canonical UTF-8 source (empty when formatting failed).
    pub text: String,
}

/// One language-metadata payload: frame metadata plus the metadata JSON.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataFrame {
    /// Frame status metadata.
    pub meta: String,
    /// Canonical language metadata JSON used by the playground highlighter.
    pub json: String,
}

/// Return the canonical language metadata as pretty-printed JSON.
///
/// This is the same value as `rupa metadata`, assembled by
/// `rupa-compiler` from the lexer keyword table, the compiler's named colours,
/// and the language reference tables.
pub fn metadata_json() -> String {
    serde_json::to_string_pretty(&rupa_compiler::metadata_json())
        .expect("language metadata is always JSON-serializable")
}

/// Package language metadata into the frame wire format used by the host.
pub fn metadata_frame() -> MetadataFrame {
    MetadataFrame {
        meta: json!({
            "status": "ok",
            "error": Value::Null,
        })
        .to_string(),
        json: metadata_json(),
    }
}

/// Format one source string exactly as `rupa fmt` does.
///
/// On success the frame carries the canonical text and `status: "ok"`. When the
/// source cannot be formatted because of lexical errors, the frame carries
/// those diagnostics, empty text, and `status: "error"`; the caller keeps its
/// current buffer.
///
/// Formatting needs no compile step, so this is cheaper than rendering and works
/// on sources that do not compile yet but are syntactically well formed.
pub fn format_source_text(source: &str) -> FormatFrame {
    match rupa_syntax::format_source(source) {
        Ok(text) => FormatFrame {
            meta: json!({
                "status": "ok",
                "diagnostics": [],
                "error": Value::Null,
            })
            .to_string(),
            text,
        },
        Err(mut bag) => {
            bag.sort();
            let index = LineIndex::new(source);
            FormatFrame {
                meta: json!({
                    "status": "error",
                    "diagnostics": bag.iter().map(|diagnostic| diagnostic_json(diagnostic, &index)).collect::<Vec<_>>(),
                    "error": Value::Null,
                })
                .to_string(),
                text: String::new(),
            }
        }
    }
}

/// Compile and render one Rupa source string.
///
/// Compilation diagnostics are always reported. Rendering failures that have a
/// stable diagnostic code (`R001`–`R003`) are reported as diagnostics;
/// everything else is reported in the `renderError` metadata field.
pub fn render_source(source: &str) -> Frame {
    let mut compiled = compile_source(source, FileId(0));
    compiled.diagnostics.sort();
    let index = LineIndex::new(source);
    let mut diagnostics: Vec<Value> = compiled
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic_json(diagnostic, &index))
        .collect();

    if compiled.has_errors() {
        return Frame::failure(diagnostics, None);
    }

    let Some(graphics) = compiled.graphics else {
        return Frame::failure(
            diagnostics,
            Some("internal error: compilation produced no graphics IR".to_string()),
        );
    };

    match render_document(&graphics) {
        Ok(output) => match encode_png(&output) {
            Ok(png) => Frame {
                meta: json!({
                    "status": "ok",
                    "width": output.width,
                    "height": output.height,
                    "diagnostics": diagnostics,
                    "renderError": Value::Null,
                })
                .to_string(),
                png,
            },
            Err(detail) => Frame::failure(diagnostics, Some(detail)),
        },
        Err(error) => match render_diagnostic(&error) {
            Some(diagnostic) => {
                diagnostics.push(diagnostic);
                Frame::failure(diagnostics, None)
            }
            None => Frame::failure(diagnostics, Some(error.to_string())),
        },
    }
}

/// Serialize a frame into the wire format read by the JavaScript host.
///
/// Layout (little endian):
///
/// ```text
/// [u32 total length][u32 metadata length][metadata UTF-8][body bytes]
/// ```
///
/// The total length counts every byte of the frame, including the header.
pub fn encode_frame(frame: &Frame) -> Vec<u8> {
    encode_payload(&frame.meta, &frame.png)
}

/// Serialize a format result into the wire format read by the JavaScript host.
///
/// The header is identical to [`encode_frame`]; the body is the formatted UTF-8
/// text instead of PNG bytes, so the host reads both results the same way and
/// releases both with `rupa_free_frame`.
pub fn encode_format_frame(frame: &FormatFrame) -> Vec<u8> {
    encode_payload(&frame.meta, frame.text.as_bytes())
}

/// Serialize language metadata into the same wire format as [`encode_frame`].
///
/// The body is the metadata JSON; the host reads it exactly like formatted
/// text and releases the buffer with `rupa_free_frame`.
pub fn encode_metadata_frame(frame: &MetadataFrame) -> Vec<u8> {
    encode_payload(&frame.meta, frame.json.as_bytes())
}

/// Build one frame from its metadata and body bytes.
fn encode_payload(meta: &str, body: &[u8]) -> Vec<u8> {
    let meta = meta.as_bytes();
    let total = FRAME_HEADER_BYTES + meta.len() + body.len();
    let mut bytes = Vec::with_capacity(total);
    bytes.extend_from_slice(&(total as u32).to_le_bytes());
    bytes.extend_from_slice(&(meta.len() as u32).to_le_bytes());
    bytes.extend_from_slice(meta);
    bytes.extend_from_slice(body);
    bytes
}

/// Encode renderer output as PNG bytes, unpremultiplying first.
fn encode_png(output: &RenderOutput) -> Result<Vec<u8>, String> {
    let straight = unpremultiply_rgba(&output.pixels);
    let mut png = Vec::new();
    encode_rgba_png_to_writer(&straight, output.width, output.height, &mut png)
        .map_err(|error| format!("PNG encoding failed: {error}"))?;
    Ok(png)
}

/// Convert a compile diagnostic into the browser metadata shape.
fn diagnostic_json(diagnostic: &Diagnostic, index: &LineIndex<'_>) -> Value {
    let start = index.line_col(diagnostic.primary_span.start_position());
    let end = index.line_col(diagnostic.primary_span.end_position());
    json!({
        "code": diagnostic.code,
        "severity": severity_name(diagnostic.severity),
        "message": diagnostic.message,
        "line": start.line,
        "column": start.column,
        "endLine": end.line,
        "endColumn": end.column,
        "help": diagnostic.help,
    })
}

/// Map a render failure onto a stable diagnostic code when one applies.
///
/// Resource failures are the three cases the language already names; other
/// renderer failures have no language-level code and are reported through the
/// `renderError` metadata field instead of inventing one.
fn render_diagnostic(error: &RenderError) -> Option<Value> {
    let (code, message) = match error {
        RenderError::MissingResource(path) => (
            codes::MISSING_RESOURCE,
            format!("picture `{path}` cannot be loaded here; the playground has no file access"),
        ),
        RenderError::Resource { path, detail } => (
            codes::UNDECODABLE_RESOURCE,
            format!("cannot load picture resource `{path}`: {detail}"),
        ),
        RenderError::MissingFont(family) => (
            codes::UNRESOLVED_FONT,
            format!(
                "cannot resolve font `{family}`; install a font byte array before rendering text"
            ),
        ),
        _ => return None,
    };

    Some(json!({
        "code": code,
        "severity": "error",
        "message": message,
        "line": 1,
        "column": 1,
        "endLine": 1,
        "endColumn": 1,
        "help": Value::Null,
    }))
}

/// Lowercase severity name used in metadata.
fn severity_name(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
        Severity::Help => "help",
    }
}
