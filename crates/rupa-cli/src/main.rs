//! Rupa CLI: command-line interface for the Rupa compiler.

mod benchmark;
mod commands;
mod diagnostics;

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::ExitCode;

/// Rupa CLI
#[derive(Parser, Debug)]
#[command(name = "rupa", version, about = "Rupa graphics compiler", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// CLI commands
#[derive(Subcommand, Debug)]
enum Commands {
    /// Check a Rupa file for syntax and semantic errors
    Check {
        /// Input .rupa file
        input: PathBuf,
        /// Print diagnostics as JSON (codes, positions, help, and edits)
        #[arg(long)]
        json: bool,
    },
    /// Render a Rupa file to PNG
    Render {
        /// Input .rupa file
        input: PathBuf,
        /// Output PNG file (default: input filename with .png in the current directory)
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,
    },
    /// Format a Rupa file canonically
    Fmt {
        /// Input .rupa file (omit with --stdin)
        #[arg(required_unless_present = "stdin", conflicts_with = "stdin")]
        input: Option<PathBuf>,
        /// Check whether the source is already canonical; write nothing
        #[arg(long)]
        check: bool,
        /// Read source from stdin and write formatted source to stdout
        #[arg(long)]
        stdin: bool,
    },
    /// Inspect a Rupa file and print its compiled structure as JSON
    Inspect {
        /// Input .rupa file
        input: PathBuf,
    },
    /// Print canonical language metadata as JSON
    Metadata {
        /// Write JSON to this file instead of stdout
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,
        /// Merge the metadata keys into an existing language-reference JSON file
        #[arg(long, value_name = "FILE", conflicts_with = "output")]
        reference: Option<PathBuf>,
    },
    /// Evaluate candidate sources against the LLM benchmark suite
    Bench {
        /// Directory containing the benchmark manifest and fixtures
        #[arg(long, default_value = "benchmarks")]
        tasks: PathBuf,
        /// Directory containing candidate sources (one file or attempt per task)
        #[arg(long)]
        candidates: PathBuf,
        /// Optional JSON report path (printed to stdout when omitted)
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Check { input, json } => commands::check(&input, json),
        Commands::Render { input, output } => commands::render(&input, output.as_deref()),
        Commands::Fmt {
            input,
            check,
            stdin,
        } => commands::fmt(input.as_deref(), check, stdin),
        Commands::Inspect { input } => commands::inspect(&input),
        Commands::Metadata { output, reference } => {
            commands::metadata(output.as_deref(), reference.as_deref())
        }
        Commands::Bench {
            tasks,
            candidates,
            output,
        } => commands::bench(&tasks, &candidates, output.as_deref()),
    };

    match result {
        Ok(code) => code,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
