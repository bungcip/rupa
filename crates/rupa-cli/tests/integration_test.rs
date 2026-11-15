//! End-to-end and golden-image integration tests for the Rupa pipeline.

mod common;

use common::{
    assert_example_matches_golden, assert_images_equal, assert_render_matches_golden, decode_png,
    golden_path, minimal_example, run_rupa, temp_dir, workspace_root,
};
use std::path::Path;
use std::process::Command;

/// Writes a 2x1 PNG: red on the left, blue on the right.
fn write_strip_png(path: &Path) {
    let file = std::fs::File::create(path).expect("create strip PNG");
    let mut encoder = png::Encoder::new(file, 2, 1);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().expect("write PNG header");
    writer
        .write_image_data(&[255, 0, 0, 255, 0, 0, 255, 255])
        .expect("write PNG pixels");
}

#[test]
fn minimal_example_compiles_without_errors() {
    let output = run_rupa(&["check", minimal_example().to_str().unwrap()]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "check failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "expected no diagnostics, got: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
}

#[test]
fn rendered_output_matches_golden_reference_pixel_exactly() {
    assert_example_matches_golden("minimal.png");
}

#[test]
fn golden_reference_has_expected_dimensions_and_colors() {
    let golden = decode_png(&golden_path("minimal.png"));
    assert_eq!((golden.width, golden.height), (200, 100));

    // White document background.
    assert_eq!(golden.pixel(0, 0), [255, 255, 255, 255]);
    assert_eq!(golden.pixel(190, 90), [255, 255, 255, 255]);
    // Rectangle at (10,10) size 80x40, filled red.
    assert_eq!(golden.pixel(20, 20), [255, 0, 0, 255]);
    assert_eq!(golden.pixel(89, 49), [255, 0, 0, 255]);
    // Circle centered at (150,50) radius 30, filled blue.
    assert_eq!(golden.pixel(150, 50), [0, 0, 255, 255]);
    // Outside both shapes.
    assert_eq!(golden.pixel(100, 80), [255, 255, 255, 255]);
}

#[test]
fn rendering_is_deterministic_across_runs() {
    let dir = temp_dir();
    let first = dir.join("first.png");
    let second = dir.join("second.png");
    let example = minimal_example();

    for output_path in [&first, &second] {
        let output = run_rupa(&[
            "render",
            example.to_str().unwrap(),
            "--output",
            output_path.to_str().unwrap(),
        ]);
        assert_eq!(
            output.status.code(),
            Some(0),
            "render failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let first_bytes = std::fs::read(&first).expect("read first PNG");
    let second_bytes = std::fs::read(&second).expect("read second PNG");
    assert_eq!(first_bytes, second_bytes, "PNG output is not byte-stable");

    let golden_bytes = std::fs::read(golden_path("minimal.png")).expect("read golden");
    assert_eq!(
        first_bytes, golden_bytes,
        "fresh render does not match golden bytes"
    );

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn golden_comparison_util_detects_mismatches() {
    // Guard the test infrastructure itself: a changed pixel must fail.
    let dir = temp_dir();
    let source = dir.join("mismatch.rupa");
    std::fs::write(
        &source,
        r#"size 200px x 100px
background white
rectangle at (10px, 10px)
    size 80px x 40px
    fill blue
"#,
    )
    .expect("write source");

    let output_path = dir.join("mismatch.png");
    let output = Command::new(env!("CARGO_BIN_EXE_rupa"))
        .args([
            "render",
            source.to_str().unwrap(),
            "--output",
            output_path.to_str().unwrap(),
        ])
        .output()
        .expect("run rupa");
    assert_eq!(output.status.code(), Some(0));

    let actual = decode_png(&output_path);
    let expected = decode_png(&golden_path("minimal.png"));
    let result = std::panic::catch_unwind(|| {
        assert_images_equal(&actual, &expected, "mismatch");
    });
    assert!(result.is_err(), "mismatched images must fail comparison");

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn picture_resource_renders_with_fit_mode() {
    let dir = temp_dir();
    write_strip_png(&dir.join("strip.png"));
    let source = dir.join("picture.rupa");
    std::fs::write(
        &source,
        r#"size 60px x 40px
background white
picture "strip.png"
    at (10px, 10px)
    size 20px x 20px
    fit fill
"#,
    )
    .expect("write source");

    let output_path = dir.join("picture.png");
    let output = run_rupa(&[
        "render",
        source.to_str().unwrap(),
        "--output",
        output_path.to_str().unwrap(),
    ]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "render failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let image = decode_png(&output_path);
    assert_eq!((image.width, image.height), (60, 40));
    assert_eq!(image.pixel(15, 15), [255, 0, 0, 255]);
    assert_eq!(image.pixel(25, 15), [0, 0, 255, 255]);
    // Outside the 20x20 frame, which starts at (10, 10).
    assert_eq!(image.pixel(9, 9), [255, 255, 255, 255]);
    assert_eq!(image.pixel(31, 31), [255, 255, 255, 255]);

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn missing_picture_resource_fails_render_with_path() {
    let dir = temp_dir();
    let source = dir.join("missing.rupa");
    std::fs::write(
        &source,
        r#"size 40px x 40px
background white
picture "missing.png"
    at (0px, 0px)
"#,
    )
    .expect("write source");

    let output_path = dir.join("missing.png");
    let output = run_rupa(&[
        "render",
        source.to_str().unwrap(),
        "--output",
        output_path.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("missing.png"),
        "diagnostic should name the resource: {stderr}"
    );
    assert!(
        !output_path.exists(),
        "failed render must not leave a PNG behind"
    );

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn group_placement_example_matches_golden_reference() {
    let source = workspace_root().join("examples/layers.rupa");
    assert_render_matches_golden(&source, "layers.png");
}

#[test]
fn layer_background_fill_example_matches_golden_reference() {
    let source = workspace_root().join("examples/layer-fill.rupa");
    assert_render_matches_golden(&source, "layer-fill.png");
}

#[test]
fn check_json_reports_ok_for_a_valid_source() {
    let output = run_rupa(&["check", minimal_example().to_str().unwrap(), "--json"]);
    assert_eq!(output.status.code(), Some(0));

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(json["status"], "ok");
    assert_eq!(json["diagnostics"].as_array().map(Vec::len), Some(0));
}

#[test]
fn check_json_reports_edits_for_an_unknown_keyword() {
    let dir = temp_dir();
    let source = dir.join("bad-keyword.rupa");
    std::fs::write(&source, "size 100px x 100px\nfil red\n").expect("write source");

    let output = run_rupa(&["check", source.to_str().unwrap(), "--json"]);
    assert_eq!(output.status.code(), Some(1), "check must fail");
    assert!(
        output.stderr.is_empty(),
        "json mode must not print human diagnostics: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(json["status"], "error");

    let diagnostic = &json["diagnostics"][0];
    assert_eq!(diagnostic["code"], "L006");
    assert_eq!(diagnostic["line"], 2);
    assert_eq!(diagnostic["column"], 1);
    assert_eq!(diagnostic["edits"][0]["replacement"], "fill");
    assert_eq!(diagnostic["edits"][0]["startLine"], 2);
    assert_eq!(diagnostic["edits"][0]["startOffset"], 19);
    assert_eq!(diagnostic["edits"][0]["endOffset"], 22);

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn check_json_reports_edits_for_an_unknown_named_color() {
    let dir = temp_dir();
    let source = dir.join("bad-color.rupa");
    std::fs::write(
        &source,
        "size 20px x 20px\nbackground white\nrectangle at (2px, 2px)\n    size 4px x 4px\n    fill gren\n",
    )
    .expect("write source");

    let output = run_rupa(&["check", source.to_str().unwrap(), "--json"]);
    assert_eq!(output.status.code(), Some(1), "check must fail");

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    let diagnostic = json["diagnostics"]
        .as_array()
        .expect("diagnostics array")
        .iter()
        .find(|entry| entry["code"] == "T002")
        .expect("T002 diagnostic");
    assert_eq!(diagnostic["edits"][0]["replacement"], "green");
    assert_eq!(diagnostic["edits"][0]["startLine"], 5);

    let _ = std::fs::remove_dir_all(dir);
}
