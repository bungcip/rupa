//! Transform utilities for Rupa.
//!
//! Transforms use a 3x2 affine matrix in column-vector form:
//!
//! ```text
//! | a c e |   | x |   | a*x + c*y + e |
//! | b d f | * | y | = | b*x + d*y + f |
//! | 0 0 1 |   | 1 |   |       1        |
//! ```
//!
//! [`Transform::multiply`] computes `self * other`, so the result applies
//! `other` first and then `self`. Rotation angles are in radians and increase
//! clockwise in Rupa's Y-down coordinate system (a positive angle maps +X
//! toward +Y).

use serde::{Deserialize, Serialize};

/// 2D transform matrix (3x2 for affine transforms)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    /// Scale X / rotation component
    pub a: f32,
    /// Shear Y
    pub b: f32,
    /// Shear X
    pub c: f32,
    /// Scale Y / rotation component
    pub d: f32,
    /// Translate X
    pub e: f32,
    /// Translate Y
    pub f: f32,
}

impl Transform {
    /// Create an identity transform
    pub fn identity() -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 0.0,
            f: 0.0,
        }
    }

    /// Create a translation transform
    pub fn translate(tx: f32, ty: f32) -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: tx,
            f: ty,
        }
    }

    /// Create a scale transform
    pub fn scale(sx: f32, sy: f32) -> Self {
        Self {
            a: sx,
            b: 0.0,
            c: 0.0,
            d: sy,
            e: 0.0,
            f: 0.0,
        }
    }

    /// Create a rotation transform (angle in radians).
    ///
    /// In Rupa's Y-down coordinate system this rotates clockwise on screen:
    /// a positive quarter turn maps `(1, 0)` to `(0, 1)`.
    pub fn rotate(angle_rad: f32) -> Self {
        let cos = angle_rad.cos();
        let sin = angle_rad.sin();
        Self {
            a: cos,
            b: sin,
            c: -sin,
            d: cos,
            e: 0.0,
            f: 0.0,
        }
    }

    /// Rotate about a pivot point.
    pub fn rotate_about(angle_rad: f32, px: f32, py: f32) -> Self {
        Self::about_pivot(Transform::rotate(angle_rad), px, py)
    }

    /// Scale about a pivot point.
    pub fn scale_about(sx: f32, sy: f32, px: f32, py: f32) -> Self {
        Self::about_pivot(Transform::scale(sx, sy), px, py)
    }

    /// Apply `transform` around `(px, py)`.
    fn about_pivot(transform: Transform, px: f32, py: f32) -> Self {
        Transform::translate(px, py)
            .multiply(&transform)
            .multiply(&Transform::translate(-px, -py))
    }

    /// Transform a point.
    pub fn transform_point(&self, x: f32, y: f32) -> (f32, f32) {
        (
            self.a * x + self.c * y + self.e,
            self.b * x + self.d * y + self.f,
        )
    }

    /// Multiply this transform by another (`self * other`).
    ///
    /// The returned transform applies `other` first, then `self`.
    pub fn multiply(&self, other: &Transform) -> Transform {
        Transform {
            a: self.a * other.a + self.c * other.b,
            b: self.b * other.a + self.d * other.b,
            c: self.a * other.c + self.c * other.d,
            d: self.b * other.c + self.d * other.d,
            e: self.a * other.e + self.c * other.f + self.e,
            f: self.b * other.e + self.d * other.f + self.f,
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

/// Transform builder for convenient construction
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransformBuilder {
    transform: Transform,
}

impl TransformBuilder {
    /// Create a new transform builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Add translation
    pub fn translate(mut self, tx: f32, ty: f32) -> Self {
        self.transform = self.transform.multiply(&Transform::translate(tx, ty));
        self
    }

    /// Add scaling
    pub fn scale(mut self, sx: f32, sy: f32) -> Self {
        self.transform = self.transform.multiply(&Transform::scale(sx, sy));
        self
    }

    /// Add rotation
    pub fn rotate(mut self, angle_rad: f32) -> Self {
        self.transform = self.transform.multiply(&Transform::rotate(angle_rad));
        self
    }

    /// Build the final transform
    pub fn build(self) -> Transform {
        self.transform
    }
}

impl From<TransformBuilder> for Transform {
    fn from(builder: TransformBuilder) -> Self {
        builder.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::{FRAC_PI_2, PI};

    fn assert_point(transform: &Transform, x: f32, y: f32, ex: f32, ey: f32) {
        let (actual_x, actual_y) = transform.transform_point(x, y);
        assert!(
            (actual_x - ex).abs() < 1e-5 && (actual_y - ey).abs() < 1e-5,
            "expected ({ex}, {ey}), got ({actual_x}, {actual_y})"
        );
    }

    #[test]
    fn identity_leaves_points_unchanged() {
        assert_point(&Transform::identity(), 3.0, 4.0, 3.0, 4.0);
    }

    #[test]
    fn translate_shifts_points() {
        let transform = Transform::translate(10.0, -5.0);
        assert_point(&transform, 3.0, 4.0, 13.0, -1.0);
    }

    #[test]
    fn scale_multiplies_coordinates() {
        let transform = Transform::scale(2.0, 3.0);
        assert_point(&transform, 4.0, 5.0, 8.0, 15.0);
    }

    #[test]
    fn rotate_90_degrees_is_clockwise_on_screen() {
        assert_point(&Transform::rotate(FRAC_PI_2), 1.0, 0.0, 0.0, 1.0);
        assert_point(&Transform::rotate(FRAC_PI_2), 0.0, 1.0, -1.0, 0.0);
        assert_point(&Transform::rotate(PI), 1.0, 0.0, -1.0, 0.0);
    }

    #[test]
    fn multiply_applies_other_then_self() {
        // Scale first, then translate: (2, 2) -> (4, 4) -> (14, 4).
        let transform = Transform::translate(10.0, 0.0).multiply(&Transform::scale(2.0, 2.0));
        assert_point(&transform, 2.0, 2.0, 14.0, 4.0);
    }

    #[test]
    fn nested_transforms_compose() {
        let transform = Transform::translate(5.0, 5.0)
            .multiply(&Transform::rotate(FRAC_PI_2))
            .multiply(&Transform::scale(2.0, 2.0));
        // (1, 0) -> scale (2, 0) -> rotate (0, 2) -> translate (5, 7).
        assert_point(&transform, 1.0, 0.0, 5.0, 7.0);
    }

    #[test]
    fn rotate_about_pivot_keeps_pivot_fixed() {
        let transform = Transform::rotate_about(FRAC_PI_2, 10.0, 10.0);
        assert_point(&transform, 10.0, 10.0, 10.0, 10.0);
        assert_point(&transform, 20.0, 10.0, 10.0, 20.0);
    }

    #[test]
    fn scale_about_pivot_keeps_pivot_fixed() {
        let transform = Transform::scale_about(2.0, 2.0, 10.0, 10.0);
        assert_point(&transform, 10.0, 10.0, 10.0, 10.0);
        assert_point(&transform, 20.0, 10.0, 30.0, 10.0);
    }

    #[test]
    fn builder_composes_in_call_order_as_nested_transforms() {
        // Builder multiplies each new transform on the right, so earlier calls
        // are applied later (outermost).
        let transform = TransformBuilder::new()
            .translate(10.0, 0.0)
            .scale(2.0, 2.0)
            .build();
        assert_point(&transform, 1.0, 0.0, 12.0, 0.0);
    }
}
