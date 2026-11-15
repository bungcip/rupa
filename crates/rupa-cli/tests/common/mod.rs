#![allow(dead_code)]
//! Shared utilities for CLI integration and golden-image tests.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

/// Workspace root directory.
pub fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Path to the minimal example source.
pub fn minimal_example() -> PathBuf {
    workspace_root().join("examples/minimal.rupa")
}

/// Path to a golden PNG under `tests/golden/`.
pub fn golden_path(name: &str) -> PathBuf {
    workspace_root().join("tests/golden").join(name)
}

/// Create a unique temporary directory for a test.
pub fn temp_dir() -> PathBuf {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let dir =
        std::env::temp_dir().join(format!("rupa-integration-test-{}-{id}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

/// Run the built `rupa` CLI binary with the given arguments.
pub fn run_rupa(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rupa"))
        .args(args)
        .output()
        .expect("run rupa CLI")
}

/// Decoded PNG image with straight-alpha RGBA8 pixels.
pub struct DecodedPng {
    /// Image width.
    pub width: u32,
    /// Image height.
    pub height: u32,
    /// Straight-alpha RGBA8 pixels, row-major.
    pub pixels: Vec<u8>,
}

impl DecodedPng {
    /// Read one pixel as RGBA.
    pub fn pixel(&self, x: u32, y: u32) -> [u8; 4] {
        assert!(x < self.width && y < self.height, "pixel out of bounds");
        let index = ((y * self.width + x) * 4) as usize;
        [
            self.pixels[index],
            self.pixels[index + 1],
            self.pixels[index + 2],
            self.pixels[index + 3],
        ]
    }
}

/// Decode a PNG file into straight-alpha RGBA8 pixels.
pub fn decode_png(path: &Path) -> DecodedPng {
    let file = std::fs::File::open(path)
        .unwrap_or_else(|error| panic!("open PNG {}: {error}", path.display()));
    let decoder = png::Decoder::new(std::io::BufReader::new(file));
    let mut reader = decoder.read_info().expect("read PNG info");
    let mut buffer = vec![0; reader.output_buffer_size().expect("known output size")];
    let info = reader.next_frame(&mut buffer).expect("decode PNG");
    buffer.truncate(info.buffer_size());
    DecodedPng {
        width: info.width,
        height: info.height,
        pixels: buffer,
    }
}

/// Assert that two decoded PNGs match exactly (dimensions and every pixel).
pub fn assert_images_equal(actual: &DecodedPng, expected: &DecodedPng, context: &str) {
    assert_eq!(
        (actual.width, actual.height),
        (expected.width, expected.height),
        "{context}: dimensions differ"
    );
    assert_eq!(
        actual.pixels.len(),
        expected.pixels.len(),
        "{context}: pixel buffer lengths differ"
    );
    if actual.pixels != expected.pixels {
        for (index, (a, e)) in actual.pixels.iter().zip(expected.pixels.iter()).enumerate() {
            if a != e {
                let pixel_index = index / 4;
                let x = pixel_index as u32 % actual.width;
                let y = pixel_index as u32 / actual.width;
                panic!(
                    "{context}: first pixel mismatch at ({x}, {y}), channel {}, actual {}, expected {}",
                    index % 4,
                    a,
                    e
                );
            }
        }
        panic!("{context}: pixel buffers differ");
    }
}

/// Render a source file and compare the result with a golden PNG.
///
/// This is the reusable entry point for adding more golden tests: pass the
/// `.rupa` fixture and the golden file name under `tests/golden/`.
pub fn assert_render_matches_golden(source: &Path, golden_name: &str) {
    let dir = temp_dir();
    let output_path = dir.join("rendered.png");
    let output = run_rupa(&[
        "render",
        source.to_str().expect("utf-8 path"),
        "--output",
        output_path.to_str().expect("utf-8 path"),
    ]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "render failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = decode_png(&output_path);
    let expected = decode_png(&golden_path(golden_name));
    assert_images_equal(&actual, &expected, golden_name);

    let _ = std::fs::remove_dir_all(dir);
}

/// Render `examples/minimal.rupa` and compare it with a golden PNG.
pub fn assert_example_matches_golden(golden_name: &str) {
    assert_render_matches_golden(&minimal_example(), golden_name);
}
