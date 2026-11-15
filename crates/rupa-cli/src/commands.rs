//! Implementations of CLI commands.

use crate::diagnostics::print_diagnostics;
use rupa_compiler::{CompiledSource, compile_source};
use rupa_image::ResourceStore;
use rupa_renderer::{
    RenderError, encode_render_output, render_document_with_resources, resolve_font_bytes,
};
use rupa_syntax::ast::FontWeight;
use rupa_syntax::diagnostic::{Diagnostic, DiagnosticBag, Severity, codes};
use rupa_syntax::span::{FileId, LineIndex};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// CLI command failure.
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    /// The input file could not be read.
    #[error("failed to read input file '{}': {source}", path.display())]
    ReadInput {
        /// Input path.
        path: PathBuf,
        /// Underlying IO error.
        #[source]
        source: std::io::Error,
    },

    /// `--output` does not name a `.png` file.
    #[error(
        "output path '{}' must end in '.png'; Rupa renders PNG images (use --output out.png)",
        path.display()
    )]
    NonPngOutput {
        /// Requested output path.
        path: PathBuf,
    },
    /// `fmt` could not write the formatted file.
    #[error("failed to write formatted file '{}': {source}", path.display())]
    WriteOutput {
        /// Output path.
        path: PathBuf,
        /// Underlying IO error.
        #[source]
        source: std::io::Error,
    },
    /// `metadata --output` could not write its JSON file.
    #[error("failed to write metadata file '{}': {source}", path.display())]
    WriteMetadata {
        /// Output path.
        path: PathBuf,
        /// Underlying IO error.
        #[source]
        source: std::io::Error,
    },
    /// `metadata --reference` could not read its reference file.
    #[error("failed to read reference file '{}': {source}", path.display())]
    ReadReference {
        /// Reference path.
        path: PathBuf,
        /// Underlying IO error.
        #[source]
        source: std::io::Error,
    },
    /// `metadata --reference` could not parse its reference file.
    #[error("failed to parse reference file '{}': {source}", path.display())]
    ParseReference {
        /// Reference path.
        path: PathBuf,
        /// Underlying JSON error.
        #[source]
        source: serde_json::Error,
    },
    /// `metadata --reference` read a JSON value that is not an object.
    #[error("reference file '{}' must contain a JSON object", path.display())]
    ReferenceNotObject {
        /// Reference path.
        path: PathBuf,
    },
    /// `fmt --stdin` could not read standard input.
    #[error("failed to read standard input: {source}")]
    ReadStdin {
        /// Underlying IO error.
        #[source]
        source: std::io::Error,
    },
    /// `inspect` or `bench` could not serialize its output.
    #[error("failed to serialize output: {source}")]
    Serialize {
        /// Underlying error.
        #[source]
        source: serde_json::Error,
    },
    /// A benchmark report could not be written.
    #[error("failed to write benchmark report '{}': {source}", path.display())]
    WriteReport {
        /// Report path.
        path: PathBuf,
        /// Underlying IO error.
        #[source]
        source: std::io::Error,
    },
    /// The benchmark suite could not be evaluated.
    #[error(transparent)]
    Benchmark(#[from] crate::benchmark::BenchmarkError),
    /// Compilation succeeded without producing graphics IR.
    #[error("internal error: compilation produced no graphics IR")]
    MissingGraphics,
    /// Rendering failed.
    #[error(transparent)]
    Render(#[from] RenderError),
    /// PNG encoding failed.
    #[error(transparent)]
    Png(#[from] rupa_renderer::PngError),
}

/// Run `rupa check`: parse and validate without rendering.
///
/// With `json`, diagnostics are printed to stdout as JSON (`code`, `severity`,
/// `message`, positions, `help`, and machine-applicable `edits`) instead of the
/// human-readable rendering on stderr.
pub fn check(input: &Path, json: bool) -> Result<ExitCode, CliError> {
    let (source, mut compiled) = compile_input(input)?;

    // Resource diagnostics: when compilation succeeds, verify that referenced
    // resources (pictures and fonts) can be resolved relative to the input.
    if !compiled.has_errors()
        && let Some(document) = &compiled.document
    {
        let resource_diagnostics = check_resources(document, input_base_dir(input));
        compiled.diagnostics.append(resource_diagnostics);
    }

    if json {
        print_diagnostics_json(input, &compiled, &source)?;
    } else {
        report(&compiled, input, &source);
    }

    if compiled.has_errors() {
        Ok(ExitCode::FAILURE)
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

/// Print `rupa check --json`: one object with the path, status, and diagnostics.
///
/// Each diagnostic carries its stable code, severity, message, 1-based
/// positions, optional help, and any machine-applicable `edits` (span plus
/// replacement text) so a tool can repair the source without parsing prose.
fn print_diagnostics_json(
    input: &Path,
    compiled: &CompiledSource,
    source: &str,
) -> Result<(), CliError> {
    let index = LineIndex::new(source);
    let diagnostics: Vec<serde_json::Value> = compiled
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic_json(diagnostic, &index))
        .collect();
    let status = if compiled.diagnostics.has_errors() {
        "error"
    } else {
        "ok"
    };
    let payload = serde_json::json!({
        "path": input.display().to_string(),
        "status": status,
        "diagnostics": diagnostics,
    });
    let json =
        serde_json::to_string_pretty(&payload).map_err(|source| CliError::Serialize { source })?;
    println!("{json}");
    Ok(())
}

/// Convert one diagnostic into the JSON shape used by `check --json`.
fn diagnostic_json(diagnostic: &Diagnostic, index: &LineIndex<'_>) -> serde_json::Value {
    let start = index.line_col(diagnostic.primary_span.start_position());
    let end = index.line_col(diagnostic.primary_span.end_position());
    let edits: Vec<serde_json::Value> = diagnostic
        .edits
        .iter()
        .map(|edit| {
            let edit_start = index.line_col(edit.span.start_position());
            let edit_end = index.line_col(edit.span.end_position());
            serde_json::json!({
                "startLine": edit_start.line,
                "startColumn": edit_start.column,
                "endLine": edit_end.line,
                "endColumn": edit_end.column,
                "startOffset": edit.span.start.offset(),
                "endOffset": edit.span.end.offset(),
                "replacement": edit.replacement,
            })
        })
        .collect();

    serde_json::json!({
        "code": diagnostic.code,
        "severity": severity_name(diagnostic.severity),
        "message": diagnostic.message,
        "line": start.line,
        "column": start.column,
        "endLine": end.line,
        "endColumn": end.column,
        "help": diagnostic.help,
        "edits": edits,
    })
}

/// Lowercase severity name used in JSON output.
fn severity_name(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
        Severity::Help => "help",
    }
}

/// Run `rupa render`: compile, render to a pixel buffer, and encode PNG.
///
/// The output path comes from `--output`. Without it, the PNG is written to the
/// input filename with a `.png` extension, in the current working directory
/// (specification §21). An explicit path must name a `.png` file, so PNG bytes
/// are never written under a misleading extension.
pub fn render(input: &Path, output: Option<&Path>) -> Result<ExitCode, CliError> {
    let default_output = default_output_path(input);
    let output = output.unwrap_or(&default_output);
    if !is_png_path(output) {
        return Err(CliError::NonPngOutput {
            path: output.to_path_buf(),
        });
    }

    let (source, compiled) = compile_input(input)?;
    report(&compiled, input, &source);

    if compiled.has_errors() {
        return Ok(ExitCode::FAILURE);
    }

    let graphics = compiled.graphics.ok_or(CliError::MissingGraphics)?;
    let mut resources = ResourceStore::new(input_base_dir(input));
    let rendered = render_document_with_resources(&graphics, &mut resources)?;
    encode_render_output(&rendered, output)?;
    Ok(ExitCode::SUCCESS)
}

/// Walk a list of document children, collecting every picture and text node
/// into the given vectors. Repeat-loop bodies are traversed recursively so
/// resources referenced inside loops are also validated (specification §22).
fn collect_resource_children<'a>(
    children: &'a [rupa_syntax::ast::DocumentChild],
    pictures: &mut Vec<&'a rupa_syntax::ast::Picture>,
    texts: &mut Vec<&'a rupa_syntax::ast::Text>,
) {
    for child in children {
        match child {
            rupa_syntax::ast::DocumentChild::Picture(picture) => pictures.push(picture),
            rupa_syntax::ast::DocumentChild::Text(text) => texts.push(text),
            rupa_syntax::ast::DocumentChild::Repeat(repeat) => {
                collect_resource_children(&repeat.body, pictures, texts);
            }
            _ => {}
        }
    }
}

/// Collect resource diagnostics for a compiled document.
fn check_resources(document: &rupa_syntax::ast::Document, base_dir: &Path) -> DiagnosticBag {
    let mut bag = DiagnosticBag::new();

    let mut pictures: Vec<&rupa_syntax::ast::Picture> = Vec::new();
    let mut texts: Vec<&rupa_syntax::ast::Text> = Vec::new();

    collect_resource_children(&document.children, &mut pictures, &mut texts);
    for layer in &document.layers {
        collect_resource_children(&layer.children, &mut pictures, &mut texts);
    }
    for group in &document.groups {
        collect_resource_children(&group.children, &mut pictures, &mut texts);
    }

    let mut resources = ResourceStore::new(base_dir);
    for picture in pictures {
        if picture.path.trim().is_empty() {
            continue;
        }
        if let Err(error) = resources.load(&picture.path) {
            bag.push(
                Diagnostic::error(
                    codes::MISSING_RESOURCE,
                    format!("cannot load picture resource `{}`: {error}", picture.path),
                    picture.span,
                )
                .with_help("ensure the image exists and is a valid PNG"),
            );
        }
    }

    for text in texts {
        if let Some(font) = &text.font
            && !font.trim().is_empty()
        {
            let bold = matches!(text.weight, Some(FontWeight::Bold));
            if let Err(error) = resolve_font_bytes(Some(font), bold) {
                bag.push(
                    Diagnostic::error(
                        codes::UNRESOLVED_FONT,
                        format!("cannot resolve font `{font}`: {error}"),
                        text.span,
                    )
                    .with_help("install the font or omit 'font' to use the default"),
                );
            }
        }
    }

    bag
}

/// Run `rupa fmt`: format a Rupa file canonically.
///
/// Without flags the file is rewritten in place. With `check`, nothing is
/// written and the exit status is 1 when the source is not canonical. With
/// `stdin`, source is read from standard input and canonical text is written to
/// standard output; `check` then suppresses the output and reports whether the
/// input already matched.
pub fn fmt(input: Option<&Path>, check: bool, stdin: bool) -> Result<ExitCode, CliError> {
    let (source, display_path) = if stdin {
        let mut source = String::new();
        std::io::stdin()
            .read_to_string(&mut source)
            .map_err(|source| CliError::ReadStdin { source })?;
        (source, PathBuf::from("<stdin>"))
    } else {
        let input = input.expect("clap requires a path unless --stdin is set");
        (read_source(input)?, input.to_path_buf())
    };

    match rupa_syntax::format_source(&source) {
        Ok(formatted) => {
            if check {
                if formatted == source {
                    return Ok(ExitCode::SUCCESS);
                }
                if stdin {
                    eprintln!("<stdin> is not formatted");
                } else {
                    eprintln!(
                        "{} is not formatted; run `rupa fmt {}` to rewrite it",
                        display_path.display(),
                        display_path.display()
                    );
                }
                return Ok(ExitCode::FAILURE);
            }
            if stdin {
                print!("{formatted}");
                return Ok(ExitCode::SUCCESS);
            }
            std::fs::write(input.expect("path required"), formatted).map_err(|source| {
                CliError::WriteOutput {
                    path: display_path,
                    source,
                }
            })?;
            Ok(ExitCode::SUCCESS)
        }
        Err(bag) => {
            print_diagnostics(&display_path, &source, &bag);
            Ok(ExitCode::FAILURE)
        }
    }
}

/// Run `rupa inspect`: print a JSON representation of the compiled document.
pub fn inspect(input: &Path) -> Result<ExitCode, CliError> {
    let (source, compiled) = compile_input(input)?;
    report(&compiled, input, &source);

    if compiled.has_errors() {
        return Ok(ExitCode::FAILURE);
    }

    let graphics = compiled.graphics.ok_or(CliError::MissingGraphics)?;
    let json =
        serde_json::to_string_pretty(&graphics).map_err(|source| CliError::Serialize { source })?;
    println!("{json}");
    Ok(ExitCode::SUCCESS)
}

/// Run `rupa metadata`: print the canonical language metadata as JSON.
///
/// The same JSON is available to the playground through `rupa_metadata()` in
/// `rupa-wasm`, so the editor highlighter and this output cannot drift apart.
/// With `reference`, the canonical keys are merged into an existing reference
/// object and the result is written back, preserving hand-authored prose fields.
pub fn metadata(output: Option<&Path>, reference: Option<&Path>) -> Result<ExitCode, CliError> {
    let metadata = rupa_compiler::metadata_json();

    if let Some(path) = reference {
        let source = std::fs::read_to_string(path).map_err(|source| CliError::ReadReference {
            path: path.to_path_buf(),
            source,
        })?;
        let mut document: serde_json::Value =
            serde_json::from_str(&source).map_err(|source| CliError::ParseReference {
                path: path.to_path_buf(),
                source,
            })?;
        let Some(document) = document.as_object_mut() else {
            return Err(CliError::ReferenceNotObject {
                path: path.to_path_buf(),
            });
        };
        if let Some(metadata) = metadata.as_object() {
            for (key, value) in metadata {
                document.insert(key.clone(), value.clone());
            }
        }
        let json = serde_json::to_string_pretty(&document)
            .map_err(|source| CliError::Serialize { source })?;
        std::fs::write(path, format!("{json}\n")).map_err(|source| CliError::WriteMetadata {
            path: path.to_path_buf(),
            source,
        })?;
        return Ok(ExitCode::SUCCESS);
    }

    let json =
        serde_json::to_string_pretty(&metadata).map_err(|source| CliError::Serialize { source })?;
    match output {
        Some(path) => {
            std::fs::write(path, format!("{json}\n")).map_err(|source| {
                CliError::WriteMetadata {
                    path: path.to_path_buf(),
                    source,
                }
            })?;
        }
        None => println!("{json}"),
    }
    Ok(ExitCode::SUCCESS)
}

/// Run `rupa bench`: evaluate candidate sources against a benchmark suite.
///
/// The report is printed as JSON to stdout, or written to `output` when given.
/// A benchmark run returns success even when tasks fail; task failures are
/// metrics, not harness errors.
pub fn bench(tasks: &Path, candidates: &Path, output: Option<&Path>) -> Result<ExitCode, CliError> {
    let report = crate::benchmark::run_suite(tasks, candidates)?;
    let json =
        serde_json::to_string_pretty(&report).map_err(|source| CliError::Serialize { source })?;

    if let Some(output) = output {
        std::fs::write(output, format!("{json}\n")).map_err(|source| CliError::WriteReport {
            path: output.to_path_buf(),
            source,
        })?;
    } else {
        println!("{json}");
    }

    Ok(ExitCode::SUCCESS)
}

/// Whether an output path names a PNG file (case-insensitive extension).
fn is_png_path(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("png"))
}

/// Default render output: the input filename with a `.png` extension, resolved
/// against the current working directory.
fn default_output_path(input: &Path) -> PathBuf {
    let stem = input
        .file_stem()
        .filter(|stem| !stem.is_empty())
        .unwrap_or_else(|| std::ffi::OsStr::new("output"));
    PathBuf::from(stem).with_extension("png")
}

/// Read and compile `input` without printing anything.
fn compile_input(input: &Path) -> Result<(String, CompiledSource), CliError> {
    let source = read_source(input)?;
    let compiled = compile_source(&source, FileId(0));
    Ok((source, compiled))
}

/// Directory relative resources resolve against.
fn input_base_dir(input: &Path) -> &Path {
    input.parent().unwrap_or_else(|| Path::new("."))
}

fn read_source(input: &Path) -> Result<String, CliError> {
    std::fs::read_to_string(input).map_err(|source| CliError::ReadInput {
        path: input.to_path_buf(),
        source,
    })
}

fn report(compiled: &CompiledSource, input: &Path, source: &str) {
    print_diagnostics(input, source, &compiled.diagnostics);
}
