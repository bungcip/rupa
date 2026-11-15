//! Paint IR for Rupa.
//!
//! Paint is separate from geometry (specification §17). Shapes reference a
//! paint through a [`PaintId`] into a [`PaintStore`]; geometry never embeds
//! renderer-specific paint data.
//!
//! Colors are stored as straight (non-premultiplied) RGBA with components in
//! the range `0.0..=1.0`. Compositing converts to premultiplied alpha at the
//! renderer boundary.

use serde::{Deserialize, Serialize};

/// Unique identifier for paint objects in a [`PaintStore`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PaintId(pub u64);

/// Straight-alpha RGBA color with components in `0.0..=1.0`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    /// Red component (0.0–1.0)
    pub r: f32,
    /// Green component (0.0–1.0)
    pub g: f32,
    /// Blue component (0.0–1.0)
    pub b: f32,
    /// Alpha component (0.0–1.0)
    pub a: f32,
}

impl Color {
    /// Create a straight-alpha color from normalized components.
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Create a color from 8-bit RGBA components.
    pub fn from_rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: f32::from(r) / 255.0,
            g: f32::from(g) / 255.0,
            b: f32::from(b) / 255.0,
            a: f32::from(a) / 255.0,
        }
    }

    /// Create an opaque color from 8-bit RGB components.
    pub fn from_rgb8(r: u8, g: u8, b: u8) -> Self {
        Self::from_rgba8(r, g, b, 255)
    }

    /// Convert back to 8-bit RGBA, clamping components to the valid range.
    pub fn to_rgba8(self) -> [u8; 4] {
        [to_u8(self.r), to_u8(self.g), to_u8(self.b), to_u8(self.a)]
    }

    /// Create white.
    pub const fn white() -> Self {
        Self::new(1.0, 1.0, 1.0, 1.0)
    }

    /// Create black.
    pub const fn black() -> Self {
        Self::new(0.0, 0.0, 0.0, 1.0)
    }

    /// Create a fully transparent color.
    pub const fn transparent() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }

    /// Convert to straight-alpha RGBA as `f32` with components in `0.0..=1.0`.
    pub fn to_straight(self) -> (f32, f32, f32, f32) {
        (self.r, self.g, self.b, self.a)
    }

    /// Convert an sRGB component to linear light.
    pub fn srgb_to_linear(value: f32) -> f32 {
        let value = value.clamp(0.0, 1.0);
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    }

    /// Convert a linear-light component to sRGB.
    pub fn linear_to_srgb(value: f32) -> f32 {
        let value = value.clamp(0.0, 1.0);
        if value <= 0.0031308 {
            value * 12.92
        } else {
            1.055 * value.powf(1.0 / 2.4) - 0.055
        }
    }

    /// Return a copy of this color with sRGB components converted to linear light.
    pub fn to_linear(self) -> Self {
        Self {
            r: Self::srgb_to_linear(self.r),
            g: Self::srgb_to_linear(self.g),
            b: Self::srgb_to_linear(self.b),
            a: self.a,
        }
    }

    /// Return a copy of this color with linear-light components converted to sRGB.
    pub fn from_linear(self) -> Self {
        Self {
            r: Self::linear_to_srgb(self.r),
            g: Self::linear_to_srgb(self.g),
            b: Self::linear_to_srgb(self.b),
            a: self.a,
        }
    }

    /// Convert to premultiplied-alpha RGBA as `f32`.
    pub fn to_premultiplied(self) -> (f32, f32, f32, f32) {
        let a = self.a.clamp(0.0, 1.0);
        if a == 0.0 {
            (0.0, 0.0, 0.0, 0.0)
        } else {
            (self.r * a, self.g * a, self.b * a, a)
        }
    }
}

fn to_u8(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

impl Default for Color {
    fn default() -> Self {
        Self::transparent()
    }
}

/// A stop in a linear gradient.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GradientStop {
    /// Normalized position in [0, 1].
    pub position: f32,
    /// Stop color.
    pub color: Color,
}

/// A linear gradient paint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinearGradient {
    /// Gradient stops.
    pub stops: Vec<GradientStop>,
    /// Angle in degrees.
    pub angle: f32,
}

/// A pattern fill.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatternPaint {
    /// Pattern kind.
    pub kind: String,
    /// Foreground color.
    pub color: Color,
    /// Background color.
    pub background: Color,
    /// Tile size in pixels.
    pub size: f32,
    /// Angle in degrees.
    pub angle: f32,
}

/// A gradient paint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Gradient {
    /// Linear gradient.
    Linear(LinearGradient),
    /// Radial gradient.
    Radial(RadialGradient),
}

/// A radial gradient paint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RadialGradient {
    /// Gradient stops.
    pub stops: Vec<GradientStop>,
}

/// Paint types in the IR.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Paint {
    /// Solid color paint.
    Solid(Color),
    /// Linear gradient paint.
    LinearGradient(LinearGradient),
    /// Radial gradient paint.
    RadialGradient(RadialGradient),
    /// Pattern fill paint.
    Pattern(PatternPaint),
    /// Generic gradient paint.
    Gradient(Gradient),
}

impl Paint {
    /// Convenience accessor for solid colors.
    pub fn solid(color: Color) -> Self {
        Self::Solid(color)
    }
}

/// Arena-backed paint store.
///
/// Shapes hold a [`PaintId`] and resolve it through this store. IDs are
/// stable for the lifetime of the store and are assigned in insertion order.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PaintStore {
    paints: Vec<Paint>,
}

impl PaintStore {
    /// Create an empty paint store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a paint and return its stable ID.
    pub fn add(&mut self, paint: Paint) -> PaintId {
        let id = PaintId(self.paints.len() as u64);
        self.paints.push(paint);
        id
    }

    /// Insert a solid color and return its stable ID.
    pub fn add_solid(&mut self, color: Color) -> PaintId {
        self.add(Paint::Solid(color))
    }

    /// Insert a linear gradient and return its stable ID.
    pub fn add_linear_gradient(&mut self, gradient: LinearGradient) -> PaintId {
        self.add(Paint::LinearGradient(gradient))
    }

    /// Insert a radial gradient and return its stable ID.
    pub fn add_radial_gradient(&mut self, gradient: RadialGradient) -> PaintId {
        self.add(Paint::RadialGradient(gradient))
    }

    /// Insert a pattern fill and return its stable ID.
    pub fn add_pattern(&mut self, pattern: PatternPaint) -> PaintId {
        self.add(Paint::Pattern(pattern))
    }

    /// Insert a generic gradient and return its stable ID.
    pub fn add_gradient(&mut self, gradient: Gradient) -> PaintId {
        self.add(Paint::Gradient(gradient))
    }

    /// Resolve a paint ID.
    pub fn get(&self, id: PaintId) -> Option<&Paint> {
        self.paints.get(id.0 as usize)
    }

    /// Number of paints in the store.
    pub fn len(&self) -> usize {
        self.paints.len()
    }

    /// Whether the store is empty.
    pub fn is_empty(&self) -> bool {
        self.paints.is_empty()
    }

    /// Iterate over paints in ID order.
    pub fn iter(&self) -> impl Iterator<Item = (PaintId, &Paint)> {
        self.paints
            .iter()
            .enumerate()
            .map(|(index, paint)| (PaintId(index as u64), paint))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_from_rgba8_roundtrip() {
        let color = Color::from_rgba8(255, 128, 0, 64);
        assert_eq!(color.to_rgba8(), [255, 128, 0, 64]);
    }

    #[test]
    fn color_named_helpers() {
        assert_eq!(Color::white().to_rgba8(), [255, 255, 255, 255]);
        assert_eq!(Color::black().to_rgba8(), [0, 0, 0, 255]);
        assert_eq!(Color::transparent().to_rgba8(), [0, 0, 0, 0]);
    }

    #[test]
    fn premultiplied_alpha() {
        let color = Color::from_rgba8(255, 0, 0, 128);
        let (r, g, b, a) = color.to_premultiplied();
        assert!((a - 128.0 / 255.0).abs() < 1e-6);
        assert!((r - a).abs() < 1e-6);
        assert_eq!((g, b), (0.0, 0.0));
    }

    #[test]
    fn color_srgb_linear_roundtrip() {
        let color = Color::from_rgb8(128, 64, 200);
        let linear = color.to_linear();
        let back = linear.from_linear();
        assert!((back.r - color.r).abs() < 1e-3);
        assert!((back.g - color.g).abs() < 1e-3);
        assert!((back.b - color.b).abs() < 1e-3);
    }

    #[test]
    fn transparent_premultiplies_to_zero() {
        assert_eq!(
            Color::transparent().to_premultiplied(),
            (0.0, 0.0, 0.0, 0.0)
        );
    }

    #[test]
    fn paint_store_assigns_stable_ids() {
        let mut store = PaintStore::new();
        assert!(store.is_empty());
        let red = store.add_solid(Color::from_rgb8(255, 0, 0));
        let blue = store.add_solid(Color::from_rgb8(0, 0, 255));
        assert_eq!(red, PaintId(0));
        assert_eq!(blue, PaintId(1));
        assert_eq!(store.len(), 2);
        assert_eq!(
            store.get(red),
            Some(&Paint::Solid(Color::from_rgb8(255, 0, 0)))
        );
        assert_eq!(
            store.get(blue),
            Some(&Paint::Solid(Color::from_rgb8(0, 0, 255)))
        );
        assert_eq!(store.get(PaintId(2)), None);
        assert_eq!(store.iter().count(), 2);
    }
}
