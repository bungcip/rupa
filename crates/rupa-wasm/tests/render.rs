//! Native tests for the browser binding: diagnostics, determinism, and parity
//! with the CLI's golden PNG.

use rupa_wasm::{FRAME_HEADER_BYTES, encode_frame, render_source};

/// Repository root, derived from this crate's manifest directory.
fn repo_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("resolve repository root")
}

/// Parse frame metadata JSON.
fn meta(frame: &rupa_wasm::Frame) -> serde_json::Value {
    serde_json::from_str(&frame.meta).expect("metadata is JSON")
}

#[test]
fn valid_source_renders_png_with_document_dimensions() {
    let frame = render_source("size 30px x 20px\nbackground red\n");
    assert!(frame.is_ok(), "metadata: {}", frame.meta);
    assert_eq!(&frame.png[..8], b"\x89PNG\r\n\x1a\n");

    let meta = meta(&frame);
    assert_eq!(meta["status"], "ok");
    assert_eq!(meta["width"], 30);
    assert_eq!(meta["height"], 20);
    assert!(meta["diagnostics"].as_array().unwrap().is_empty());
    assert!(meta["renderError"].is_null());
}

#[test]
fn invalid_source_reports_stable_diagnostic_with_position() {
    let frame = render_source("size 40px x 40px\ncircle center at (10px, 10px)\n    fill red\n");
    assert!(!frame.is_ok());
    assert!(frame.png.is_empty());

    let meta = meta(&frame);
    assert_eq!(meta["status"], "error");
    let diagnostics = meta["diagnostics"].as_array().expect("diagnostics array");
    let diagnostic = diagnostics
        .iter()
        .find(|entry| entry["code"] == "S005")
        .unwrap_or_else(|| panic!("expected S005 (missing radius): {diagnostics:?}"));
    assert_eq!(diagnostic["severity"], "error");
    assert!(
        diagnostic["line"].as_u64().unwrap() >= 1,
        "line must be 1-based: {diagnostic:?}"
    );
    assert!(diagnostic["column"].as_u64().unwrap() >= 1);
    assert!(
        !diagnostic["message"].as_str().unwrap().is_empty(),
        "message must be present"
    );
}

#[test]
fn default_text_renders_with_the_vendored_font_and_matches_the_cli_golden() {
    let root = repo_root();
    let source =
        std::fs::read_to_string(root.join("examples/text.rupa")).expect("read examples/text.rupa");
    let golden = std::fs::read(root.join("tests/golden/text.png")).expect("read text golden PNG");

    let frame = render_source(&source);
    assert!(frame.is_ok(), "metadata: {}", frame.meta);
    assert_eq!(
        frame.png, golden,
        "browser text rendering must use the same vendored font as the CLI"
    );
}

#[test]
fn rendering_is_deterministic() {
    let source = std::fs::read_to_string(repo_root().join("examples/procedural.rupa"))
        .expect("read examples/procedural.rupa");
    let first = render_source(&source);
    let second = render_source(&source);
    assert!(first.is_ok(), "metadata: {}", first.meta);
    assert_eq!(first, second);
}

#[test]
fn png_matches_the_cli_golden_image() {
    let root = repo_root();
    let source = std::fs::read_to_string(root.join("examples/minimal.rupa"))
        .expect("read examples/minimal.rupa");
    let golden = std::fs::read(root.join("tests/golden/minimal.png")).expect("read golden PNG");

    let frame = render_source(&source);
    assert!(frame.is_ok(), "metadata: {}", frame.meta);
    assert_eq!(
        frame.png, golden,
        "browser renderer output must match the CLI golden image byte for byte"
    );
}

#[test]
fn frame_wire_format_roundtrips_metadata_and_png() {
    let frame = render_source("size 16px x 16px\nbackground black\n");
    let bytes = encode_frame(&frame);
    let total = u32::from_le_bytes(bytes[0..4].try_into().unwrap()) as usize;
    let meta_len = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;

    assert_eq!(total, bytes.len());
    assert_eq!(
        std::str::from_utf8(&bytes[FRAME_HEADER_BYTES..FRAME_HEADER_BYTES + meta_len]).unwrap(),
        frame.meta
    );
    assert_eq!(
        &bytes[FRAME_HEADER_BYTES + meta_len..],
        frame.png.as_slice()
    );
}

#[test]
fn inline_and_indented_sources_render_identically() {
    let inline = "size 320px x 200px\nbackground white\n\nrectangle at (10px, 40px) size 80px x 40px fill red\ncircle center at (150px, 50px) radius 30px fill blue\n";
    let indented = "size 320px x 200px\nbackground white\n\nrectangle at (10px, 40px)\n    size 80px x 40px\n    fill red\n\ncircle center at (150px, 50px)\n    radius 30px\n    fill blue\n";

    let inline_frame = render_source(inline);
    let indented_frame = render_source(indented);

    assert!(inline_frame.is_ok(), "metadata: {}", inline_frame.meta);
    assert!(indented_frame.is_ok(), "metadata: {}", indented_frame.meta);
    assert!(
        !inline_frame.png.is_empty(),
        "inline form must render a PNG"
    );
    assert_eq!(
        inline_frame.png, indented_frame.png,
        "inline and indented forms must produce byte-identical PNGs"
    );
}

#[test]
fn picture_without_file_access_reports_resource_diagnostic() {
    let frame = render_source("size 40px x 40px\npicture \"photo.png\"\n    at (0px, 0px)\n");
    assert!(!frame.is_ok());

    let meta = meta(&frame);
    let diagnostics = meta["diagnostics"].as_array().expect("diagnostics array");
    assert!(
        diagnostics.iter().any(|entry| entry["code"] == "R001"),
        "expected R001 for a picture without resources: {diagnostics:?}"
    );
}
