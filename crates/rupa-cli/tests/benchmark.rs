//! Integration tests for the offline LLM benchmark harness.

mod common;

use common::{run_rupa, temp_dir, workspace_root};
use serde_json::Value;
use std::path::{Path, PathBuf};

fn suite() -> (PathBuf, Value) {
    let tasks_dir = workspace_root().join("benchmarks");
    let source = std::fs::read_to_string(tasks_dir.join("tasks.json")).expect("read tasks.json");
    let suite: Value = serde_json::from_str(&source).expect("parse tasks.json");
    (tasks_dir, suite)
}

fn tasks(suite: &Value) -> &[Value] {
    suite["tasks"].as_array().expect("tasks array")
}

fn task_by_id<'a>(suite: &'a Value, id: &str) -> &'a Value {
    tasks(suite)
        .iter()
        .find(|task| task["id"] == id)
        .expect("benchmark task")
}

fn run_bench(tasks_dir: &Path, candidates: &Path, output: Option<&Path>) -> Value {
    let mut args = vec![
        "bench".to_string(),
        "--tasks".to_string(),
        tasks_dir.to_str().unwrap().to_string(),
        "--candidates".to_string(),
        candidates.to_str().unwrap().to_string(),
    ];
    if let Some(output) = output {
        args.push("--output".to_string());
        args.push(output.to_str().unwrap().to_string());
    }
    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let result = run_rupa(&arg_refs);
    assert_eq!(
        result.status.code(),
        Some(0),
        "bench failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    if let Some(output) = output {
        let source = std::fs::read_to_string(output).expect("read benchmark report");
        serde_json::from_str(&source).expect("parse benchmark report")
    } else {
        serde_json::from_slice(&result.stdout).expect("parse benchmark report from stdout")
    }
}

fn copy_reference_solutions(tasks_dir: &Path, suite: &Value, candidates: &Path) {
    for task in tasks(suite) {
        let id = task["id"].as_str().expect("task id");
        let solution = task["solution"].as_str().expect("task solution");
        std::fs::copy(
            tasks_dir.join(solution),
            candidates.join(format!("{id}.rupa")),
        )
        .expect("copy reference solution");
    }
}

#[test]
fn reference_solutions_score_perfectly() {
    let (tasks_dir, suite) = suite();
    let candidates = temp_dir();
    copy_reference_solutions(&tasks_dir, &suite, &candidates);

    let report_path = candidates.join("report.json");
    let report = run_bench(&tasks_dir, &candidates, Some(&report_path));

    assert_eq!(report["task_count"], tasks(&suite).len());
    assert_eq!(report["attempted_count"], tasks(&suite).len());
    assert_eq!(report["total_attempts"], tasks(&suite).len());
    assert_eq!(report["first_attempt_compilation_success_rate"], 1.0);
    assert_eq!(report["compilation_success_rate"], 1.0);
    assert_eq!(report["rendering_success_rate"], 1.0);
    assert_eq!(report["syntax_error_rate"], 0.0);
    assert_eq!(report["semantic_error_rate"], 0.0);
    assert_eq!(report["average_repair_iterations"], 0.0);
    assert_eq!(report["average_visual_similarity"], 1.0);
    assert!(report["average_token_count"].as_f64().unwrap() > 0.0);
    assert!(report["average_source_length_bytes"].as_f64().unwrap() > 0.0);

    for task in report["tasks"].as_array().expect("task reports") {
        assert_eq!(task["attempted"], true);
        assert_eq!(task["attempt_count"], 1);
        assert_eq!(task["final_compiles"], true);
        assert_eq!(task["render_success"], true);
        assert_eq!(task["visual_similarity"], 1.0);
        assert_eq!(task["pixel_match_ratio"], 1.0);
    }

    let _ = std::fs::remove_dir_all(candidates);
}

#[test]
fn repair_attempts_are_counted_and_first_attempt_failure_is_reported() {
    let (tasks_dir, suite) = suite();
    let candidates = temp_dir();
    let repair = task_by_id(&suite, "repair-invalid");
    let starter = repair["starter"].as_str().expect("repair starter");
    let solution = repair["solution"].as_str().expect("repair solution");

    let attempt_dir = candidates.join("repair-invalid");
    std::fs::create_dir_all(&attempt_dir).expect("create attempt directory");
    std::fs::copy(tasks_dir.join(starter), attempt_dir.join("attempt-01.rupa"))
        .expect("copy starter");
    std::fs::copy(
        tasks_dir.join(solution),
        attempt_dir.join("attempt-02.rupa"),
    )
    .expect("copy solution");

    let report = run_bench(&tasks_dir, &candidates, None);
    assert_eq!(report["task_count"], tasks(&suite).len());
    assert_eq!(report["attempted_count"], 1);
    assert_eq!(report["total_attempts"], 2);
    assert_eq!(report["first_attempt_compilation_success_rate"], 0.0);
    assert_eq!(report["compilation_success_rate"], 1.0);
    assert_eq!(report["rendering_success_rate"], 1.0);
    assert_eq!(report["semantic_error_rate"], 0.5);
    assert_eq!(report["syntax_error_rate"], 0.0);
    assert_eq!(report["average_repair_iterations"], 1.0);
    assert_eq!(report["average_visual_similarity"], 1.0);

    let task = report["tasks"]
        .as_array()
        .expect("task reports")
        .iter()
        .find(|task| task["id"] == "repair-invalid")
        .expect("repair-invalid report");
    assert_eq!(task["attempt_count"], 2);
    assert_eq!(task["repair_iterations"], 1);
    assert_eq!(task["first_attempt_compiles"], false);
    assert_eq!(task["final_compiles"], true);
    let first = &task["attempts"][0];
    assert_eq!(first["compiles"], false);
    assert_eq!(first["semantic_errors"], 1);
    assert_eq!(first["diagnostic_codes"][0], "S005");

    let _ = std::fs::remove_dir_all(candidates);
}

#[test]
fn failed_candidate_scores_zero_without_failing_the_harness() {
    let (tasks_dir, suite) = suite();
    let candidates = temp_dir();
    std::fs::write(
        candidates.join("basic-shape.rupa"),
        "size 10px x 10px\ncircle center at (5px, 5px)\n    fill red\n",
    )
    .expect("write broken candidate");

    let report = run_bench(&tasks_dir, &candidates, None);
    assert_eq!(report["task_count"], tasks(&suite).len());
    assert_eq!(report["attempted_count"], 1);
    assert_eq!(report["compilation_success_rate"], 0.0);
    assert_eq!(report["rendering_success_rate"], 0.0);
    assert_eq!(report["average_visual_similarity"], 0.0);
    assert!(report["semantic_error_rate"].as_f64().unwrap() > 0.0);

    let task = report["tasks"]
        .as_array()
        .expect("task reports")
        .iter()
        .find(|task| task["id"] == "basic-shape")
        .expect("basic-shape report");
    assert_eq!(task["final_compiles"], false);
    assert_eq!(task["render_success"], false);
    assert_eq!(task["visual_similarity"], Value::Null);

    let _ = std::fs::remove_dir_all(candidates);
}

#[test]
fn unattempted_tasks_are_reported() {
    let (tasks_dir, suite) = suite();
    let candidates = temp_dir();
    let report = run_bench(&tasks_dir, &candidates, None);
    assert_eq!(report["task_count"], tasks(&suite).len());
    assert_eq!(report["attempted_count"], 0);
    assert_eq!(report["total_attempts"], 0);
    assert_eq!(report["compilation_success_rate"], 0.0);
    assert_eq!(report["average_visual_similarity"], 0.0);
    let _ = std::fs::remove_dir_all(candidates);
}
