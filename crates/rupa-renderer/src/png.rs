//! PNG encoding for Rupa.
//!
//! The renderer produces premultiplied alpha for compositing. PNG stores
//! straight (non-premultiplied) alpha, so this module unpremultiplies before
//! encoding. Encoded images use 8-bit RGBA and sRGB color space.

use crate::vector::RenderOutput;
use rupa_image::encode::{encode_rgba_png, unpremultiply_rgba, validate_rgba_buffer};
use std::path::Path;

/// Encodes a premultiplied RGBA pixel buffer to a PNG file.
///
/// `pixels` must contain exactly `width * height * 4` bytes. The buffer is
/// unpremultiplied before encoding so standard PNG viewers see correct colors
/// for semi-transparent pixels.
pub fn encode_png(pixels: &[u8], width: u32, height: u32, path: &Path) -> Result<(), PngError> {
    validate_rgba_buffer(pixels, width, height)?;
    let straight = unpremultiply_rgba(pixels);
    encode_rgba_png(&straight, width, height, path)?;
    Ok(())
}

/// Encode a renderer output buffer to a PNG file.
pub fn encode_render_output(output: &RenderOutput, path: &Path) -> Result<(), PngError> {
    encode_png(&output.pixels, output.width, output.height, path)
}

/// PNG encoding error
#[derive(Debug, thiserror::Error)]
pub enum PngError {
    /// Image encoding error (IO, dimensions, buffer length, or PNG failure).
    #[error(transparent)]
    Encode(#[from] rupa_image::EncodeError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vector::{RenderDocument, render_document};
    use rupa_geometry::Transform;
    use rupa_ir::geometry::{Circle as IrCircle, Geometry, Rectangle as IrRectangle, ShapeNode};
    use rupa_ir::paint::{Color, Paint, PaintId};
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_FILE: AtomicU64 = AtomicU64::new(0);

    fn temp_png_path() -> PathBuf {
        let id = NEXT_FILE.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "rupa-renderer-png-test-{}-{id}.png",
            std::process::id()
        ))
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

    fn shape(geometry: Geometry, fill: Option<PaintId>, opacity: f32) -> ShapeNode {
        ShapeNode {
            geometry,
            fill,
            stroke: None,
            transform: Transform::identity(),
            opacity,
        }
    }

    fn minimal_document() -> RenderDocument {
        let mut doc = RenderDocument::new(200, 100).with_background(Color::white());
        let red = doc.add_paint(Paint::Solid(Color::from_rgb8(255, 0, 0)));
        let blue = doc.add_paint(Paint::Solid(Color::from_rgb8(0, 0, 255)));
        doc.push_shape(shape(
            Geometry::Rectangle(IrRectangle {
                x: 10.0,
                y: 10.0,
                width: 80.0,
                height: 40.0,
            }),
            Some(red),
            1.0,
        ));
        doc.push_shape(shape(
            Geometry::Circle(IrCircle {
                cx: 150.0,
                cy: 50.0,
                radius: 30.0,
            }),
            Some(blue),
            1.0,
        ));
        doc
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

    fn assert_near(actual: [u8; 4], expected: [u8; 4], tolerance: u8) {
        for (index, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
            assert!(
                a.abs_diff(*e) <= tolerance,
                "channel {index}: expected {expected:?}, got {actual:?}"
            );
        }
    }

    #[test]
    fn minimal_document_writes_readable_png_with_expected_pixels() {
        let output = render_document(&minimal_document()).expect("render");
        let path = temp_png_path();
        encode_render_output(&output, &path).expect("encode PNG");

        assert!(path.is_file());
        let (width, height, pixels) = decode(&path);
        assert_eq!((width, height), (200, 100));
        assert_eq!(pixel_at(&pixels, width, 20, 20), [255, 0, 0, 255]);
        assert_eq!(pixel_at(&pixels, width, 150, 50), [0, 0, 255, 255]);
        assert_eq!(pixel_at(&pixels, width, 190, 90), [255, 255, 255, 255]);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn transparent_background_keeps_alpha_zero() {
        let mut doc = RenderDocument::new(20, 20);
        let red = doc.add_paint(Paint::Solid(Color::from_rgb8(255, 0, 0)));
        doc.push_shape(shape(
            Geometry::Rectangle(IrRectangle {
                x: 5.0,
                y: 5.0,
                width: 10.0,
                height: 10.0,
            }),
            Some(red),
            1.0,
        ));

        let output = render_document(&doc).expect("render");
        let path = temp_png_path();
        encode_render_output(&output, &path).expect("encode PNG");
        let (width, height, pixels) = decode(&path);
        assert_eq!((width, height), (20, 20));
        assert_eq!(pixel_at(&pixels, width, 10, 10), [255, 0, 0, 255]);
        assert_eq!(pixel_at(&pixels, width, 0, 0), [0, 0, 0, 0]);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn semi_transparent_pixels_are_unpremultiplied_for_png() {
        let mut doc = RenderDocument::new(10, 10);
        let half_red = doc.add_paint(Paint::Solid(Color::new(1.0, 0.0, 0.0, 0.5)));
        doc.push_shape(shape(
            Geometry::Rectangle(IrRectangle {
                x: 0.0,
                y: 0.0,
                width: 10.0,
                height: 10.0,
            }),
            Some(half_red),
            1.0,
        ));

        let output = render_document(&doc).expect("render");
        // Renderer output is premultiplied.
        assert_near(output.pixel(5, 5).unwrap(), [128, 0, 0, 128], 1);

        let path = temp_png_path();
        encode_render_output(&output, &path).expect("encode PNG");
        let (width, _height, pixels) = decode(&path);
        // PNG stores straight alpha.
        assert_near(pixel_at(&pixels, width, 5, 5), [255, 0, 0, 128], 1);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn rejects_wrong_buffer_length_before_writing() {
        let path = temp_png_path();
        let error = encode_png(&[0; 7], 2, 1, &path).expect_err("invalid buffer");
        assert!(matches!(
            error,
            PngError::Encode(rupa_image::EncodeError::InvalidBufferLength {
                expected: 8,
                actual: 7
            })
        ));
        assert!(!path.exists());
    }

    #[test]
    fn rejects_zero_dimensions() {
        let path = temp_png_path();
        let error = encode_png(&[], 0, 10, &path).expect_err("invalid dimensions");
        assert!(matches!(
            error,
            PngError::Encode(rupa_image::EncodeError::InvalidDimensions {
                width: 0,
                height: 10
            })
        ));
        assert!(!path.exists());
    }
}
