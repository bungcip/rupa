//! Offline evaluation harness for LLM-generated Rupa source.
//!
//! A benchmark suite is a JSON manifest of tasks, each with a prompt and a
//! canonical solution. Candidate sources are supplied as files or attempt
//! directories. The harness compiles and renders candidates, compares them with
//! the canonical solution render, and reports deterministic metrics
//! (specification §23.5).
//!
//! The harness never calls a model itself. This keeps benchmark runs offline,
//! reproducible, and independent of provider APIs.

use rupa_compiler::compile_source;
use rupa_image::ResourceStore;
use rupa_renderer::{RenderOutput, render_document_with_resources};
use rupa_syntax::Lexer;
use rupa_syntax::diagnostic::Severity;
use rupa_syntax::span::FileId;
use rupa_syntax::token::TokenKind;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Benchmark harness error.
#[derive(Debug, thiserror::Error)]
pub enum BenchmarkError {
    /// The task manifest could not be read.
    #[error("failed to read benchmark manifest '{}': {source}", path.display())]
    ReadManifest {
        /// Manifest path.
        path: PathBuf,
        /// Underlying IO error.
        #[source]
        source: std::io::Error,
    },
    /// The task manifest is not valid JSON or does not match the schema.
    #[error("failed to parse benchmark manifest '{}': {source}", path.display())]
    ParseManifest {
        /// Manifest path.
        path: PathBuf,
        /// Underlying JSON error.
        #[source]
        source: serde_json::Error,
    },
    /// A source file could not be read.
    #[error("failed to read benchmark source '{}': {source}", path.display())]
    ReadSource {
        /// Source path.
        path: PathBuf,
        /// Underlying IO error.
        #[source]
        source: std::io::Error,
    },
    /// The candidates directory could not be read.
    #[error("failed to read candidates directory '{}': {source}", path.display())]
    ReadCandidates {
        /// Candidates directory.
        path: PathBuf,
        /// Underlying IO error.
        #[source]
        source: std::io::Error,
    },
    /// A task or its canonical solution is invalid.
    #[error("invalid benchmark task '{id}': {message}")]
    InvalidTask {
        /// Task identifier.
        id: String,
        /// Explanation of the problem.
        message: String,
    },
}

/// Kind of benchmark task.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskKind {
    /// Create a document from a prompt.
    #[default]
    Create,
    /// Modify an existing source file.
    Modify,
    /// Repair intentionally invalid source.
    Repair,
    /// Reproduce a reference composition.
    Reference,
}

/// One task as declared in the benchmark manifest.
#[derive(Debug, Clone, Deserialize)]
pub struct BenchmarkTask {
    /// Stable task identifier, also the candidate directory/file name.
    pub id: String,
    /// Task category.
    #[serde(default)]
    pub kind: TaskKind,
    /// Natural-language prompt given to the model.
    pub prompt: String,
    /// Canonical solution, relative to the manifest directory.
    pub solution: PathBuf,
    /// Optional starting source for modify/repair tasks.
    #[serde(default)]
    pub starter: Option<PathBuf>,
    /// Diagnostic codes that the starter is expected to emit.
    #[serde(default)]
    pub expected_diagnostics: Vec<String>,
}

/// Benchmark suite manifest.
#[derive(Debug, Clone, Deserialize)]
pub struct BenchmarkSuite {
    /// Manifest schema version.
    pub version: u32,
    /// Tasks in the suite.
    pub tasks: Vec<BenchmarkTask>,
}

/// Metrics for one candidate attempt.
#[derive(Debug, Clone, Serialize)]
pub struct AttemptReport {
    /// Candidate source file.
    pub source: String,
    /// Whether the source compiled and lowered successfully.
    pub compiles: bool,
    /// Whether the source rendered successfully.
    pub render_success: bool,
    /// Error diagnostics with lexer/parser codes (`L*`/`P*`).
    pub syntax_errors: usize,
    /// Error diagnostics from later phases.
    pub semantic_errors: usize,
    /// Sorted unique error diagnostic codes.
    pub diagnostic_codes: Vec<String>,
    /// Normalized visual similarity to the reference render, when rendered.
    pub visual_similarity: Option<f64>,
    /// Fraction of pixels exactly equal to the reference, when rendered.
    pub pixel_match_ratio: Option<f64>,
    /// Lexical token count (excluding layout tokens).
    pub token_count: u64,
    /// Source length in bytes.
    pub source_length_bytes: u64,
    /// Source line count.
    pub source_lines: u64,
}

/// Metrics for one benchmark task.
#[derive(Debug, Clone, Serialize)]
pub struct TaskReport {
    /// Stable task identifier.
    pub id: String,
    /// Task category.
    pub kind: TaskKind,
    /// Prompt presented to the model.
    pub prompt: String,
    /// Whether any candidate attempt was found.
    pub attempted: bool,
    /// Number of candidate attempts.
    pub attempt_count: usize,
    /// Retries after the first attempt.
    pub repair_iterations: u32,
    /// First attempt compiled successfully.
    pub first_attempt_compiles: Option<bool>,
    /// Final attempt compiled successfully.
    pub final_compiles: Option<bool>,
    /// Final attempt rendered successfully.
    pub render_success: Option<bool>,
    /// Final visual similarity to the canonical solution render.
    pub visual_similarity: Option<f64>,
    /// Final exact pixel-match ratio.
    pub pixel_match_ratio: Option<f64>,
    /// Final lexical token count.
    pub token_count: Option<u64>,
    /// Final source length in bytes.
    pub source_length_bytes: Option<u64>,
    /// Final source line count.
    pub source_lines: Option<u64>,
    /// Per-attempt metrics.
    pub attempts: Vec<AttemptReport>,
}

/// Aggregate suite report.
#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkReport {
    /// Manifest schema version.
    pub suite_version: u32,
    /// Number of tasks in the suite.
    pub task_count: usize,
    /// Number of tasks with at least one candidate attempt.
    pub attempted_count: usize,
    /// Total number of candidate attempts evaluated.
    pub total_attempts: usize,
    /// Fraction of tasks whose first attempt compiled.
    pub first_attempt_compilation_success_rate: f64,
    /// Fraction of tasks whose final attempt compiled.
    pub compilation_success_rate: f64,
    /// Fraction of tasks whose final attempt rendered.
    pub rendering_success_rate: f64,
    /// Fraction of attempts containing at least one syntax error.
    pub syntax_error_rate: f64,
    /// Fraction of attempts containing at least one semantic error.
    pub semantic_error_rate: f64,
    /// Mean repair iterations over attempted tasks.
    pub average_repair_iterations: f64,
    /// Mean final visual similarity; failed renders count as 0.
    pub average_visual_similarity: f64,
    /// Mean final token count over attempted tasks.
    pub average_token_count: f64,
    /// Mean final source length over attempted tasks.
    pub average_source_length_bytes: f64,
    /// Per-task reports.
    pub tasks: Vec<TaskReport>,
}

/// Load a suite manifest from a benchmarks directory.
pub fn load_suite(tasks_dir: &Path) -> Result<BenchmarkSuite, BenchmarkError> {
    let path = tasks_dir.join("tasks.json");
    let source = std::fs::read_to_string(&path).map_err(|source| BenchmarkError::ReadManifest {
        path: path.clone(),
        source,
    })?;
    serde_json::from_str(&source).map_err(|source| BenchmarkError::ParseManifest { path, source })
}

/// Evaluate all tasks in `tasks_dir` against candidates in `candidates_dir`.
pub fn run_suite(
    tasks_dir: &Path,
    candidates_dir: &Path,
) -> Result<BenchmarkReport, BenchmarkError> {
    let suite = load_suite(tasks_dir)?;
    if !candidates_dir.is_dir() {
        return Err(BenchmarkError::ReadCandidates {
            path: candidates_dir.to_path_buf(),
            source: std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "candidates path is not a directory",
            ),
        });
    }

    let mut tasks = Vec::with_capacity(suite.tasks.len());
    for task in &suite.tasks {
        tasks.push(evaluate_task(tasks_dir, candidates_dir, task)?);
    }

    Ok(aggregate(suite.version, tasks))
}

fn evaluate_task(
    tasks_dir: &Path,
    candidates_dir: &Path,
    task: &BenchmarkTask,
) -> Result<TaskReport, BenchmarkError> {
    if task.id.trim().is_empty() {
        return Err(BenchmarkError::InvalidTask {
            id: task.id.clone(),
            message: "task id must not be empty".to_string(),
        });
    }

    let attempts = discover_attempts(candidates_dir, &task.id)?;
    if attempts.is_empty() {
        return Ok(TaskReport {
            id: task.id.clone(),
            kind: task.kind,
            prompt: task.prompt.clone(),
            attempted: false,
            attempt_count: 0,
            repair_iterations: 0,
            first_attempt_compiles: None,
            final_compiles: None,
            render_success: None,
            visual_similarity: None,
            pixel_match_ratio: None,
            token_count: None,
            source_length_bytes: None,
            source_lines: None,
            attempts: Vec::new(),
        });
    }

    let solution_path = tasks_dir.join(&task.solution);
    let solution_source = read_source(&solution_path)?;
    let reference = render_source(&solution_source, &solution_path).map_err(|message| {
        BenchmarkError::InvalidTask {
            id: task.id.clone(),
            message: format!("canonical solution does not render: {message}"),
        }
    })?;

    if let Some(starter) = &task.starter {
        let starter_path = tasks_dir.join(starter);
        let starter_source = read_source(&starter_path)?;
        let compiled = compile_source(starter_source, FileId(0));
        let codes: BTreeSet<&str> = compiled
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == Severity::Error)
            .map(|diagnostic| diagnostic.code.as_str())
            .collect();
        for expected in &task.expected_diagnostics {
            if !codes.contains(expected.as_str()) {
                return Err(BenchmarkError::InvalidTask {
                    id: task.id.clone(),
                    message: format!(
                        "starter '{}' no longer emits expected diagnostic {expected}",
                        starter_path.display()
                    ),
                });
            }
        }
    }

    let reports: Vec<AttemptReport> = attempts
        .iter()
        .map(|path| evaluate_attempt(path, &reference))
        .collect::<Result<_, _>>()?;

    let first = reports.first();
    let final_attempt = reports.last();
    Ok(TaskReport {
        id: task.id.clone(),
        kind: task.kind,
        prompt: task.prompt.clone(),
        attempted: !reports.is_empty(),
        attempt_count: reports.len(),
        repair_iterations: reports.len().saturating_sub(1) as u32,
        first_attempt_compiles: first.map(|attempt| attempt.compiles),
        final_compiles: final_attempt.map(|attempt| attempt.compiles),
        render_success: final_attempt.map(|attempt| attempt.render_success),
        visual_similarity: final_attempt.and_then(|attempt| attempt.visual_similarity),
        pixel_match_ratio: final_attempt.and_then(|attempt| attempt.pixel_match_ratio),
        token_count: final_attempt.map(|attempt| attempt.token_count),
        source_length_bytes: final_attempt.map(|attempt| attempt.source_length_bytes),
        source_lines: final_attempt.map(|attempt| attempt.source_lines),
        attempts: reports,
    })
}

fn discover_attempts(candidates_dir: &Path, task_id: &str) -> Result<Vec<PathBuf>, BenchmarkError> {
    let directory = candidates_dir.join(task_id);
    if directory.is_dir() {
        let mut attempts: Vec<PathBuf> = std::fs::read_dir(&directory)
            .map_err(|source| BenchmarkError::ReadCandidates {
                path: directory.clone(),
                source,
            })?
            .map(|entry| entry.expect("candidate directory entry").path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "rupa"))
            .collect();
        attempts.sort();
        return Ok(attempts);
    }

    let single = candidates_dir.join(format!("{task_id}.rupa"));
    if single.is_file() {
        Ok(vec![single])
    } else {
        Ok(Vec::new())
    }
}

fn evaluate_attempt(
    path: &Path,
    reference: &RenderOutput,
) -> Result<AttemptReport, BenchmarkError> {
    let source = read_source(path)?;
    let token_count = count_tokens(&source);
    let compiled = compile_source(source.clone(), FileId(0));

    let mut syntax_errors = 0;
    let mut semantic_errors = 0;
    let mut codes = BTreeSet::new();
    for diagnostic in compiled
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Error)
    {
        codes.insert(diagnostic.code.clone());
        if diagnostic.code.starts_with('L') || diagnostic.code.starts_with('P') {
            syntax_errors += 1;
        } else {
            semantic_errors += 1;
        }
    }

    let compiles = !compiled.has_errors() && compiled.graphics.is_some();
    let (render_success, visual_similarity, pixel_match_ratio) =
        if let Some(graphics) = compiled.graphics.as_ref() {
            let base_dir = path.parent().unwrap_or_else(|| Path::new("."));
            let mut resources = ResourceStore::new(base_dir);
            match render_document_with_resources(graphics, &mut resources) {
                Ok(output) => {
                    let (similarity, match_ratio) = compare_render(reference, &output);
                    (true, Some(similarity), Some(match_ratio))
                }
                Err(_) => (false, None, None),
            }
        } else {
            (false, None, None)
        };

    Ok(AttemptReport {
        source: path.display().to_string(),
        compiles,
        render_success,
        syntax_errors,
        semantic_errors,
        diagnostic_codes: codes.into_iter().collect(),
        visual_similarity,
        pixel_match_ratio,
        token_count,
        source_length_bytes: source.len() as u64,
        source_lines: source.lines().count() as u64,
    })
}

fn read_source(path: &Path) -> Result<String, BenchmarkError> {
    std::fs::read_to_string(path).map_err(|source| BenchmarkError::ReadSource {
        path: path.to_path_buf(),
        source,
    })
}

fn render_source(source: &str, path: &Path) -> Result<RenderOutput, String> {
    let compiled = compile_source(source.to_string(), FileId(0));
    if compiled.has_errors() {
        let codes: Vec<&str> = compiled
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == Severity::Error)
            .map(|diagnostic| diagnostic.code.as_str())
            .collect();
        return Err(format!("compilation failed with: {}", codes.join(", ")));
    }
    let graphics = compiled
        .graphics
        .as_ref()
        .ok_or_else(|| "compilation produced no graphics".to_string())?;
    let base_dir = path.parent().unwrap_or_else(|| Path::new("."));
    let mut resources = ResourceStore::new(base_dir);
    render_document_with_resources(graphics, &mut resources).map_err(|error| error.to_string())
}

fn count_tokens(source: &str) -> u64 {
    let (tokens, _) = Lexer::new(source.to_string(), FileId(0)).tokenize();
    tokens
        .iter()
        .filter(|token| {
            !matches!(
                token.kind,
                TokenKind::Newline | TokenKind::Indent | TokenKind::Dedent | TokenKind::Eof
            )
        })
        .count() as u64
}

fn compare_render(reference: &RenderOutput, actual: &RenderOutput) -> (f64, f64) {
    if reference.width != actual.width || reference.height != actual.height {
        return (0.0, 0.0);
    }
    let pixel_count = u64::from(reference.width) * u64::from(reference.height);
    if pixel_count == 0
        || reference.pixels.len() != actual.pixels.len()
        || reference.pixels.len() != pixel_count as usize * 4
    {
        return (0.0, 0.0);
    }

    let mut matching_pixels = 0u64;
    let mut difference_sum = 0u64;
    for (reference_pixel, actual_pixel) in reference
        .pixels
        .as_chunks::<4>()
        .0
        .iter()
        .zip(actual.pixels.as_chunks::<4>().0.iter())
    {
        if reference_pixel == actual_pixel {
            matching_pixels += 1;
        }
        difference_sum += reference_pixel
            .iter()
            .zip(actual_pixel)
            .map(|(a, b)| u64::from(a.abs_diff(*b)))
            .sum::<u64>();
    }

    let max_difference = pixel_count as f64 * 4.0 * 255.0;
    let similarity = (1.0 - difference_sum as f64 / max_difference).clamp(0.0, 1.0);
    (similarity, matching_pixels as f64 / pixel_count as f64)
}

fn aggregate(version: u32, tasks: Vec<TaskReport>) -> BenchmarkReport {
    let attempted: Vec<&TaskReport> = tasks.iter().filter(|task| task.attempted).collect();
    let attempts: Vec<&AttemptReport> = attempted.iter().flat_map(|task| &task.attempts).collect();

    let task_count = tasks.len();
    let attempted_count = attempted.len();
    let total_attempts = attempts.len();
    let first_attempt_successes = attempted
        .iter()
        .filter(|task| task.first_attempt_compiles == Some(true))
        .count();
    let final_successes = attempted
        .iter()
        .filter(|task| task.final_compiles == Some(true))
        .count();
    let render_successes = attempted
        .iter()
        .filter(|task| task.render_success == Some(true))
        .count();
    let syntax_error_attempts = attempts
        .iter()
        .filter(|attempt| attempt.syntax_errors > 0)
        .count();
    let semantic_error_attempts = attempts
        .iter()
        .filter(|attempt| attempt.semantic_errors > 0)
        .count();

    BenchmarkReport {
        suite_version: version,
        task_count,
        attempted_count,
        total_attempts,
        first_attempt_compilation_success_rate: ratio(first_attempt_successes, attempted_count),
        compilation_success_rate: ratio(final_successes, attempted_count),
        rendering_success_rate: ratio(render_successes, attempted_count),
        syntax_error_rate: ratio(syntax_error_attempts, total_attempts),
        semantic_error_rate: ratio(semantic_error_attempts, total_attempts),
        average_repair_iterations: mean(
            attempted
                .iter()
                .map(|task| f64::from(task.repair_iterations)),
        ),
        average_visual_similarity: mean(
            attempted
                .iter()
                .map(|task| task.visual_similarity.unwrap_or(0.0)),
        ),
        average_token_count: mean(
            attempted
                .iter()
                .map(|task| task.token_count.unwrap_or(0) as f64),
        ),
        average_source_length_bytes: mean(
            attempted
                .iter()
                .map(|task| task.source_length_bytes.unwrap_or(0) as f64),
        ),
        tasks,
    }
}

fn ratio(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

fn mean(values: impl Iterator<Item = f64>) -> f64 {
    let mut sum = 0.0;
    let mut count = 0usize;
    for value in values {
        sum += value;
        count += 1;
    }
    if count == 0 { 0.0 } else { sum / count as f64 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_count_excludes_layout_tokens() {
        let source = "size 10px x 10px\n";
        // size, number, unit, x, number, unit
        assert_eq!(count_tokens(source), 6);
    }

    #[test]
    fn compare_identical_render_is_perfect() {
        let output = RenderOutput {
            width: 2,
            height: 1,
            pixels: vec![255, 0, 0, 255, 0, 0, 255, 255],
        };
        let (similarity, matching) = compare_render(&output, &output);
        assert_eq!(similarity, 1.0);
        assert_eq!(matching, 1.0);
    }

    #[test]
    fn compare_different_dimensions_is_zero() {
        let a = RenderOutput {
            width: 1,
            height: 1,
            pixels: vec![0, 0, 0, 255],
        };
        let b = RenderOutput {
            width: 2,
            height: 1,
            pixels: vec![0, 0, 0, 255, 0, 0, 0, 255],
        };
        assert_eq!(compare_render(&a, &b), (0.0, 0.0));
    }

    #[test]
    fn compare_measures_partial_difference() {
        let reference = RenderOutput {
            width: 2,
            height: 1,
            pixels: vec![0, 0, 0, 255, 0, 0, 0, 255],
        };
        let actual = RenderOutput {
            width: 2,
            height: 1,
            pixels: vec![0, 0, 0, 255, 255, 255, 255, 255],
        };
        let (similarity, matching) = compare_render(&reference, &actual);
        assert_eq!(matching, 0.5);
        assert!(similarity > 0.0 && similarity < 1.0);
    }

    #[test]
    fn aggregate_without_attempts_is_zeroed() {
        let report = aggregate(
            1,
            vec![TaskReport {
                id: "idle".to_string(),
                kind: TaskKind::Create,
                prompt: String::new(),
                attempted: false,
                attempt_count: 0,
                repair_iterations: 0,
                first_attempt_compiles: None,
                final_compiles: None,
                render_success: None,
                visual_similarity: None,
                pixel_match_ratio: None,
                token_count: None,
                source_length_bytes: None,
                source_lines: None,
                attempts: Vec::new(),
            }],
        );
        assert_eq!(report.task_count, 1);
        assert_eq!(report.attempted_count, 0);
        assert_eq!(report.compilation_success_rate, 0.0);
        assert_eq!(report.average_visual_similarity, 0.0);
    }
}
