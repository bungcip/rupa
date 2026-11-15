//! Executable documentation tests.
//!
//! Every example under `examples/` must compile and render, every example under
//! `examples/invalid/` must fail with the diagnostic codes listed in
//! `expected-diagnostics.json`, and the structured language reference example
//! must compile. This keeps documentation and fixtures from drifting away from
//! the implementation (specification §22.2).

mod common;

use common::{decode_png, run_rupa, temp_dir, workspace_root};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

fn rupa_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("read directory {}: {error}", dir.display()))
        .map(|entry| entry.expect("directory entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "rupa"))
        .collect();
    files.sort();
    files
}

fn assert_success(args: &[&str], context: &str) {
    let output = run_rupa(args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{context} failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "{context} produced diagnostics:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn valid_examples_compile_and_render() {
    let examples_dir = workspace_root().join("examples");
    let files = rupa_files(&examples_dir);
    assert!(!files.is_empty(), "no examples found in {examples_dir:?}");

    let output_dir = temp_dir();
    for file in &files {
        let path = file.to_str().expect("utf-8 example path");
        assert_success(&["check", path], &format!("check {path}"));

        let stem = file
            .file_stem()
            .and_then(|stem| stem.to_str())
            .expect("utf-8 example stem");
        let rendered = output_dir.join(format!("{stem}.png"));
        let rendered_path = rendered.to_str().expect("utf-8 output path");
        assert_success(
            &["render", path, "--output", rendered_path],
            &format!("render {path}"),
        );
        let image = decode_png(&rendered);
        assert!(image.width > 0 && image.height > 0);
    }

    let _ = std::fs::remove_dir_all(output_dir);
}

#[test]
fn invalid_examples_fail_with_documented_diagnostic_codes() {
    let invalid_dir = workspace_root().join("examples/invalid");
    let manifest_path = invalid_dir.join("expected-diagnostics.json");
    let manifest_source =
        std::fs::read_to_string(&manifest_path).expect("read expected diagnostics manifest");
    let expectations: BTreeMap<String, Vec<String>> =
        serde_json::from_str(&manifest_source).expect("parse expected diagnostics manifest");

    let files = rupa_files(&invalid_dir);
    let actual: BTreeSet<String> = files
        .iter()
        .map(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .expect("utf-8 invalid example name")
                .to_string()
        })
        .collect();
    let documented: BTreeSet<String> = expectations.keys().cloned().collect();
    assert_eq!(
        actual, documented,
        "every invalid example must be listed in expected-diagnostics.json"
    );

    for file in &files {
        let name = file
            .file_name()
            .and_then(|name| name.to_str())
            .expect("utf-8 invalid example name");
        let path = file.to_str().expect("utf-8 invalid example path");
        let output = run_rupa(&["check", path]);
        assert_ne!(
            output.status.code(),
            Some(0),
            "invalid example {path} unexpectedly passed"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        for code in &expectations[name] {
            assert!(
                stderr.contains(&format!("error[{code}]")),
                "invalid example {path} did not report {code}:\n{stderr}"
            );
        }
    }
}

#[test]
fn structured_language_reference_example_compiles() {
    let reference_path = workspace_root().join("docs/language-reference.json");
    let source = std::fs::read_to_string(&reference_path).expect("read language reference");
    let reference: serde_json::Value =
        serde_json::from_str(&source).expect("parse language reference");
    let example = reference["example"]
        .as_str()
        .expect("language reference has an example");

    let dir = temp_dir();
    let path = dir.join("language-reference.rupa");
    std::fs::write(&path, example).expect("write language reference example");
    assert_success(
        &["check", path.to_str().expect("utf-8 reference path")],
        "language reference example",
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn checked_in_examples_are_fmt_stable() {
    let examples_dir = workspace_root().join("examples");
    let files = rupa_files(&examples_dir);
    assert!(!files.is_empty(), "no examples found in {examples_dir:?}");

    for file in &files {
        let source = std::fs::read_to_string(file).expect("read example");
        let formatted = rupa_syntax::format_source(&source)
            .unwrap_or_else(|error| panic!("{} failed to format: {error:?}", file.display()));
        assert_eq!(formatted, source, "{} is not fmt-stable", file.display());
    }
}
