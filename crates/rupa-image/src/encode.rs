//! Image encoding for Rupa.
//!
//! PNG output uses straight (non-premultiplied) 8-bit RGBA in the sRGB color
//! space. The renderer produces premultiplied alpha for compositing; callers
//! must convert with [`unpremultiply_rgba`] before encoding a premultiplied
//! buffer.

use crate::decode::ImageBuffer;
use std::io::Write;
use std::path::Path;

/// Encodes a straight-alpha RGBA image buffer to PNG.
pub fn encode_png(buffer: &ImageBuffer, path: &Path) -> Result<(), EncodeError> {
    encode_rgba_png(&buffer.pixels, buffer.width, buffer.height, path)
}

/// Encode straight-alpha RGBA8 pixels to a PNG file at `path`.
///
/// The output has sRGB color space metadata and an 8-bit RGBA color type.
pub fn encode_rgba_png(
    pixels: &[u8],
    width: u32,
    height: u32,
    path: &Path,
) -> Result<(), EncodeError> {
    let file = std::fs::File::create(path)?;
    let writer = std::io::BufWriter::new(file);
    encode_rgba_png_to_writer(pixels, width, height, writer)
}

/// Encode straight-alpha RGBA8 pixels to an arbitrary writer.
pub fn encode_rgba_png_to_writer<W: Write>(
    pixels: &[u8],
    width: u32,
    height: u32,
    writer: W,
) -> Result<(), EncodeError> {
    validate_rgba_buffer(pixels, width, height)?;

    let mut encoder = png::Encoder::new(writer, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
    let mut writer = encoder
        .write_header()
        .map_err(|error| EncodeError::Encoding(error.to_string()))?;
    writer
        .write_image_data(pixels)
        .map_err(|error| EncodeError::Encoding(error.to_string()))?;
    Ok(())
}

/// Convert premultiplied RGBA8 pixels to straight alpha in place.
///
/// Fully transparent pixels become `(0, 0, 0, 0)`. Premultiplied channels are
/// clamped to the alpha value, so malformed input cannot overflow.
pub fn unpremultiply_rgba(pixels: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(pixels.len());
    for pixel in pixels.as_chunks::<4>().0 {
        let a = u16::from(pixel[3]);
        if a == 0 {
            output.extend_from_slice(&[0, 0, 0, 0]);
            continue;
        }
        let channel = |value: u8| -> u8 {
            let value = u16::from(value).min(a);
            (((u32::from(value) * 255) + u32::from(a) / 2) / u32::from(a)) as u8
        };
        output.push(channel(pixel[0]));
        output.push(channel(pixel[1]));
        output.push(channel(pixel[2]));
        output.push(pixel[3]);
    }
    output
}

/// Validate dimensions and buffer length for an RGBA8 image.
pub fn validate_rgba_buffer(pixels: &[u8], width: u32, height: u32) -> Result<(), EncodeError> {
    if width == 0 || height == 0 {
        return Err(EncodeError::InvalidDimensions { width, height });
    }
    let expected = (width as usize)
        .checked_mul(height as usize)
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or(EncodeError::InvalidDimensions { width, height })?;
    if pixels.len() != expected {
        return Err(EncodeError::InvalidBufferLength {
            expected,
            actual: pixels.len(),
        });
    }
    Ok(())
}

/// Encoding error
#[derive(Debug, thiserror::Error)]
pub enum EncodeError {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// Encoding error
    #[error("Encoding error: {0}")]
    Encoding(String),
    /// Image dimensions must be non-zero.
    #[error("invalid image dimensions {width}x{height}")]
    InvalidDimensions {
        /// Requested width.
        width: u32,
        /// Requested height.
        height: u32,
    },
    /// Pixel buffer length does not match dimensions.
    #[error("invalid RGBA buffer length: expected {expected} bytes, found {actual}")]
    InvalidBufferLength {
        /// Expected byte length.
        expected: usize,
        /// Actual byte length.
        actual: usize,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_FILE: AtomicU64 = AtomicU64::new(0);

    fn temp_png_path() -> std::path::PathBuf {
        let id = NEXT_FILE.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("rupa-image-test-{}-{id}.png", std::process::id()))
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

    #[test]
    fn encodes_rgba_png_with_correct_dimensions_and_pixels() {
        let pixels = vec![
            255, 0, 0, 255, // red
            0, 0, 255, 255, // blue
            255, 255, 255, 255, // white
            0, 0, 0, 0, // transparent
        ];
        let path = temp_png_path();
        encode_rgba_png(&pixels, 2, 2, &path).expect("encode PNG");

        assert!(path.is_file());
        let (width, height, decoded) = decode(&path);
        assert_eq!((width, height), (2, 2));
        assert_eq!(decoded, pixels);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn encodes_image_buffer() {
        let buffer = ImageBuffer {
            width: 1,
            height: 1,
            pixels: vec![12, 34, 56, 78],
        };
        let path = temp_png_path();
        encode_png(&buffer, &path).expect("encode PNG");
        let (width, height, decoded) = decode(&path);
        assert_eq!((width, height), (1, 1));
        assert_eq!(decoded, buffer.pixels);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn encoded_png_declares_srgb_color_space() {
        let pixels = vec![10, 20, 30, 255];
        let mut output = Vec::new();
        encode_rgba_png_to_writer(&pixels, 1, 1, Cursor::new(&mut output)).expect("encode PNG");
        let decoder = png::Decoder::new(Cursor::new(output));
        let reader = decoder.read_info().expect("read PNG info");
        assert_eq!(
            reader.info().srgb,
            Some(png::SrgbRenderingIntent::Perceptual)
        );
    }

    #[test]
    fn rejects_zero_dimensions() {
        assert!(matches!(
            encode_rgba_png(&[], 0, 10, &temp_png_path()),
            Err(EncodeError::InvalidDimensions { .. })
        ));
    }

    #[test]
    fn rejects_wrong_buffer_length() {
        assert!(matches!(
            encode_rgba_png(&[0; 7], 2, 1, &temp_png_path()),
            Err(EncodeError::InvalidBufferLength {
                expected: 8,
                actual: 7
            })
        ));
    }

    #[test]
    fn unpremultiply_recovers_straight_alpha() {
        // Premultiplied 50% red.
        assert_eq!(unpremultiply_rgba(&[128, 0, 0, 128]), vec![255, 0, 0, 128]);
        // Opaque colors are unchanged.
        assert_eq!(
            unpremultiply_rgba(&[255, 128, 0, 255]),
            vec![255, 128, 0, 255]
        );
        // Fully transparent pixels are normalized.
        assert_eq!(unpremultiply_rgba(&[5, 6, 7, 0]), vec![0, 0, 0, 0]);
    }

    #[test]
    fn unpremultiply_is_roundtrip_stable_for_valid_input() {
        let straight = [255u8, 0, 0, 128];
        let a = u16::from(straight[3]);
        let premultiplied = [((u16::from(straight[0]) * a + 127) / 255) as u8, 0, 0, 128];
        let recovered = unpremultiply_rgba(&premultiplied);
        for (actual, expected) in recovered.iter().zip(straight.iter()) {
            assert!(actual.abs_diff(*expected) <= 1);
        }
    }
}
