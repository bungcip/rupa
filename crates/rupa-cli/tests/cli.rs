//! Integration tests for the `rupa` binary.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

const VALID_SOURCE: &str = r#"size 200px x 100px
background white

rectangle at (10px, 10px)
    size 80px x 40px
    fill red

circle center at (150px, 50px)
    radius 30px
    fill blue

"#;

const CANONICAL_SOURCE: &str = r#"size 200px x 100px
background white

rectangle at (10px, 10px)
    size 80px x 40px
    fill red

circle center at (150px, 50px)
    radius 30px
    fill blue
"#;

fn temp_dir() -> PathBuf {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("rupa-cli-test-{}-{id}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn write_source(dir: &Path, name: &str, source: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, source).expect("write source");
    path
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rupa"))
        .args(args)
        .output()
        .expect("run rupa")
}

fn run_in(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rupa"))
        .args(args)
        .current_dir(dir)
        .output()
        .expect("run rupa")
}

fn run_with_stdin(args: &[&str], input: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rupa"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn rupa");
    child
        .stdin
        .take()
        .expect("stdin pipe")
        .write_all(input.as_bytes())
        .expect("write stdin");
    child.wait_with_output().expect("wait for rupa")
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn decode(path: &Path) -> (u32, u32, Vec<u8>) {
    let file = std::fs::File::open(path).expect("open PNG");
    let decoder = png::Decoder::new(std::io::BufReader::new(file));
    let mut reader = decoder.read_info().expect("read PNG info");
    let mut buffer = vec![0; reader.output_buffer_size().expect("known output size")];
    let info = reader.next_frame(&mut buffer).expect("decode PNG");
    buffer.truncate(info.buffer_size());
    (info.width, info.height, buffer)
}

fn pixel_at(pixels: &[u8], width: u32, x: u32, y: u32) -> [u8; 4] {
    let index = ((y * width + x) * 4) as usize;
    [
        pixels[index],
        pixels[index + 1],
        pixels[index + 2],
        pixels[index + 3],
    ]
}

#[test]
fn top_level_help_lists_check_and_render() {
    let output = run(&["--help"]);
    assert_eq!(output.status.code(), Some(0));
    let help = stdout(&output);
    assert!(help.contains("check"));
    assert!(help.contains("render"));
    assert!(help.contains("Commands:"));
}

#[test]
fn check_help_shows_input_argument() {
    let output = run(&["check", "--help"]);
    assert_eq!(output.status.code(), Some(0));
    let help = stdout(&output);
    assert!(help.contains("<INPUT>") || help.contains("INPUT"));
}

#[test]
fn render_help_shows_output_option() {
    let output = run(&["render", "--help"]);
    assert_eq!(output.status.code(), Some(0));
    let help = stdout(&output);
    assert!(help.contains("--output"));
}

#[test]
fn check_valid_file_succeeds_quietly() {
    let dir = temp_dir();
    let input = write_source(&dir, "valid.rupa", VALID_SOURCE);
    let output = run(&["check", input.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).is_empty(), "stderr: {}", stderr(&output));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn check_renders_source_line_caret_and_span_label() {
    let dir = temp_dir();
    let source =
        "size 100px x 100px\nrectangle at (10px, 10px)\n    size 20px x 20px\n    fil red\n";
    let input = write_source(&dir, "label.rupa", source);
    let output = run(&["check", input.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));

    let error = stderr(&output);
    assert!(error.contains("4 |     fil red\n"), "stderr: {error}");
    assert!(
        error.contains("|     ^^^ unknown keyword or property `fil` in rectangle\n"),
        "stderr: {error}"
    );
    assert!(
        error.contains("= help: Expected at, size, fill, stroke, width, rotate, scale, or opacity; did you mean `fill`?\n"),
        "stderr: {error}"
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn fmt_canonicalizes_document_settings_and_is_idempotent() {
    let dir = temp_dir();
    let input = write_source(
        &dir,
        "fmt.rupa",
        "background white\nrectangle at (10px, 10px)\n    size 20px x 20px\n    fill red\nsize 100px x 100px\n",
    );
    let output = run(&["fmt", input.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    let formatted = std::fs::read_to_string(&input).expect("read formatted source");
    assert!(formatted.starts_with("size 100px x 100px\nbackground white\n\nrectangle"));
    assert!(!formatted.contains("document"));

    let output = run(&["fmt", input.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    assert_eq!(
        std::fs::read_to_string(&input).expect("read formatted source"),
        formatted,
        "fmt must be idempotent"
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn fmt_help_shows_check_and_stdin() {
    let output = run(&["fmt", "--help"]);
    assert_eq!(output.status.code(), Some(0));
    let help = stdout(&output);
    assert!(help.contains("--check"), "help: {help}");
    assert!(help.contains("--stdin"), "help: {help}");
}

#[test]
fn fmt_check_exits_one_for_unformatted_file_without_rewriting() {
    let dir = temp_dir();
    let original = "background white\nsize 100px x 100px\n";
    let input = write_source(&dir, "check.rupa", original);
    let output = run(&["fmt", "--check", input.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    assert!(stdout(&output).is_empty());
    assert!(
        stderr(&output).contains("is not formatted"),
        "stderr: {}",
        stderr(&output)
    );
    assert_eq!(
        std::fs::read_to_string(&input).expect("read checked source"),
        original,
        "--check must not rewrite the file"
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn fmt_check_succeeds_for_canonical_file() {
    let dir = temp_dir();
    let input = write_source(&dir, "canonical.rupa", CANONICAL_SOURCE);
    let output = run(&["fmt", "--check", input.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).is_empty(), "stderr: {}", stderr(&output));
    assert_eq!(
        std::fs::read_to_string(&input).expect("read checked source"),
        CANONICAL_SOURCE
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn fmt_stdin_writes_canonical_source_to_stdout() {
    let input = "background white\nrectangle at (10px, 10px)\n    size 20px x 20px\n    fill red\nsize 100px x 100px\n";
    let output = run_with_stdin(&["fmt", "--stdin"], input);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    let formatted = stdout(&output);
    assert!(formatted.starts_with("size 100px x 100px\nbackground white\n\nrectangle"));
    assert!(stderr(&output).is_empty(), "stderr: {}", stderr(&output));
}

#[test]
fn fmt_stdin_check_reports_without_writing_stdout() {
    let output = run_with_stdin(
        &["fmt", "--stdin", "--check"],
        "background white\nsize 100px x 100px\n",
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(stdout(&output).is_empty());
    assert!(
        stderr(&output).contains("<stdin> is not formatted"),
        "stderr: {}",
        stderr(&output)
    );

    let output = run_with_stdin(&["fmt", "--stdin", "--check"], CANONICAL_SOURCE);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).is_empty(), "stderr: {}", stderr(&output));
}

#[test]
fn fmt_stdin_reports_diagnostics_without_stdout() {
    let output = run_with_stdin(
        &["fmt", "--stdin"],
        "size 100px x 100px\n\tbackground white\n",
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(stdout(&output).is_empty());
    assert!(
        stderr(&output).contains("error[L002]"),
        "stderr: {}",
        stderr(&output)
    );
}

#[test]
fn fmt_stdin_conflicts_with_input_path() {
    let output = run(&["fmt", "--stdin", "unused.rupa"]);
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn fmt_rejects_lexical_errors_without_rewriting() {
    let dir = temp_dir();
    let original = "size 100px x 100px\n\tbackground white\n";
    let input = write_source(&dir, "invalid.rupa", original);
    let output = run(&["fmt", input.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("error[L002]"),
        "stderr: {}",
        stderr(&output)
    );
    assert_eq!(
        std::fs::read_to_string(&input).expect("read invalid source"),
        original,
        "failed fmt must not rewrite the file"
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn inspect_reports_implicit_root_layer() {
    let dir = temp_dir();
    let input = write_source(&dir, "inspect.rupa", VALID_SOURCE);
    let output = run(&["inspect", input.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("inspect JSON");
    assert_eq!(report["width"], 200);
    assert_eq!(report["height"], 100);
    assert_eq!(report["root"]["name"], "root");
    assert_eq!(report["root"]["width"], 200);
    assert_eq!(report["root"]["height"], 100);
    assert_eq!(report["root"]["clip"], false);
    assert_eq!(report["root"]["children"].as_array().unwrap().len(), 2);
    assert!(report["layers"].as_array().unwrap().is_empty());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn inspect_reports_groups_and_layer_planes() {
    let dir = temp_dir();
    let source = r#"size 64px x 40px
group "Tile"
    size 10px x 10px
    rectangle at (0px, 0px)
        size 10px x 10px
        fill red
layer "Sky"
    fill blue
    place "Tile" at (5px, 5px)
"#;
    let input = write_source(&dir, "groups.rupa", source);
    let output = run(&["inspect", input.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("inspect JSON");

    assert_eq!(report["groups"][0]["name"], "Tile");
    assert_eq!(report["groups"][0]["bounds"]["width"], 10.0);
    assert_eq!(report["layers"][0]["name"], "Sky");
    // The layer is not placeable, so it appears as a root plane child.
    let root_children = report["root"]["children"]
        .as_array()
        .expect("root children");
    assert!(
        root_children
            .iter()
            .any(|child| child.get("Plane").is_some()),
        "root children: {root_children:?}"
    );
    // The placement lives inside the layer, referencing the group by name.
    assert_eq!(
        report["layers"][0]["children"][1]["Instance"]["group"],
        "Tile"
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn check_reports_missing_picture_resource_in_group() {
    let dir = temp_dir();
    let source = r#"size 40px x 40px
group "Tile"
    picture "missing.png" at (0px, 0px)
        size 10px x 10px
place "Tile" at (0px, 0px)
    z-index 0
"#;
    let input = write_source(&dir, "group-missing-pic.rupa", source);
    let output = run(&["check", input.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1), "stderr: {}", stderr(&output));
    let error = stderr(&output);
    assert!(error.contains("error[R001]"), "stderr: {error}");
    assert!(error.contains("missing.png"), "stderr: {error}");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn check_reports_unresolved_font_in_group() {
    let dir = temp_dir();
    let source = r#"size 40px x 40px
group "Tile"
    text "hi" at (0px, 0px)
        size 10px
        font "Definitely-Not-A-Font-XYZ"
place "Tile" at (0px, 0px)
    z-index 0
"#;
    let input = write_source(&dir, "group-missing-font.rupa", source);
    let output = run(&["check", input.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1), "stderr: {}", stderr(&output));
    let error = stderr(&output);
    assert!(error.contains("error[R003]"), "stderr: {error}");
    assert!(
        error.contains("Definitely-Not-A-Font-XYZ"),
        "stderr: {error}"
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn check_invalid_file_reports_diagnostic_and_exits_one() {
    let dir = temp_dir();
    let input = write_source(&dir, "invalid.rupa", "size 0px x 100px\n");
    let output = run(&["check", input.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    let error = stderr(&output);
    assert!(error.contains("error[C003]"), "stderr: {error}");
    assert!(error.contains("invalid.rupa:1:"), "stderr: {error}");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn check_reports_multiple_errors_together() {
    let dir = temp_dir();
    let input = write_source(&dir, "many.rupa", "rectangle\n    size 0px x 0px\n");
    let output = run(&["check", input.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    let error = stderr(&output);
    // Only the genuinely broken shape properties report.
    assert!(error.contains("S003"), "stderr: {error}");
    assert!(error.contains("S001"), "stderr: {error}");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn check_missing_input_file_errors() {
    let dir = temp_dir();
    let missing = dir.join("missing.rupa");
    let output = run(&["check", missing.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("failed to read input file"));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn render_writes_png_with_expected_dimensions_and_pixels() {
    let dir = temp_dir();
    let input = write_source(&dir, "valid.rupa", VALID_SOURCE);
    let png = dir.join("out.png");
    let output = run(&[
        "render",
        input.to_str().unwrap(),
        "--output",
        png.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    assert!(png.is_file());

    let (width, height, pixels) = decode(&png);
    assert_eq!((width, height), (200, 100));
    assert_eq!(pixel_at(&pixels, width, 20, 20), [255, 0, 0, 255]);
    assert_eq!(pixel_at(&pixels, width, 150, 50), [0, 0, 255, 255]);
    assert_eq!(pixel_at(&pixels, width, 190, 90), [255, 255, 255, 255]);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn render_without_size_and_background_uses_documented_defaults() {
    let dir = temp_dir();
    let input = write_source(
        &dir,
        "minimal.rupa",
        "circle center at (128px, 128px)\n    radius 20px\n    fill red\n",
    );

    let check = run(&["check", input.to_str().unwrap()]);
    assert_eq!(check.status.code(), Some(0), "stderr: {}", stderr(&check));

    let png = dir.join("out.png");
    let output = run(&[
        "render",
        input.to_str().unwrap(),
        "--output",
        png.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));

    let (width, height, pixels) = decode(&png);
    assert_eq!((width, height), (256, 256));
    assert_eq!(pixel_at(&pixels, width, 128, 128), [255, 0, 0, 255]);
    // Omitted `background` leaves the canvas transparent.
    assert_eq!(pixel_at(&pixels, width, 0, 0), [0, 0, 0, 0]);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn render_ellipse_square_and_triangle_end_to_end() {
    let dir = temp_dir();
    let source = r#"size 200px x 100px
background white

ellipse center at (50px, 50px)
    size 40px x 20px
    fill orange

square at (100px, 10px)
    size 20px
    fill red

triangle at (150px, 50px)
    size 30px x 30px
    fill purple

"#;
    let input = write_source(&dir, "shapes.rupa", source);
    let png = dir.join("out.png");
    let output = run(&[
        "render",
        input.to_str().unwrap(),
        "--output",
        png.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));

    let (width, height, pixels) = decode(&png);
    assert_eq!((width, height), (200, 100));
    assert_eq!(pixel_at(&pixels, width, 50, 50), [255, 165, 0, 255]);
    assert_eq!(pixel_at(&pixels, width, 110, 20), [255, 0, 0, 255]);
    assert_eq!(pixel_at(&pixels, width, 165, 70), [128, 0, 128, 255]);
    assert_eq!(pixel_at(&pixels, width, 50, 10), [255, 255, 255, 255]);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn render_polygon_end_to_end() {
    let dir = temp_dir();
    let source = r#"size 100px x 100px
background white

polygon
    points
        (20px, 20px)
        (80px, 20px)
        (50px, 80px)
    fill purple

"#;
    let input = write_source(&dir, "polygon.rupa", source);
    let png = dir.join("out.png");
    let output = run(&[
        "render",
        input.to_str().unwrap(),
        "--output",
        png.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));

    let (width, height, pixels) = decode(&png);
    assert_eq!((width, height), (100, 100));
    assert_eq!(pixel_at(&pixels, width, 50, 40), [128, 0, 128, 255]);
    assert_eq!(pixel_at(&pixels, width, 10, 10), [255, 255, 255, 255]);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn check_reports_polygon_with_too_few_points() {
    let dir = temp_dir();
    let input = write_source(
        &dir,
        "polygon-invalid.rupa",
        r#"size 100px x 100px
polygon
    points
        (0px, 0px)
        (10px, 0px)
    fill red
"#,
    );
    let output = run(&["check", input.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("error[S009]"),
        "stderr: {}",
        stderr(&output)
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn render_line_and_polyline_end_to_end() {
    let dir = temp_dir();
    let source = r#"size 100px x 100px
background white

line from (10px, 20px) to (90px, 20px)
    stroke black
    width 4px

polyline
    points
        (10px, 80px)
        (30px, 60px)
        (50px, 80px)
    stroke blue
    width 4px

"#;
    let input = write_source(&dir, "lines.rupa", source);
    let png = dir.join("out.png");
    let output = run(&[
        "render",
        input.to_str().unwrap(),
        "--output",
        png.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));

    let (width, height, pixels) = decode(&png);
    assert_eq!((width, height), (100, 100));
    assert_eq!(pixel_at(&pixels, width, 50, 20), [0, 0, 0, 255]);
    assert_eq!(pixel_at(&pixels, width, 20, 70), [0, 0, 255, 255]);
    assert_eq!(pixel_at(&pixels, width, 40, 70), [0, 0, 255, 255]);
    // Polyline stays open: no segment along the bottom.
    assert_eq!(pixel_at(&pixels, width, 30, 80), [255, 255, 255, 255]);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn check_reports_line_without_stroke() {
    let dir = temp_dir();
    let input = write_source(
        &dir,
        "line-invalid.rupa",
        r#"size 100px x 100px
line from (0px, 0px) to (10px, 10px)
    width 2px
"#,
    );
    let output = run(&["check", input.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("error[S010]"),
        "stderr: {}",
        stderr(&output)
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn render_path_with_fill_and_stroke_end_to_end() {
    let dir = temp_dir();
    let source = r#"size 100px x 100px
background white

path
    start at (20px, 20px)
    line to (80px, 20px)
    curve to (80px, 80px)
        control (100px, 40px) and (100px, 60px)
    line to (20px, 80px)
    close
    fill purple
    stroke black
    width 2px

"#;
    let input = write_source(&dir, "path.rupa", source);
    let png = dir.join("out.png");
    let output = run(&[
        "render",
        input.to_str().unwrap(),
        "--output",
        png.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));

    let (width, height, pixels) = decode(&png);
    assert_eq!((width, height), (100, 100));
    assert_eq!(pixel_at(&pixels, width, 50, 40), [128, 0, 128, 255]);
    assert_eq!(pixel_at(&pixels, width, 10, 10), [255, 255, 255, 255]);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn check_reports_path_without_start() {
    let dir = temp_dir();
    let input = write_source(
        &dir,
        "path-invalid.rupa",
        r#"size 100px x 100px
path
    line to (10px, 10px)
    fill red
"#,
    );
    let output = run(&["check", input.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("error[S013]"),
        "stderr: {}",
        stderr(&output)
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn render_rectangle_with_fill_and_stroke_end_to_end() {
    let dir = temp_dir();
    let source = r#"size 60px x 60px
background white

rectangle at (10px, 10px)
    size 40px x 40px
    fill blue
    stroke black
    width 4px

"#;
    let input = write_source(&dir, "appearance.rupa", source);
    let png = dir.join("out.png");
    let output = run(&[
        "render",
        input.to_str().unwrap(),
        "--output",
        png.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));

    let (width, height, pixels) = decode(&png);
    assert_eq!((width, height), (60, 60));
    assert_eq!(pixel_at(&pixels, width, 30, 30), [0, 0, 255, 255]);
    assert_eq!(pixel_at(&pixels, width, 10, 30), [0, 0, 0, 255]);
    assert_eq!(pixel_at(&pixels, width, 5, 5), [255, 255, 255, 255]);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn render_transform_properties_end_to_end() {
    let dir = temp_dir();
    let source = r#"size 100px x 100px
background white

rectangle at (30px, 30px)
    size 40px x 20px
    fill blue
    rotate 90deg

rectangle at (10px, 10px)
    size 20px x 20px
    fill red
    scale 2
    opacity 0.5

"#;
    let input = write_source(&dir, "transforms.rupa", source);
    let png = dir.join("out.png");
    let output = run(&[
        "render",
        input.to_str().unwrap(),
        "--output",
        png.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));

    let (width, height, pixels) = decode(&png);
    assert_eq!((width, height), (100, 100));
    // Rotated rectangle occupies x 40..60, y 20..60 around center (50,40).
    assert_eq!(pixel_at(&pixels, width, 50, 25), [0, 0, 255, 255]);
    // Scaled 50% red square blends with white.
    assert_eq!(pixel_at(&pixels, width, 20, 20), [255, 127, 127, 255]);
    assert_eq!(pixel_at(&pixels, width, 70, 70), [255, 255, 255, 255]);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn render_stroke_cap_and_join_end_to_end() {
    let dir = temp_dir();
    let source = r#"size 100px x 60px
background white

line from (30px, 30px) to (70px, 30px)
    stroke black
    width 10px
    cap square
    join round

"#;
    let input = write_source(&dir, "stroke-style.rupa", source);
    let png = dir.join("out.png");
    let output = run(&[
        "render",
        input.to_str().unwrap(),
        "--output",
        png.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));

    let (width, height, pixels) = decode(&png);
    assert_eq!((width, height), (100, 60));
    assert_eq!(pixel_at(&pixels, width, 50, 30), [0, 0, 0, 255]);
    // Square cap extends half the stroke width beyond the endpoint.
    assert_eq!(pixel_at(&pixels, width, 26, 30), [0, 0, 0, 255]);
    assert_eq!(pixel_at(&pixels, width, 23, 30), [255, 255, 255, 255]);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn check_group_definitions_without_placements() {
    let dir = temp_dir();
    let source = r#"size 300px x 200px
group "Cloud"
    size 240px x 120px
    circle center at (60px, 60px)
        radius 40px
        fill white
"#;
    let input = write_source(&dir, "layers.rupa", source);
    let output = run(&["check", input.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    assert!(stderr(&output).is_empty(), "stderr: {}", stderr(&output));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn render_layer_without_size_and_background_fill() {
    let dir = temp_dir();
    let source = r#"size 60px x 40px
layer "Sky"
    fill blue
    z-index 1

    circle center at (30px, 20px)
        radius 5px
        fill red
"#;
    let input = write_source(&dir, "layer-defaults.rupa", source);
    let png = dir.join("out.png");
    let output = run(&[
        "render",
        input.to_str().unwrap(),
        "--output",
        png.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));

    let (width, height, pixels) = decode(&png);
    // Omitted layer size defaults to the document canvas.
    assert_eq!((width, height), (60, 40));
    assert_eq!(pixel_at(&pixels, width, 2, 2), [0, 0, 255, 255]);
    assert_eq!(pixel_at(&pixels, width, 57, 37), [0, 0, 255, 255]);
    assert_eq!(pixel_at(&pixels, width, 30, 20), [255, 0, 0, 255]);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn check_reports_duplicate_group_names() {
    let dir = temp_dir();
    let source = r#"size 300px x 200px
group "Cloud"
    size 10px x 10px
group "Cloud"
    size 10px x 10px
"#;
    let input = write_source(&dir, "duplicate-groups.rupa", source);
    let output = run(&["check", input.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("error[S030]"),
        "stderr: {}",
        stderr(&output)
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn render_two_group_instances_end_to_end() {
    let dir = temp_dir();
    let source = r#"size 200px x 100px
background white

group "Dot"
    size 20px x 20px
    rectangle at (0px, 0px)
        size 20px x 20px
        fill red

place "Dot" at (10px, 10px)
    z-index 0

place "Dot" at (50px, 10px)
    z-index 1

place "Dot" at (90px, 10px)
    scale 0.5
    opacity 0.5
    z-index 2

"#;
    let input = write_source(&dir, "layers-instances.rupa", source);
    let png = dir.join("out.png");
    let output = run(&[
        "render",
        input.to_str().unwrap(),
        "--output",
        png.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));

    let (width, height, pixels) = decode(&png);
    assert_eq!((width, height), (200, 100));
    assert_eq!(pixel_at(&pixels, width, 15, 15), [255, 0, 0, 255]);
    assert_eq!(pixel_at(&pixels, width, 55, 15), [255, 0, 0, 255]);
    assert_eq!(pixel_at(&pixels, width, 35, 15), [255, 255, 255, 255]);
    // Third instance is scaled/opacity-blended around the group center.
    assert_eq!(pixel_at(&pixels, width, 100, 20), [255, 127, 127, 255]);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn render_orders_group_instances_by_z_index() {
    let dir = temp_dir();
    let source = r#"size 100px x 100px
background white

group "High"
    size 20px x 20px
    rectangle at (0px, 0px)
        size 20px x 20px
        fill blue

group "Low"
    size 20px x 20px
    rectangle at (0px, 0px)
        size 20px x 20px
        fill green

place "High" at (20px, 20px)
    z-index 5

place "Low" at (25px, 25px)
    z-index 1

rectangle at (10px, 10px)
    size 60px x 60px
    fill red

"#;
    let input = write_source(&dir, "z-order.rupa", source);
    let png = dir.join("out.png");
    let output = run(&[
        "render",
        input.to_str().unwrap(),
        "--output",
        png.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));

    let (width, height, pixels) = decode(&png);
    assert_eq!((width, height), (100, 100));
    // Root shape participates at z-index 0: visible where no instance covers it.
    assert_eq!(pixel_at(&pixels, width, 15, 15), [255, 0, 0, 255]);
    // "High" (z-index 5) covers the overlapping, later-declared "Low" (1).
    assert_eq!(pixel_at(&pixels, width, 30, 30), [0, 0, 255, 255]);
    assert_eq!(pixel_at(&pixels, width, 43, 43), [0, 255, 0, 255]);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn check_reports_unknown_group_placement() {
    let dir = temp_dir();
    let source = r#"size 100px x 100px
place "Missing" at (0px, 0px)
    z-index 0
"#;
    let input = write_source(&dir, "unknown-group.rupa", source);
    let output = run(&["check", input.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("error[S033]"),
        "stderr: {}",
        stderr(&output)
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn check_and_render_reject_recursive_group_reference() {
    let dir = temp_dir();
    let source = r#"size 100px x 100px
group "A"
    size 10px x 10px
    place "B" at (0px, 0px)
        z-index 0
group "B"
    size 10px x 10px
    place "A" at (0px, 0px)
        z-index 0
"#;
    let input = write_source(&dir, "cycle.rupa", source);

    let check = run(&["check", input.to_str().unwrap()]);
    assert_eq!(check.status.code(), Some(1));
    let diagnostics = stderr(&check);
    assert!(diagnostics.contains("error[S035]"), "stderr: {diagnostics}");
    assert!(
        diagnostics.contains("recursive group reference"),
        "stderr: {diagnostics}"
    );

    let png = dir.join("out.png");
    let render = run(&[
        "render",
        input.to_str().unwrap(),
        "--output",
        png.to_str().unwrap(),
    ]);
    assert_eq!(render.status.code(), Some(1));
    assert!(!png.exists(), "cyclic document must not write a PNG");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn render_instance_maps_local_origin_to_placement() {
    let dir = temp_dir();
    let source = r#"size 100px x 100px
background white

group "Tile"
    size 20px x 20px
    rectangle at (5px, 5px)
        size 10px x 10px
        fill blue

place "Tile" at (30px, 40px)
    z-index 0

"#;
    let input = write_source(&dir, "local.rupa", source);
    let png = dir.join("out.png");
    let output = run(&[
        "render",
        input.to_str().unwrap(),
        "--output",
        png.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));

    let (width, height, pixels) = decode(&png);
    assert_eq!((width, height), (100, 100));
    // Local child (5,5)-(15,15) maps to (35,45)-(45,55).
    assert_eq!(pixel_at(&pixels, width, 40, 50), [0, 0, 255, 255]);
    assert_eq!(pixel_at(&pixels, width, 32, 42), [255, 255, 255, 255]);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn render_nested_group_composition_end_to_end() {
    let dir = temp_dir();
    let source = r#"size 120px x 100px
background white

group "Branch"
    size 100px x 40px
    rectangle at (0px, 0px)
        size 100px x 40px
        fill blue
    place "Leaf" at (80px, 20px)
        z-index 1

group "Leaf"
    size 20px x 20px
    circle center at (10px, 10px)
        radius 10px
        fill green

place "Branch" at (10px, 20px)
    z-index 0

"#;
    let input = write_source(&dir, "nested.rupa", source);
    let png = dir.join("out.png");
    let output = run(&[
        "render",
        input.to_str().unwrap(),
        "--output",
        png.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));

    let (width, height, pixels) = decode(&png);
    assert_eq!((width, height), (120, 100));
    // Branch rectangle at document (10,20)-(110,60).
    assert_eq!(pixel_at(&pixels, width, 20, 30), [0, 0, 255, 255]);
    // Leaf center (100,50) is nested inside Branch at local (80,20).
    assert_eq!(pixel_at(&pixels, width, 100, 50), [0, 255, 0, 255]);
    assert_eq!(pixel_at(&pixels, width, 5, 5), [255, 255, 255, 255]);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn render_defaults_output_to_input_filename_in_current_directory() {
    let dir = temp_dir();
    write_source(&dir, "valid.rupa", VALID_SOURCE);

    let output = run_in(&dir, &["render", "valid.rupa"]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));

    let png = dir.join("valid.png");
    assert!(png.is_file(), "default output must be valid.png");
    let (width, height, pixels) = decode(&png);
    assert_eq!((width, height), (200, 100));
    assert_eq!(pixel_at(&pixels, width, 20, 20), [255, 0, 0, 255]);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn render_invalid_source_does_not_write_png() {
    let dir = temp_dir();
    let input = write_source(&dir, "invalid.rupa", "size 0px x 100px\n");
    let png = dir.join("out.png");
    let output = run(&[
        "render",
        input.to_str().unwrap(),
        "--output",
        png.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("error[C003]"));
    assert!(!png.exists());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn render_rejects_non_png_output_paths() {
    let dir = temp_dir();
    let input = write_source(&dir, "valid.rupa", VALID_SOURCE);

    for name in ["out.jpg", "out"] {
        let output_path = dir.join(name);
        let output = run(&[
            "render",
            input.to_str().unwrap(),
            "--output",
            output_path.to_str().unwrap(),
        ]);
        assert_eq!(output.status.code(), Some(1), "case: {name}");
        let error = stderr(&output);
        assert!(
            error.contains("must end in '.png'"),
            "case: {name}; stderr: {error}"
        );
        assert!(!output_path.exists(), "case: {name} wrote an output file");
    }

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn render_accepts_uppercase_png_extension() {
    let dir = temp_dir();
    let input = write_source(&dir, "valid.rupa", VALID_SOURCE);
    let output_path = dir.join("OUT.PNG");
    let output = run(&[
        "render",
        input.to_str().unwrap(),
        "--output",
        output_path.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    assert!(output_path.is_file());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn render_honours_explicit_output_path() {
    let dir = temp_dir();
    let input = write_source(&dir, "valid.rupa", VALID_SOURCE);
    let output_path = dir.join("custom-name.png");
    let output = run(&[
        "render",
        input.to_str().unwrap(),
        "--output",
        output_path.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    assert!(output_path.is_file());
    assert!(!dir.join("valid.png").exists());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn check_rejects_legacy_export_as_unknown_keyword() {
    let dir = temp_dir();
    let input = write_source(
        &dir,
        "legacy-export.rupa",
        "size 100px x 100px\nexport \"out.png\"\n",
    );
    let output = run(&["check", input.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    let error = stderr(&output);
    assert!(error.contains("error[L006]"), "stderr: {error}");
    assert!(error.contains("export"), "stderr: {error}");
    let _ = std::fs::remove_dir_all(dir);
}
