//! Image decoding for Rupa.
//!
//! Decoded images are canonicalized to 8-bit straight-alpha RGBA, row-major,
//! top-left origin. That buffer is independent of the input PNG color type, so
//! later pipeline stages (pictures, paints, raster operations) only handle one
//! pixel layout.
//!
//! Only PNG input is supported for now (specification §10). Format detection
//! uses the PNG signature rather than the file extension, so a mislabeled file
//! produces a format diagnostic instead of a misleading decode error.
//!
//! Decoding is deterministic: no color management or metadata is applied, and
//! 16-bit samples are reduced to their most significant byte.

use std::io::Cursor;
use std::path::{Path, PathBuf};

const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

/// Decodes an image file into a straight-alpha RGBA8 buffer.
///
/// The error includes the offending path so callers can report which resource
/// failed to load.
pub fn decode_image(path: &Path) -> Result<ImageBuffer, ImageError> {
    let bytes = std::fs::read(path).map_err(|source| ImageError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    decode_png(&bytes).map_err(|kind| ImageError::Decode {
        path: path.to_path_buf(),
        kind,
    })
}

/// Decodes PNG bytes into a straight-alpha RGBA8 buffer.
///
/// Palettes, grayscale, RGB, and alpha channels are all expanded to RGBA8.
pub fn decode_png(bytes: &[u8]) -> Result<ImageBuffer, ImageErrorKind> {
    if !bytes.starts_with(&PNG_SIGNATURE) {
        return Err(ImageErrorKind::UnsupportedFormat);
    }

    let mut decoder = png::Decoder::new(Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder
        .read_info()
        .map_err(|error| ImageErrorKind::Decoding(error.to_string()))?;
    let size = reader
        .output_buffer_size()
        .ok_or_else(|| ImageErrorKind::Decoding("image dimensions overflow".to_string()))?;
    let mut buffer = vec![0; size];
    let frame = reader
        .next_frame(&mut buffer)
        .map_err(|error| ImageErrorKind::Decoding(error.to_string()))?;
    buffer.truncate(frame.buffer_size());

    let (color_type, _) = reader.output_color_type();
    let pixels = to_rgba8(&buffer, color_type, frame.width, frame.height)?;
    ImageBuffer::new(frame.width, frame.height, pixels)
}

/// Expand normalized PNG samples to `width * height * 4` RGBA8 bytes.
fn to_rgba8(
    samples: &[u8],
    color_type: png::ColorType,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, ImageErrorKind> {
    let pixel_count = usize::try_from(u64::from(width) * u64::from(height))
        .map_err(|_| ImageErrorKind::InvalidDimensions { width, height })?;
    let channels = match color_type {
        png::ColorType::Grayscale => 1,
        png::ColorType::GrayscaleAlpha => 2,
        png::ColorType::Rgb => 3,
        png::ColorType::Rgba => 4,
        png::ColorType::Indexed => {
            return Err(ImageErrorKind::Decoding(
                "indexed PNG could not be expanded".to_string(),
            ));
        }
    };
    if samples.len() != pixel_count * channels {
        return Err(ImageErrorKind::Decoding(format!(
            "unexpected sample count {} for {}x{} {:?} image",
            samples.len(),
            width,
            height,
            color_type
        )));
    }

    let mut pixels = Vec::with_capacity(pixel_count * 4);
    for sample in samples.chunks_exact(channels) {
        match channels {
            1 => pixels.extend_from_slice(&[sample[0], sample[0], sample[0], 255]),
            2 => pixels.extend_from_slice(&[sample[0], sample[0], sample[0], sample[1]]),
            3 => pixels.extend_from_slice(&[sample[0], sample[1], sample[2], 255]),
            _ => pixels.extend_from_slice(sample),
        }
    }
    Ok(pixels)
}

/// Decoded image buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageBuffer {
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
    /// Straight-alpha RGBA8 pixel data (row-major)
    pub pixels: Vec<u8>,
}

impl ImageBuffer {
    /// Creates a buffer from straight-alpha RGBA8 pixels.
    ///
    /// Dimensions must be non-zero and `pixels` must contain exactly
    /// `width * height * 4` bytes.
    pub fn new(width: u32, height: u32, pixels: Vec<u8>) -> Result<Self, ImageErrorKind> {
        if width == 0 || height == 0 {
            return Err(ImageErrorKind::InvalidDimensions { width, height });
        }
        let expected = usize::try_from(u64::from(width) * u64::from(height))
            .ok()
            .and_then(|count| count.checked_mul(4))
            .ok_or(ImageErrorKind::InvalidDimensions { width, height })?;
        if pixels.len() != expected {
            return Err(ImageErrorKind::Decoding(format!(
                "expected {expected} RGBA bytes, found {}",
                pixels.len()
            )));
        }
        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    /// Returns the straight-alpha RGBA value at `(x, y)`, if in bounds.
    pub fn pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let offset = ((u64::from(y) * u64::from(self.width) + u64::from(x)) * 4) as usize;
        let pixel = &self.pixels[offset..offset + 4];
        Some([pixel[0], pixel[1], pixel[2], pixel[3]])
    }
}

/// Image loading error, including the path that failed.
#[derive(Debug, thiserror::Error)]
pub enum ImageError {
    /// The image file could not be read.
    #[error("cannot read image {}: {source}", path.display())]
    Read {
        /// Path that failed to read.
        path: PathBuf,
        /// Underlying IO error.
        #[source]
        source: std::io::Error,
    },
    /// The file was read but is not a decodable supported image.
    #[error("cannot load image {}: {kind}", path.display())]
    Decode {
        /// Path that failed to decode.
        path: PathBuf,
        /// Format or decoding failure.
        #[source]
        kind: ImageErrorKind,
    },
}

impl ImageError {
    /// Path of the image resource that failed.
    pub fn path(&self) -> &Path {
        match self {
            Self::Read { path, .. } | Self::Decode { path, .. } => path,
        }
    }

    /// Decoding failure kind, if the file was readable.
    pub fn kind(&self) -> Option<&ImageErrorKind> {
        match self {
            Self::Read { .. } => None,
            Self::Decode { kind, .. } => Some(kind),
        }
    }
}

/// Reason an image could not be decoded, independent of its path.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ImageErrorKind {
    /// The input is not a supported image format.
    #[error("unsupported image format; only PNG input is supported")]
    UnsupportedFormat,
    /// The PNG container or pixel data is invalid.
    #[error("invalid PNG data: {0}")]
    Decoding(String),
    /// Image dimensions are zero or overflow the buffer size.
    #[error("invalid image dimensions {width}x{height}")]
    InvalidDimensions {
        /// Image width.
        width: u32,
        /// Image height.
        height: u32,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_FILE: AtomicU64 = AtomicU64::new(0);

    fn temp_path(extension: &str) -> PathBuf {
        let id = NEXT_FILE.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "rupa-image-decode-{}-{id}.{extension}",
            std::process::id()
        ))
    }

    fn write_png(pixels: &[u8], width: u32, height: u32, color: png::ColorType) -> Vec<u8> {
        let mut output = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut output, width, height);
            encoder.set_color(color);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().expect("write header");
            writer.write_image_data(pixels).expect("write pixels");
        }
        output
    }

    #[test]
    fn decodes_rgba_png_bytes_preserving_alpha() {
        let rgba = [
            255, 0, 0, 255, // red
            0, 128, 0, 64, // half transparent green
            0, 0, 0, 0, // transparent
            10, 20, 30, 200,
        ];
        let png = write_png(&rgba, 2, 2, png::ColorType::Rgba);

        let image = decode_png(&png).expect("decode");

        assert_eq!((image.width, image.height), (2, 2));
        assert_eq!(image.pixels, rgba);
        assert_eq!(image.pixel(1, 0), Some([0, 128, 0, 64]));
    }

    #[test]
    fn expands_rgb_png_to_opaque_rgba() {
        let png = write_png(&[1, 2, 3, 4, 5, 6], 2, 1, png::ColorType::Rgb);

        let image = decode_png(&png).expect("decode");

        assert_eq!(image.pixels, vec![1, 2, 3, 255, 4, 5, 6, 255]);
    }

    #[test]
    fn expands_grayscale_png_to_rgba() {
        let png = write_png(&[90, 200], 2, 1, png::ColorType::Grayscale);

        let image = decode_png(&png).expect("decode");

        assert_eq!(image.pixels, vec![90, 90, 90, 255, 200, 200, 200, 255]);
    }

    #[test]
    fn expands_grayscale_alpha_png() {
        let png = write_png(&[40, 255, 80, 0], 2, 1, png::ColorType::GrayscaleAlpha);

        let image = decode_png(&png).expect("decode");

        assert_eq!(image.pixels, vec![40, 40, 40, 255, 80, 80, 80, 0]);
    }

    #[test]
    fn expands_indexed_png_to_rgba() {
        let mut output = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut output, 2, 1);
            encoder.set_color(png::ColorType::Indexed);
            encoder.set_depth(png::BitDepth::Eight);
            encoder.set_palette(vec![255, 0, 0, 0, 0, 255]);
            let mut writer = encoder.write_header().expect("write header");
            writer.write_image_data(&[0, 1]).expect("write pixels");
        }

        let image = decode_png(&output).expect("decode");

        assert_eq!(image.pixels, vec![255, 0, 0, 255, 0, 0, 255, 255]);
    }

    #[test]
    fn decodes_png_file_resolved_by_path() {
        let png = write_png(&[9, 8, 7, 255], 1, 1, png::ColorType::Rgba);
        let path = temp_path("png");
        std::fs::File::create(&path)
            .expect("create")
            .write_all(&png)
            .expect("write");

        let image = decode_image(&path).expect("decode");

        assert_eq!(image.pixel(0, 0), Some([9, 8, 7, 255]));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn rejects_non_png_data_with_unsupported_format() {
        let path = temp_path("jpg");
        std::fs::write(&path, b"\xFF\xD8\xFF\xE0not a png").expect("write");

        let error = decode_image(&path).expect_err("must fail");

        assert_eq!(error.path(), path.as_path());
        assert_eq!(error.kind(), Some(&ImageErrorKind::UnsupportedFormat));
        let message = error.to_string();
        assert!(
            message.contains("only PNG"),
            "unexpected message: {message}"
        );
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn rejects_truncated_png_with_decoding_error() {
        let png = write_png(&[1, 2, 3, 255], 1, 1, png::ColorType::Rgba);
        let truncated = &png[..png.len() / 2];

        let error = decode_png(truncated).expect_err("must fail");

        assert!(matches!(error, ImageErrorKind::Decoding(_)), "{error}");
    }

    #[test]
    fn missing_file_reports_read_error_with_path() {
        let path = temp_path("png");

        let error = decode_image(&path).expect_err("must fail");

        assert!(matches!(error, ImageError::Read { .. }), "{error}");
        assert_eq!(error.path(), path.as_path());
        assert!(error.kind().is_none());
    }

    #[test]
    fn rejects_zero_dimensions_and_wrong_buffer_lengths() {
        assert_eq!(
            ImageBuffer::new(0, 4, vec![]).expect_err("zero width"),
            ImageErrorKind::InvalidDimensions {
                width: 0,
                height: 4
            }
        );
        assert!(matches!(
            ImageBuffer::new(2, 2, vec![0; 15]),
            Err(ImageErrorKind::Decoding(_))
        ));
        assert!(ImageBuffer::new(2, 2, vec![0; 16]).is_ok());
    }

    #[test]
    fn pixel_lookup_is_bounds_checked() {
        let image = ImageBuffer::new(2, 2, (0..16).collect()).expect("valid buffer");
        assert_eq!(image.pixel(0, 1), Some([8, 9, 10, 11]));
        assert_eq!(image.pixel(2, 0), None);
        assert_eq!(image.pixel(0, 2), None);
    }
}
