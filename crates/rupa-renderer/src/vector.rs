//! Vector rendering for Rupa.
//!
//! Converts geometry IR into `tiny-skia` paths, applies solid paint, opacity,
//! and affine transforms, then rasterizes into a premultiplied RGBA pixel
//! buffer (specification §18.1).
//!
//! Transforms use the primitive's geometric center as pivot:
//! - rectangle: center of its bounding box;
//! - circle: its center.
//!
//! Layer content is clipped to the definition's declared bounds
//! (specification §6.4). Clipping is always enabled: each instance builds a
//! device-space mask from its transformed local bounds, and nested instances
//! intersect that mask with their parent's.

use crate::fonts::resolve_font_bytes;
use rupa_geometry::Transform;
use rupa_image::{ImageBuffer, ResourceStore};
use rupa_ir::document::{
    CompositionChild, Filter, FitMode, PictureNode, TextNode, stable_z_index_order,
};
use rupa_ir::geometry::{Geometry, ShapeNode};
use rupa_ir::paint::{Color, Gradient, Paint, PaintId, PaintStore};
use tiny_skia::{
    Color as SkiaColor, FillRule, Mask, Paint as SkiaPaint, Path, PathBuilder, Pixmap, PixmapPaint,
    Transform as SkiaTransform,
};

/// A rasterized image in premultiplied RGBA8 (row-major).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderOutput {
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
    /// Premultiplied RGBA8 pixels, row-major, `width * height * 4` bytes.
    pub pixels: Vec<u8>,
}

impl RenderOutput {
    /// Read one pixel as premultiplied RGBA.
    ///
    /// Returns `None` for out-of-bounds coordinates.
    pub fn pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let index = ((y * self.width + x) * 4) as usize;
        Some([
            self.pixels[index],
            self.pixels[index + 1],
            self.pixels[index + 2],
            self.pixels[index + 3],
        ])
    }
}

/// An IR document ready for rendering.
///
/// This is the backend-independent IR document from `rupa-ir`.
pub use rupa_ir::document::GraphicsDocument as RenderDocument;

/// Rendering failure.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum RenderError {
    /// Canvas dimensions must be non-zero.
    #[error("canvas dimensions must be non-zero")]
    InvalidDimensions,
    /// Geometry is degenerate or not finite.
    #[error("invalid {shape} geometry: {detail}")]
    InvalidGeometry {
        /// Shape kind (`rectangle`/`circle`).
        shape: &'static str,
        /// Human-readable reason.
        detail: String,
    },
    /// A shape references a paint that does not exist in the paint store.
    #[error("shape references missing paint {0:?}")]
    MissingPaint(PaintId),
    /// A layer plane references a missing definition.
    #[error("layer plane references missing layer `{0}`")]
    MissingLayer(String),
    /// A placement references a missing group definition.
    #[error("placement references missing group `{0}`")]
    MissingGroup(String),
    /// A group recursively references one of its own ancestors.
    #[error("cyclic group reference through `{0}`")]
    CyclicGroup(String),
    /// A picture was rendered without a resource store.
    #[error("picture `{0}` requires a resource store; use render_document_with_resources")]
    MissingResource(String),
    /// A text font could not be resolved.
    #[error("cannot resolve font `{0}`")]
    MissingFont(String),
    /// A picture resource could not be decoded.
    #[error("cannot load picture resource `{path}`: {detail}")]
    Resource {
        /// Resource path as written in the source.
        path: String,
        /// Decoding failure description.
        detail: String,
    },
    /// A picture frame or image is degenerate.
    #[error("invalid picture frame or image: {0}")]
    InvalidPicture(String),
    /// A clip mask could not be allocated.
    #[error("failed to allocate clip mask")]
    ClipMask,
    /// A pattern fill tile could not be allocated.
    #[error("failed to allocate pattern fill tile")]
    PatternTile,
    /// tiny-skia could not finish the path.
    #[error("failed to build path")]
    PathBuild,
}

/// CPU vector renderer backed by a `tiny-skia` pixmap.
pub struct VectorRenderer {
    width: u32,
    height: u32,
    pixmap: Pixmap,
    font_cache: std::collections::HashMap<String, Vec<u8>>,
}

impl VectorRenderer {
    /// Create a transparent renderer for the given dimensions.
    pub fn new(width: u32, height: u32) -> Result<Self, RenderError> {
        if width == 0 || height == 0 {
            return Err(RenderError::InvalidDimensions);
        }
        let pixmap = Pixmap::new(width, height).ok_or(RenderError::InvalidDimensions)?;
        Ok(Self {
            width,
            height,
            pixmap,
            font_cache: std::collections::HashMap::new(),
        })
    }

    /// Canvas width.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Canvas height.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Borrow the premultiplied RGBA8 pixel buffer.
    pub fn data(&self) -> &[u8] {
        self.pixmap.data()
    }

    /// Consume the renderer and return the pixel buffer.
    pub fn into_pixels(self) -> Vec<u8> {
        self.pixmap.take()
    }

    /// Consume the renderer and return its backing pixmap.
    pub fn take_pixmap(self) -> Pixmap {
        self.pixmap
    }

    /// Fill the whole canvas with an opaque or semi-transparent color.
    pub fn clear(&mut self, color: Color) {
        self.pixmap.fill(to_skia_color(color));
    }

    /// Render IR shapes in order using paints from `paints`.
    ///
    /// The canvas is not cleared; use [`Self::clear`] first if needed.
    pub fn render_shapes(
        &mut self,
        shapes: &[ShapeNode],
        paints: &PaintStore,
    ) -> Result<(), RenderError> {
        for shape in shapes {
            self.render_shape_with_parent(shape, paints, Transform::identity(), 1.0, None)?;
        }
        Ok(())
    }

    /// Render one composition's children in stable z-index order.
    ///
    /// Nested instances compose their local-to-parent transform with the
    /// enclosing composition's transform and multiply opacity down the
    /// hierarchy. `active_layers` holds the definition names on the current
    /// ancestry so a cyclic reference is reported instead of recursing without
    /// bound; static cycle rejection is TASK-024.
    ///
    /// `clip` is the enclosing layer's device-space clip mask. Each instance
    /// clips its content to its own transformed bounds and intersects that
    /// region with `clip` (specification §6.4).
    #[allow(clippy::too_many_arguments)]
    fn render_composition(
        &mut self,
        children: &[CompositionChild],
        document: &RenderDocument,
        parent: Transform,
        opacity_scale: f32,
        active_layers: &mut Vec<String>,
        clip: Option<&Mask>,
        mut resources: Option<&mut ResourceStore>,
    ) -> Result<(), RenderError> {
        for child in stable_z_index_order(children) {
            match child {
                CompositionChild::Shape(shape) => {
                    self.render_shape_with_parent(
                        shape,
                        &document.paints,
                        parent,
                        opacity_scale,
                        clip,
                    )?;
                }
                CompositionChild::Picture(picture) => {
                    let Some(resources) = resources.as_deref_mut() else {
                        return Err(RenderError::MissingResource(picture.path.clone()));
                    };
                    self.render_picture(picture, parent, opacity_scale, clip, resources)?;
                }
                CompositionChild::Text(text) => {
                    self.render_text(text, parent, opacity_scale, clip)?;
                }
                CompositionChild::Clear(color) => {
                    self.clear(*color);
                }
                CompositionChild::Pixel(pixel) => {
                    let x = pixel.x as i32;
                    let y = pixel.y as i32;
                    if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
                        continue;
                    }
                    let (pr, pg, pb, pa) = pixel.color.to_premultiplied();
                    let idx = ((y as u32 * self.width + x as u32) * 4) as usize;
                    let data = self.pixmap.data_mut();
                    data[idx] = (pr * 255.0).round() as u8;
                    data[idx + 1] = (pg * 255.0).round() as u8;
                    data[idx + 2] = (pb * 255.0).round() as u8;
                    data[idx + 3] = (pa * 255.0).round() as u8;
                }
                CompositionChild::Brush(brush) => {
                    if brush.points.is_empty() {
                        continue;
                    }
                    let mut builder = PathBuilder::new();
                    let first = &brush.points[0];
                    builder.move_to(first.x, first.y);
                    for point in &brush.points[1..] {
                        builder.line_to(point.x, point.y);
                    }
                    let Some(path) = builder.finish() else {
                        continue;
                    };
                    let opacity = opacity_scale.clamp(0.0, 1.0);
                    if opacity <= 0.0 {
                        continue;
                    }
                    let color = with_opacity(brush.color, opacity);
                    let mut paint = SkiaPaint {
                        anti_alias: true,
                        ..SkiaPaint::default()
                    };
                    paint.set_color(to_skia_color(color));
                    let stroke = tiny_skia::Stroke {
                        width: brush.size,
                        line_cap: tiny_skia::LineCap::Round,
                        line_join: tiny_skia::LineJoin::Round,
                        ..tiny_skia::Stroke::default()
                    };
                    self.pixmap.stroke_path(
                        &path,
                        &paint,
                        &stroke,
                        to_skia_transform(&parent),
                        clip,
                    );
                }
                CompositionChild::Instance(instance) => {
                    let group = document
                        .groups
                        .iter()
                        .find(|group| group.name == instance.group)
                        .ok_or_else(|| RenderError::MissingGroup(instance.group.clone()))?;
                    if active_layers.contains(&instance.group) {
                        return Err(RenderError::CyclicGroup(instance.group.clone()));
                    }
                    let base = instance.local_to_parent_transform(group);
                    self.render_composite(
                        &instance.group,
                        &group.children,
                        group.bounds,
                        group.clip,
                        parent,
                        base,
                        opacity_scale,
                        instance.opacity,
                        instance.blend_mode,
                        false,
                        &[],
                        document,
                        active_layers,
                        clip,
                        resources.as_deref_mut(),
                    )?;
                }
                CompositionChild::Plane(plane) => {
                    let layer = document
                        .layers
                        .iter()
                        .find(|layer| layer.name == plane.layer)
                        .ok_or_else(|| RenderError::MissingLayer(plane.layer.clone()))?;
                    if active_layers.contains(&plane.layer) {
                        return Err(RenderError::CyclicGroup(plane.layer.clone()));
                    }
                    let bounds = rupa_ir::document::Bounds {
                        x: 0.0,
                        y: 0.0,
                        width: layer.width as f32,
                        height: layer.height as f32,
                    };
                    self.render_composite(
                        &plane.layer,
                        &layer.children,
                        bounds,
                        layer.clip,
                        parent,
                        Transform::identity(),
                        opacity_scale,
                        plane.opacity,
                        plane.blend_mode,
                        matches!(layer.mode, rupa_ir::document::LayerMode::Raster),
                        &layer.effects,
                        document,
                        active_layers,
                        clip,
                        resources.as_deref_mut(),
                    )?;
                }
            }
        }
        Ok(())
    }

    /// Render a group instance or a layer plane into the parent composition.
    ///
    /// `bounds` is the composite's region in its own local coordinates. It is
    /// clipped only when `clip_to_bounds` is set (declared group size, or a layer
    /// plane). `base` maps local `(0, 0)` into the parent space. Composite
    /// effects that cannot be expressed by drawing children directly (raster
    /// mode, blend modes, filters) render into an offscreen surface first.
    #[allow(clippy::too_many_arguments)]
    fn render_composite(
        &mut self,
        name: &str,
        children: &[CompositionChild],
        bounds: rupa_ir::document::Bounds,
        clip_to_bounds: bool,
        parent: Transform,
        base: Transform,
        opacity_scale: f32,
        opacity: f32,
        blend_mode: rupa_ir::BlendMode,
        raster: bool,
        effects: &[rupa_ir::Filter],
        document: &RenderDocument,
        active_layers: &mut Vec<String>,
        clip: Option<&Mask>,
        mut resources: Option<&mut ResourceStore>,
    ) -> Result<(), RenderError> {
        let transform = parent.multiply(&base);
        let bounds_clip = if clip_to_bounds {
            Some(self.rect_clip_mask(
                bounds.x,
                bounds.y,
                bounds.width,
                bounds.height,
                &transform,
                clip,
            )?)
        } else {
            None
        };
        let effective_clip = bounds_clip.as_ref().or(clip);
        let use_offscreen =
            raster || blend_mode != rupa_ir::BlendMode::Normal || !effects.is_empty();

        active_layers.push(name.to_string());
        if use_offscreen {
            // The offscreen surface spans `bounds`; children are drawn with the
            // bounds origin at the surface origin, and the surface is composited
            // back at the bounds origin so placement stays identical.
            let width = bounds.width.ceil().max(1.0) as u32;
            let height = bounds.height.ceil().max(1.0) as u32;
            let composite_opacity = (opacity_scale * opacity).clamp(0.0, 1.0);
            if composite_opacity > 0.0 {
                let mut surface = VectorRenderer::new(width, height)?;
                surface.render_composition(
                    children,
                    document,
                    Transform::translate(-bounds.x, -bounds.y),
                    1.0,
                    active_layers,
                    None,
                    resources.as_deref_mut(),
                )?;
                let mut surface_pixmap = surface.take_pixmap();
                apply_filters(&mut surface_pixmap, effects);
                let paint = PixmapPaint {
                    opacity: composite_opacity,
                    blend_mode: to_skia_blend_mode(blend_mode),
                    ..PixmapPaint::default()
                };
                let draw_transform = transform.multiply(&Transform::translate(bounds.x, bounds.y));
                self.pixmap.draw_pixmap(
                    0,
                    0,
                    surface_pixmap.as_ref(),
                    &paint,
                    to_skia_transform(&draw_transform),
                    effective_clip,
                );
            }
        } else {
            self.render_composition(
                children,
                document,
                transform,
                opacity_scale * opacity,
                active_layers,
                effective_clip,
                resources,
            )?;
        }
        active_layers.pop();
        Ok(())
    }

    /// Build a device-space clip mask for a rectangle in some parent space.
    ///
    /// The mask covers `(x, y)` to `(x + width, y + height)` transformed by
    /// `transform`. When `parent_clip` is present, the result is intersected
    /// with it so nested content stays inside every enclosing clip region.
    /// Edges are antialiased.
    fn rect_clip_mask(
        &self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        transform: &Transform,
        parent_clip: Option<&Mask>,
    ) -> Result<Mask, RenderError> {
        let mut mask = Mask::new(self.width, self.height).ok_or(RenderError::ClipMask)?;
        let bounds =
            tiny_skia::Rect::from_xywh(x, y, width, height).ok_or(RenderError::ClipMask)?;
        let mut builder = PathBuilder::new();
        builder.push_rect(bounds);
        let path = builder.finish().ok_or(RenderError::PathBuild)?;
        mask.fill_path(&path, FillRule::Winding, true, to_skia_transform(transform));

        if let Some(parent) = parent_clip {
            for (value, parent_value) in mask.data_mut().iter_mut().zip(parent.data()) {
                *value = (u16::from(*value) * u16::from(*parent_value) / 255) as u8;
            }
        }

        Ok(mask)
    }

    fn render_shape_with_parent(
        &mut self,
        shape: &ShapeNode,
        paints: &PaintStore,
        parent: Transform,
        opacity_scale: f32,
        clip: Option<&Mask>,
    ) -> Result<(), RenderError> {
        if shape.fill.is_none() && shape.stroke.is_none() {
            return Ok(());
        }
        let opacity = (shape.opacity * opacity_scale).clamp(0.0, 1.0);
        if opacity <= 0.0 {
            return Ok(());
        }

        let path = geometry_path(&shape.geometry)?;
        let origin = geometry_center(&shape.geometry);
        let local = transform_about_origin(&shape.transform, origin);
        let transform = to_skia_transform(&parent.multiply(&local));

        if let Some(fill_id) = shape.fill {
            let paint = paints
                .get(fill_id)
                .ok_or(RenderError::MissingPaint(fill_id))?;
            match paint {
                Paint::Solid(color) => {
                    let mut fill = SkiaPaint {
                        anti_alias: true,
                        ..SkiaPaint::default()
                    };
                    fill.set_color(to_skia_color(with_opacity(*color, opacity)));
                    self.pixmap
                        .fill_path(&path, &fill, FillRule::Winding, transform, clip);
                }
                Paint::LinearGradient(gradient) | Paint::Gradient(Gradient::Linear(gradient)) => {
                    let gradient = gradient.clone();
                    let gradient = with_gradient_opacity(&gradient, opacity);
                    let (min_x, min_y, max_x, max_y) = geometry_bounds(&shape.geometry);
                    let cx = (min_x + max_x) / 2.0;
                    let cy = (min_y + max_y) / 2.0;
                    let angle = gradient.angle.to_radians();
                    let half_len = ((max_x - min_x).hypot(max_y - min_y)) / 2.0;
                    let dx = angle.cos() * half_len;
                    let dy = angle.sin() * half_len;
                    let (sx, sy) = transform_point(cx - dx, cy - dy, &transform);
                    let (ex, ey) = transform_point(cx + dx, cy + dy, &transform);
                    let stops: Vec<tiny_skia::GradientStop> = gradient
                        .stops
                        .iter()
                        .map(|stop| {
                            tiny_skia::GradientStop::new(stop.position, to_skia_color(stop.color))
                        })
                        .collect();
                    if let Some(shader) = tiny_skia::LinearGradient::new(
                        tiny_skia::Point { x: sx, y: sy },
                        tiny_skia::Point { x: ex, y: ey },
                        stops,
                        tiny_skia::SpreadMode::Pad,
                        tiny_skia::Transform::identity(),
                    ) {
                        let fill = SkiaPaint {
                            anti_alias: true,
                            shader,
                            ..SkiaPaint::default()
                        };
                        self.pixmap
                            .fill_path(&path, &fill, FillRule::Winding, transform, clip);
                    }
                }
                Paint::Pattern(pattern) => {
                    let pattern = pattern.clone();
                    let tile_size = pattern.size.round().max(2.0) as u32;
                    let Some(mut tile) = tiny_skia::Pixmap::new(tile_size, tile_size) else {
                        return Err(RenderError::PatternTile);
                    };
                    render_pattern_tile(&mut tile, &pattern);
                    let shader = tiny_skia::Pattern::new(
                        tile.as_ref(),
                        tiny_skia::SpreadMode::Repeat,
                        tiny_skia::FilterQuality::Nearest,
                        opacity,
                        tiny_skia::Transform::identity(),
                    );
                    let fill = SkiaPaint {
                        anti_alias: true,
                        shader,
                        ..SkiaPaint::default()
                    };
                    self.pixmap
                        .fill_path(&path, &fill, FillRule::Winding, transform, clip);
                    // keep tile alive until end of block
                    let _ = tile;
                }
                Paint::RadialGradient(gradient) | Paint::Gradient(Gradient::Radial(gradient)) => {
                    let gradient = gradient.clone();
                    let (min_x, min_y, max_x, max_y) = geometry_bounds(&shape.geometry);
                    let cx = (min_x + max_x) / 2.0;
                    let cy = (min_y + max_y) / 2.0;

                    let (cx, cy) = transform_point(cx, cy, &transform);
                    // Approximate scale for radius.
                    let (sx, _) = transform_point(min_x, cy, &transform);
                    let scaled_radius = (sx - cx).abs().max(1.0);
                    let stops: Vec<tiny_skia::GradientStop> = gradient
                        .stops
                        .iter()
                        .map(|stop| {
                            tiny_skia::GradientStop::new(
                                stop.position,
                                to_skia_color(Color {
                                    a: stop.color.a * opacity,
                                    ..stop.color
                                }),
                            )
                        })
                        .collect();
                    if let Some(shader) = tiny_skia::RadialGradient::new(
                        tiny_skia::Point { x: cx, y: cy },
                        0.0,
                        tiny_skia::Point { x: cx, y: cy },
                        scaled_radius,
                        stops,
                        tiny_skia::SpreadMode::Pad,
                        tiny_skia::Transform::identity(),
                    ) {
                        let fill = SkiaPaint {
                            anti_alias: true,
                            shader,
                            ..SkiaPaint::default()
                        };
                        self.pixmap
                            .fill_path(&path, &fill, FillRule::Winding, transform, clip);
                    }
                }
            }
        }

        if let Some(stroke) = &shape.stroke {
            let color = solid_paint_color(paints, stroke.paint)?;
            let mut stroke_paint = SkiaPaint {
                anti_alias: true,
                ..SkiaPaint::default()
            };
            stroke_paint.set_color(to_skia_color(with_opacity(color, opacity)));
            let skia_stroke = to_skia_stroke(&stroke.style);
            self.pixmap
                .stroke_path(&path, &stroke_paint, &skia_stroke, transform, clip);
        }

        Ok(())
    }

    /// Render a text primitive into the canvas.
    fn render_text(
        &mut self,
        text: &TextNode,
        parent: Transform,
        opacity_scale: f32,
        _clip: Option<&Mask>,
    ) -> Result<(), RenderError> {
        let opacity = (text.opacity * opacity_scale).clamp(0.0, 1.0);
        if opacity <= 0.0 || text.text.is_empty() {
            return Ok(());
        }

        let bold = matches!(text.weight, rupa_ir::document::FontWeight::Bold);
        let cache_key = format!("{}:{}", text.font.clone().unwrap_or_default(), bold);
        if !self.font_cache.contains_key(&cache_key) {
            let bytes = resolve_font_bytes(text.font.as_deref(), bold)?;
            self.font_cache.insert(cache_key.clone(), bytes);
        }
        let font_bytes = self.font_cache.get(&cache_key).cloned().unwrap_or_default();
        let font =
            fontdue::Font::from_bytes(font_bytes.as_slice(), fontdue::FontSettings::default())
                .map_err(|_| RenderError::MissingFont(text.font.clone().unwrap_or_default()))?;

        let transform = parent.multiply(&text.transform);

        // Measure glyphs.
        let mut entries: Vec<(fontdue::Metrics, Vec<u8>)> = Vec::new();
        let mut total_width = 0.0f32;
        for ch in text.text.chars() {
            let (metrics, bitmap) = font.rasterize(ch, text.size);
            total_width += metrics.advance_width;
            entries.push((metrics, bitmap));
        }

        let start_offset = match text.align {
            rupa_ir::document::TextAlign::Center => -total_width / 2.0,
            rupa_ir::document::TextAlign::Right => -total_width,
            rupa_ir::document::TextAlign::Left => 0.0,
        };

        let mut cursor = text.x + start_offset;
        let data = self.pixmap.data_mut();
        let width = self.width as i32;
        let height = self.height as i32;
        for (metrics, bitmap) in &entries {
            let gx = cursor + metrics.xmin as f32;
            let gy = text.y + metrics.ymin as f32;
            let (dx, dy) = transform.transform_point(gx, gy);
            let dx = dx.round() as i32;
            let dy = dy.round() as i32;

            for row in 0..metrics.height as i32 {
                for col in 0..metrics.width as i32 {
                    let src_alpha =
                        bitmap[(row as usize) * metrics.width + (col as usize)] as f32 / 255.0;
                    if src_alpha <= 0.0 {
                        continue;
                    }
                    let px = dx + col;
                    let py = dy + row;
                    if px < 0 || py < 0 || px >= width || py >= height {
                        continue;
                    }
                    let factor = src_alpha * opacity;
                    let src_r = text.fill.r * factor;
                    let src_g = text.fill.g * factor;
                    let src_b = text.fill.b * factor;
                    let src_a = factor;

                    let idx = ((py as u32 * self.width + px as u32) * 4) as usize;
                    let dst_r = data[idx] as f32 / 255.0;
                    let dst_g = data[idx + 1] as f32 / 255.0;
                    let dst_b = data[idx + 2] as f32 / 255.0;
                    let dst_a = data[idx + 3] as f32 / 255.0;

                    let out_a = src_a + dst_a * (1.0 - src_a);
                    let out_r = if out_a > 0.0 {
                        (src_r + dst_r * dst_a * (1.0 - src_a)) / out_a
                    } else {
                        0.0
                    };
                    let out_g = if out_a > 0.0 {
                        (src_g + dst_g * dst_a * (1.0 - src_a)) / out_a
                    } else {
                        0.0
                    };
                    let out_b = if out_a > 0.0 {
                        (src_b + dst_b * dst_a * (1.0 - src_a)) / out_a
                    } else {
                        0.0
                    };

                    data[idx] = (out_r * out_a * 255.0).round() as u8;
                    data[idx + 1] = (out_g * out_a * 255.0).round() as u8;
                    data[idx + 2] = (out_b * out_a * 255.0).round() as u8;
                    data[idx + 3] = (out_a * 255.0).round() as u8;
                }
            }
            cursor += metrics.advance_width;
        }

        Ok(())
    }

    /// Render one external picture into the canvas.
    ///
    /// The frame is `at` plus the optional declared size (natural image size
    /// when absent). Fit modes position and scale the image within it:
    /// `contain` letterboxes, `cover` center-crops, `fill` stretches, and
    /// `none` draws at natural size from the frame's top-left. The frame is
    /// clipped in device space, so rotated frames crop correctly.
    fn render_picture(
        &mut self,
        picture: &PictureNode,
        parent: Transform,
        opacity_scale: f32,
        clip: Option<&Mask>,
        resources: &mut ResourceStore,
    ) -> Result<(), RenderError> {
        let opacity = (picture.opacity * opacity_scale).clamp(0.0, 1.0);
        if opacity <= 0.0 {
            return Ok(());
        }

        let image = resources
            .load(&picture.path)
            .map_err(|error| RenderError::Resource {
                path: picture.path.clone(),
                detail: error.to_string(),
            })?;
        let natural_width = image.width as f32;
        let natural_height = image.height as f32;
        let frame_width = picture.width.unwrap_or(natural_width);
        let frame_height = picture.height.unwrap_or(natural_height);
        if !frame_width.is_finite()
            || !frame_height.is_finite()
            || frame_width <= 0.0
            || frame_height <= 0.0
        {
            return Err(RenderError::InvalidPicture(format!(
                "frame must be finite and positive, got {frame_width}x{frame_height}"
            )));
        }

        let (scale_x, scale_y, offset_x, offset_y) = fit_layout(
            picture.fit,
            picture.x,
            picture.y,
            frame_width,
            frame_height,
            natural_width,
            natural_height,
        );

        let center = (
            picture.x + frame_width / 2.0,
            picture.y + frame_height / 2.0,
        );
        let local = transform_about_origin(&picture.transform, center);
        let frame_transform = parent.multiply(&local);
        let frame_clip = self.rect_clip_mask(
            picture.x,
            picture.y,
            frame_width,
            frame_height,
            &frame_transform,
            clip,
        )?;

        let image_transform = frame_transform
            .multiply(&Transform::translate(offset_x, offset_y))
            .multiply(&Transform::scale(scale_x, scale_y));
        let skia_image = image_pixmap(image).ok_or_else(|| {
            RenderError::InvalidPicture("could not build image pixmap".to_string())
        })?;
        let paint = PixmapPaint {
            opacity,
            ..PixmapPaint::default()
        };
        self.pixmap.draw_pixmap(
            0,
            0,
            skia_image.as_ref(),
            &paint,
            to_skia_transform(&image_transform),
            Some(&frame_clip),
        );
        Ok(())
    }
}

/// Compute scale and offset mapping a natural image size into a frame.
fn fit_layout(
    fit: FitMode,
    frame_x: f32,
    frame_y: f32,
    frame_width: f32,
    frame_height: f32,
    natural_width: f32,
    natural_height: f32,
) -> (f32, f32, f32, f32) {
    let contain = (frame_width / natural_width).min(frame_height / natural_height);
    match fit {
        FitMode::Fill => (
            frame_width / natural_width,
            frame_height / natural_height,
            frame_x,
            frame_y,
        ),
        FitMode::None => (1.0, 1.0, frame_x, frame_y),
        FitMode::Contain => (
            contain,
            contain,
            frame_x + (frame_width - natural_width * contain) / 2.0,
            frame_y + (frame_height - natural_height * contain) / 2.0,
        ),
        FitMode::Cover => {
            let cover = (frame_width / natural_width).max(frame_height / natural_height);
            (
                cover,
                cover,
                frame_x + (frame_width - natural_width * cover) / 2.0,
                frame_y + (frame_height - natural_height * cover) / 2.0,
            )
        }
    }
}

/// Convert a straight-alpha image buffer into a premultiplied skia pixmap.
fn image_pixmap(image: &ImageBuffer) -> Option<Pixmap> {
    let mut data = Vec::with_capacity(image.pixels.len());
    for pixel in image.pixels.as_chunks::<4>().0 {
        let alpha = u32::from(pixel[3]);
        let channel = |value: u8| ((u32::from(value) * alpha + 127) / 255) as u8;
        data.extend_from_slice(&[
            channel(pixel[0]),
            channel(pixel[1]),
            channel(pixel[2]),
            pixel[3],
        ]);
    }
    let size = tiny_skia::IntSize::from_wh(image.width, image.height)?;
    Pixmap::from_vec(data, size)
}

/// Render an IR document to a pixel buffer.
///
/// Documents containing pictures need [`render_document_with_resources`],
/// which resolves and decodes their resources; this entry point reports
/// [`RenderError::MissingResource`] for them.
pub fn render_document(document: &RenderDocument) -> Result<RenderOutput, RenderError> {
    render_document_inner(document, None)
}

/// Render an IR document, resolving picture resources relative to the store.
pub fn render_document_with_resources(
    document: &RenderDocument,
    resources: &mut ResourceStore,
) -> Result<RenderOutput, RenderError> {
    render_document_inner(document, Some(resources))
}

fn render_document_inner(
    document: &RenderDocument,
    resources: Option<&mut ResourceStore>,
) -> Result<RenderOutput, RenderError> {
    let mut renderer = VectorRenderer::new(document.width, document.height)?;
    if let Some(background) = document.background {
        renderer.clear(background);
    }
    renderer.render_composition(
        &document.root.children,
        document,
        Transform::identity(),
        1.0,
        &mut Vec::new(),
        None,
        resources,
    )?;
    Ok(RenderOutput {
        width: renderer.width,
        height: renderer.height,
        pixels: renderer.into_pixels(),
    })
}

fn geometry_path(geometry: &Geometry) -> Result<Path, RenderError> {
    let mut builder = PathBuilder::new();
    match geometry {
        Geometry::Rectangle(rect) => {
            if !rect.x.is_finite()
                || !rect.y.is_finite()
                || !rect.width.is_finite()
                || !rect.height.is_finite()
                || rect.width <= 0.0
                || rect.height <= 0.0
            {
                return Err(RenderError::InvalidGeometry {
                    shape: "rectangle",
                    detail: "position must be finite and width/height must be positive".to_string(),
                });
            }
            let skia_rect = tiny_skia::Rect::from_xywh(rect.x, rect.y, rect.width, rect.height)
                .ok_or_else(|| RenderError::InvalidGeometry {
                    shape: "rectangle",
                    detail: "could not construct rectangle".to_string(),
                })?;
            builder.push_rect(skia_rect);
        }
        Geometry::Circle(circle) => {
            if !circle.cx.is_finite()
                || !circle.cy.is_finite()
                || !circle.radius.is_finite()
                || circle.radius <= 0.0
            {
                return Err(RenderError::InvalidGeometry {
                    shape: "circle",
                    detail: "center must be finite and radius must be positive".to_string(),
                });
            }
            builder.push_circle(circle.cx, circle.cy, circle.radius);
        }
        Geometry::Ellipse(ellipse) => {
            if !ellipse.cx.is_finite()
                || !ellipse.cy.is_finite()
                || !ellipse.width.is_finite()
                || !ellipse.height.is_finite()
                || ellipse.width <= 0.0
                || ellipse.height <= 0.0
            {
                return Err(RenderError::InvalidGeometry {
                    shape: "ellipse",
                    detail: "center must be finite and width/height must be positive".to_string(),
                });
            }
            let rect = tiny_skia::Rect::from_xywh(
                ellipse.cx - ellipse.width / 2.0,
                ellipse.cy - ellipse.height / 2.0,
                ellipse.width,
                ellipse.height,
            )
            .ok_or_else(|| RenderError::InvalidGeometry {
                shape: "ellipse",
                detail: "could not construct ellipse bounds".to_string(),
            })?;
            builder.push_oval(rect);
        }
        Geometry::Triangle(triangle) => {
            if !triangle.x.is_finite()
                || !triangle.y.is_finite()
                || !triangle.width.is_finite()
                || !triangle.height.is_finite()
                || triangle.width <= 0.0
                || triangle.height <= 0.0
            {
                return Err(RenderError::InvalidGeometry {
                    shape: "triangle",
                    detail: "position must be finite and width/height must be positive".to_string(),
                });
            }
            let apex_x = triangle.x + triangle.width / 2.0;
            let base_y = triangle.y + triangle.height;
            builder.move_to(apex_x, triangle.y);
            builder.line_to(triangle.x, base_y);
            builder.line_to(triangle.x + triangle.width, base_y);
            builder.close();
        }
        Geometry::Polygon(polygon) => {
            if polygon.points.len() < 3 {
                return Err(RenderError::InvalidGeometry {
                    shape: "polygon",
                    detail: "at least three points are required".to_string(),
                });
            }
            if polygon
                .points
                .iter()
                .any(|point| !point.x.is_finite() || !point.y.is_finite())
            {
                return Err(RenderError::InvalidGeometry {
                    shape: "polygon",
                    detail: "point coordinates must be finite".to_string(),
                });
            }
            let first = polygon.points[0];
            builder.move_to(first.x, first.y);
            for point in &polygon.points[1..] {
                builder.line_to(point.x, point.y);
            }
            builder.close();
        }
        Geometry::Line(line) => {
            if !line.x1.is_finite()
                || !line.y1.is_finite()
                || !line.x2.is_finite()
                || !line.y2.is_finite()
            {
                return Err(RenderError::InvalidGeometry {
                    shape: "line",
                    detail: "endpoints must be finite".to_string(),
                });
            }
            builder.move_to(line.x1, line.y1);
            builder.line_to(line.x2, line.y2);
        }
        Geometry::Polyline(polyline) => {
            if polyline.points.len() < 2 {
                return Err(RenderError::InvalidGeometry {
                    shape: "polyline",
                    detail: "at least two points are required".to_string(),
                });
            }
            if polyline
                .points
                .iter()
                .any(|point| !point.x.is_finite() || !point.y.is_finite())
            {
                return Err(RenderError::InvalidGeometry {
                    shape: "polyline",
                    detail: "point coordinates must be finite".to_string(),
                });
            }
            let first = polyline.points[0];
            builder.move_to(first.x, first.y);
            for point in &polyline.points[1..] {
                builder.line_to(point.x, point.y);
            }
        }
        Geometry::Path(path) => {
            if path.commands.is_empty() {
                return Err(RenderError::InvalidGeometry {
                    shape: "path",
                    detail: "at least one command is required".to_string(),
                });
            }
            let mut started = false;
            for command in &path.commands {
                match command {
                    rupa_ir::geometry::PathCommand::MoveTo(point) => {
                        ensure_point(point, "path")?;
                        builder.move_to(point.x, point.y);
                        started = true;
                    }
                    rupa_ir::geometry::PathCommand::LineTo(point) => {
                        if !started {
                            return Err(path_not_started());
                        }
                        ensure_point(point, "path")?;
                        builder.line_to(point.x, point.y);
                    }
                    rupa_ir::geometry::PathCommand::CubicTo {
                        control1,
                        control2,
                        to,
                    } => {
                        if !started {
                            return Err(path_not_started());
                        }
                        ensure_point(control1, "path")?;
                        ensure_point(control2, "path")?;
                        ensure_point(to, "path")?;
                        builder
                            .cubic_to(control1.x, control1.y, control2.x, control2.y, to.x, to.y);
                    }
                    rupa_ir::geometry::PathCommand::QuadTo { control, to } => {
                        if !started {
                            return Err(path_not_started());
                        }
                        ensure_point(control, "path")?;
                        ensure_point(to, "path")?;
                        builder.quad_to(control.x, control.y, to.x, to.y);
                    }
                    rupa_ir::geometry::PathCommand::Close => {
                        if !started {
                            return Err(path_not_started());
                        }
                        builder.close();
                        started = false;
                    }
                }
            }
        }
    }
    builder.finish().ok_or(RenderError::PathBuild)
}

fn geometry_center(geometry: &Geometry) -> (f32, f32) {
    match geometry {
        Geometry::Rectangle(rect) => (rect.x + rect.width / 2.0, rect.y + rect.height / 2.0),
        Geometry::Circle(circle) => (circle.cx, circle.cy),
        Geometry::Ellipse(ellipse) => (ellipse.cx, ellipse.cy),
        Geometry::Triangle(triangle) => (
            triangle.x + triangle.width / 2.0,
            triangle.y + triangle.height / 2.0,
        ),
        Geometry::Polygon(polygon) => polygon_center(&polygon.points),
        Geometry::Line(line) => ((line.x1 + line.x2) / 2.0, (line.y1 + line.y2) / 2.0),
        Geometry::Polyline(polyline) => polygon_center(&polyline.points),
        Geometry::Path(path) => path_center(&path.commands),
    }
}

fn polygon_center(points: &[rupa_ir::geometry::Point]) -> (f32, f32) {
    let Some(first) = points.first() else {
        return (0.0, 0.0);
    };
    let (mut min_x, mut max_x) = (first.x, first.x);
    let (mut min_y, mut max_y) = (first.y, first.y);
    for point in points.iter().skip(1) {
        min_x = min_x.min(point.x);
        max_x = max_x.max(point.x);
        min_y = min_y.min(point.y);
        max_y = max_y.max(point.y);
    }
    ((min_x + max_x) / 2.0, (min_y + max_y) / 2.0)
}

fn solid_paint_color(paints: &PaintStore, id: PaintId) -> Result<Color, RenderError> {
    let paint = paints.get(id).ok_or(RenderError::MissingPaint(id))?;
    match paint {
        Paint::Solid(color) => Ok(*color),
        _ => Err(RenderError::MissingPaint(id)),
    }
}

fn geometry_bounds(geometry: &rupa_ir::geometry::Geometry) -> (f32, f32, f32, f32) {
    use rupa_ir::geometry::Geometry;
    match geometry {
        Geometry::Rectangle(rect) => (rect.x, rect.y, rect.x + rect.width, rect.y + rect.height),
        Geometry::Circle(circle) => (
            circle.cx - circle.radius,
            circle.cy - circle.radius,
            circle.cx + circle.radius,
            circle.cy + circle.radius,
        ),
        Geometry::Ellipse(ellipse) => (
            ellipse.cx - ellipse.width / 2.0,
            ellipse.cy - ellipse.height / 2.0,
            ellipse.cx + ellipse.width / 2.0,
            ellipse.cy + ellipse.height / 2.0,
        ),
        Geometry::Triangle(triangle) => (
            triangle.x,
            triangle.y,
            triangle.x + triangle.width,
            triangle.y + triangle.height,
        ),
        Geometry::Polygon(polygon) => bounds_points(&polygon.points),
        Geometry::Line(line) => (
            line.x1.min(line.x2),
            line.y1.min(line.y2),
            line.x1.max(line.x2),
            line.y1.max(line.y2),
        ),
        Geometry::Polyline(polyline) => bounds_points(&polyline.points),
        Geometry::Path(path) => {
            // Approximate bounds from control points.
            let mut min_x = f32::INFINITY;
            let mut min_y = f32::INFINITY;
            let mut max_x = f32::NEG_INFINITY;
            let mut max_y = f32::NEG_INFINITY;
            for command in &path.commands {
                use rupa_ir::geometry::PathCommand;
                let mut include = |x: f32, y: f32| {
                    min_x = min_x.min(x);
                    min_y = min_y.min(y);
                    max_x = max_x.max(x);
                    max_y = max_y.max(y);
                };
                match command {
                    PathCommand::MoveTo(point) | PathCommand::LineTo(point) => {
                        include(point.x, point.y)
                    }
                    PathCommand::CubicTo {
                        control1,
                        control2,
                        to,
                    } => {
                        include(control1.x, control1.y);
                        include(control2.x, control2.y);
                        include(to.x, to.y);
                    }
                    PathCommand::QuadTo { control, to } => {
                        include(control.x, control.y);
                        include(to.x, to.y);
                    }
                    PathCommand::Close => {}
                }
            }
            if min_x.is_finite() {
                (min_x, min_y, max_x, max_y)
            } else {
                (0.0, 0.0, 1.0, 1.0)
            }
        }
    }
}

fn bounds_points(points: &[rupa_ir::geometry::Point]) -> (f32, f32, f32, f32) {
    if points.is_empty() {
        return (0.0, 0.0, 1.0, 1.0);
    }
    let mut min_x = points[0].x;
    let mut min_y = points[0].y;
    let mut max_x = points[0].x;
    let mut max_y = points[0].y;
    for point in points.iter().skip(1) {
        min_x = min_x.min(point.x);
        min_y = min_y.min(point.y);
        max_x = max_x.max(point.x);
        max_y = max_y.max(point.y);
    }
    (min_x, min_y, max_x, max_y)
}

fn transform_point(x: f32, y: f32, transform: &tiny_skia::Transform) -> (f32, f32) {
    let mut point = tiny_skia::Point { x, y };
    transform.map_point(&mut point);
    (point.x, point.y)
}

fn render_pattern_tile(tile: &mut tiny_skia::Pixmap, pattern: &rupa_ir::paint::PatternPaint) {
    let size = tile.width();
    let fg = to_skia_color(pattern.color);
    let bg = to_skia_color(pattern.background);
    let data = tile.data_mut();
    for y in 0..size {
        for x in 0..size {
            let idx = ((y * size + x) * 4) as usize;
            let color = match pattern.kind.as_str() {
                "stripes" => {
                    if (x / (size / 2).max(1)).is_multiple_of(2) {
                        fg
                    } else {
                        bg
                    }
                }
                "checkerboard" => {
                    let cell = size / 2;
                    if ((x / cell.max(1)) + (y / cell.max(1))).is_multiple_of(2) {
                        fg
                    } else {
                        bg
                    }
                }
                "dots" => {
                    let cx = size as f32 / 2.0;
                    let cy = size as f32 / 2.0;
                    let r = size as f32 / 4.0;
                    let dx = x as f32 - cx;
                    let dy = y as f32 - cy;
                    if dx * dx + dy * dy <= r * r { fg } else { bg }
                }
                "grid" => {
                    if x == 0 || y == 0 {
                        fg
                    } else {
                        bg
                    }
                }
                _ => bg,
            };
            let r = (color.red() * 255.0).round() as u8;
            let g = (color.green() * 255.0).round() as u8;
            let b = (color.blue() * 255.0).round() as u8;
            let a = (color.alpha() * 255.0).round() as u8;
            data[idx] = r;
            data[idx + 1] = g;
            data[idx + 2] = b;
            data[idx + 3] = a;
        }
    }
}

fn with_gradient_opacity(
    gradient: &rupa_ir::paint::LinearGradient,
    opacity: f32,
) -> rupa_ir::paint::LinearGradient {
    let mut gradient = gradient.clone();
    for stop in &mut gradient.stops {
        stop.color.a *= opacity;
    }
    gradient
}

fn to_skia_stroke(style: &rupa_ir::geometry::StrokeStyle) -> tiny_skia::Stroke {
    use rupa_ir::geometry::{LineCap, LineJoin};
    tiny_skia::Stroke {
        width: style.width,
        miter_limit: style.miter_limit,
        line_cap: match style.cap {
            LineCap::Butt => tiny_skia::LineCap::Butt,
            LineCap::Round => tiny_skia::LineCap::Round,
            LineCap::Square => tiny_skia::LineCap::Square,
        },
        line_join: match style.join {
            LineJoin::Miter => tiny_skia::LineJoin::Miter,
            LineJoin::Round => tiny_skia::LineJoin::Round,
            LineJoin::Bevel => tiny_skia::LineJoin::Bevel,
        },
        ..tiny_skia::Stroke::default()
    }
}

fn ensure_point(point: &rupa_ir::geometry::Point, shape: &'static str) -> Result<(), RenderError> {
    if point.x.is_finite() && point.y.is_finite() {
        Ok(())
    } else {
        Err(RenderError::InvalidGeometry {
            shape,
            detail: "point coordinates must be finite".to_string(),
        })
    }
}

fn path_not_started() -> RenderError {
    RenderError::InvalidGeometry {
        shape: "path",
        detail: "path must begin each subpath with a move command".to_string(),
    }
}

fn path_center(commands: &[rupa_ir::geometry::PathCommand]) -> (f32, f32) {
    use rupa_ir::geometry::PathCommand;
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    let mut include = |point: &rupa_ir::geometry::Point| {
        min_x = min_x.min(point.x);
        min_y = min_y.min(point.y);
        max_x = max_x.max(point.x);
        max_y = max_y.max(point.y);
    };
    for command in commands {
        match command {
            PathCommand::MoveTo(point) | PathCommand::LineTo(point) => include(point),
            PathCommand::CubicTo {
                control1,
                control2,
                to,
            } => {
                include(control1);
                include(control2);
                include(to);
            }
            PathCommand::QuadTo { control, to } => {
                include(control);
                include(to);
            }
            PathCommand::Close => {}
        }
    }
    if min_x.is_finite() {
        ((min_x + max_x) / 2.0, (min_y + max_y) / 2.0)
    } else {
        (0.0, 0.0)
    }
}

/// Build `translate(origin) * transform * translate(-origin)`.
pub fn transform_about_origin(transform: &Transform, origin: (f32, f32)) -> Transform {
    Transform::translate(origin.0, origin.1)
        .multiply(transform)
        .multiply(&Transform::translate(-origin.0, -origin.1))
}

fn with_opacity(color: Color, opacity: f32) -> Color {
    Color {
        a: color.a * opacity,
        ..color
    }
}

fn to_skia_color(color: Color) -> SkiaColor {
    let [r, g, b, a] = color.to_rgba8();
    SkiaColor::from_rgba8(r, g, b, a)
}

fn to_skia_transform(transform: &Transform) -> SkiaTransform {
    SkiaTransform::from_row(
        transform.a,
        transform.b,
        transform.c,
        transform.d,
        transform.e,
        transform.f,
    )
}

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod tests {
    use super::*;
    use rupa_ir::geometry::{
        Circle as IrCircle, Ellipse as IrEllipse, Line as IrLine, Path as IrPath,
        PathCommand as IrPathCommand, Point as IrPoint, Polygon as IrPolygon,
        Polyline as IrPolyline, Rectangle as IrRectangle, Stroke as IrStroke, StrokeStyle,
        Triangle as IrTriangle,
    };
    use rupa_ir::paint::Color;

    const RED: Color = Color::new(1.0, 0.0, 0.0, 1.0);
    const BLUE: Color = Color::new(0.0, 0.0, 1.0, 1.0);
    const WHITE: Color = Color::white();

    fn rect_shape(x: f32, y: f32, width: f32, height: f32, paint: PaintId) -> ShapeNode {
        ShapeNode {
            geometry: Geometry::Rectangle(IrRectangle {
                x,
                y,
                width,
                height,
            }),
            fill: Some(paint),
            stroke: None,
            transform: Transform::identity(),
            opacity: 1.0,
        }
    }

    fn circle_shape(cx: f32, cy: f32, radius: f32, paint: PaintId) -> ShapeNode {
        ShapeNode {
            geometry: Geometry::Circle(IrCircle { cx, cy, radius }),
            fill: Some(paint),
            stroke: None,
            transform: Transform::identity(),
            opacity: 1.0,
        }
    }

    fn document(width: u32, height: u32) -> RenderDocument {
        RenderDocument::new(width, height).with_background(WHITE)
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
    fn invalid_dimensions_are_rejected() {
        assert_eq!(
            render_document(&RenderDocument::new(0, 10)).unwrap_err(),
            RenderError::InvalidDimensions
        );
        assert_eq!(
            render_document(&RenderDocument::new(10, 0)).unwrap_err(),
            RenderError::InvalidDimensions
        );
    }

    #[test]
    fn rectangle_renders_at_position_and_size() {
        let mut doc = document(100, 60);
        let red = doc.add_paint(Paint::Solid(RED));
        doc.push_shape(rect_shape(10.0, 10.0, 80.0, 40.0, red));

        let output = render_document(&doc).expect("render");
        assert_eq!(output.width, 100);
        assert_eq!(output.height, 60);
        assert_eq!(output.pixels.len(), 100 * 60 * 4);
        assert_eq!(output.pixel(50, 30), Some([255, 0, 0, 255]));
        // Outside the rectangle remains background white.
        assert_eq!(output.pixel(5, 5), Some([255, 255, 255, 255]));
        assert_eq!(output.pixel(95, 55), Some([255, 255, 255, 255]));
    }

    #[test]
    fn circle_renders_at_center_and_radius() {
        let mut doc = document(200, 200);
        let blue = doc.add_paint(Paint::Solid(BLUE));
        doc.push_shape(circle_shape(100.0, 100.0, 30.0, blue));

        let output = render_document(&doc).expect("render");
        assert_eq!(output.pixel(100, 100), Some([0, 0, 255, 255]));
        assert_eq!(output.pixel(100, 75), Some([0, 0, 255, 255]));
        // Outside the circle.
        assert_eq!(output.pixel(100, 60), Some([255, 255, 255, 255]));
        assert_eq!(output.pixel(10, 10), Some([255, 255, 255, 255]));
    }

    #[test]
    fn colors_match_expected_srgb_values() {
        let mut doc = document(20, 20);
        let color = Color::from_rgba8(0x79, 0x50, 0xF2, 0xFF);
        let paint = doc.add_paint(Paint::Solid(color));
        doc.push_shape(rect_shape(0.0, 0.0, 20.0, 20.0, paint));

        let output = render_document(&doc).expect("render");
        assert_eq!(output.pixel(10, 10), Some([0x79, 0x50, 0xF2, 255]));
    }

    #[test]
    fn transparent_background_produces_zero_alpha_outside_shape() {
        let mut doc = RenderDocument::new(40, 40);
        let red = doc.add_paint(Paint::Solid(RED));
        doc.push_shape(rect_shape(10.0, 10.0, 20.0, 20.0, red));

        let output = render_document(&doc).expect("render");
        assert_eq!(output.pixel(0, 0), Some([0, 0, 0, 0]));
        assert_eq!(output.pixel(20, 20), Some([255, 0, 0, 255]));
    }

    #[test]
    fn semi_transparent_fill_is_premultiplied_in_output() {
        let mut doc = RenderDocument::new(20, 20);
        let half_red = doc.add_paint(Paint::Solid(Color::new(1.0, 0.0, 0.0, 0.5)));
        doc.push_shape(rect_shape(0.0, 0.0, 20.0, 20.0, half_red));

        let output = render_document(&doc).expect("render");
        // tiny-skia stores premultiplied RGBA: red 50% -> (128, 0, 0, 128).
        assert_near(output.pixel(10, 10).unwrap(), [128, 0, 0, 128], 1);
    }

    #[test]
    fn overlapping_semi_transparent_shapes_composite_in_order() {
        let mut doc = document(20, 20);
        let half_red = doc.add_paint(Paint::Solid(Color::new(1.0, 0.0, 0.0, 0.5)));
        let half_blue = doc.add_paint(Paint::Solid(Color::new(0.0, 0.0, 1.0, 0.5)));
        doc.push_shape(rect_shape(0.0, 0.0, 20.0, 20.0, half_red));
        doc.push_shape(rect_shape(0.0, 0.0, 20.0, 20.0, half_blue));

        let output = render_document(&doc).expect("render");
        // red 50% over white -> (255,128,128); blue 50% over that -> (128,64,192).
        assert_near(output.pixel(10, 10).unwrap(), [128, 64, 192, 255], 2);
    }

    #[test]
    fn translate_transform_shifts_shape() {
        let mut doc = document(100, 50);
        let red = doc.add_paint(Paint::Solid(RED));
        let mut shape = rect_shape(10.0, 10.0, 20.0, 20.0, red);
        shape.transform = Transform::translate(30.0, 5.0);
        doc.push_shape(shape);

        let output = render_document(&doc).expect("render");
        assert_eq!(output.pixel(15, 15), Some([255, 255, 255, 255]));
        assert_eq!(output.pixel(45, 20), Some([255, 0, 0, 255]));
    }

    #[test]
    fn scale_uses_shape_center_as_origin() {
        let mut doc = RenderDocument::new(40, 40);
        let red = doc.add_paint(Paint::Solid(RED));
        let mut shape = rect_shape(10.0, 10.0, 10.0, 10.0, red);
        // Center (15,15), scale 2 -> bounds (5,5)-(25,25).
        shape.transform = Transform::scale(2.0, 2.0);
        doc.push_shape(shape);

        let output = render_document(&doc).expect("render");
        assert_eq!(output.pixel(6, 6), Some([255, 0, 0, 255]));
        assert_eq!(output.pixel(4, 4), Some([0, 0, 0, 0]));
        assert_eq!(output.pixel(24, 24), Some([255, 0, 0, 255]));
        assert_eq!(output.pixel(26, 26), Some([0, 0, 0, 0]));
    }

    #[test]
    fn rotate_uses_shape_center_as_origin() {
        let mut doc = RenderDocument::new(60, 60);
        let red = doc.add_paint(Paint::Solid(RED));
        // Rectangle (20,20) 40x20 -> center (40,30). A quarter turn about
        // the center gives bounds (30,10)-(50,50).
        let mut shape = rect_shape(20.0, 20.0, 40.0, 20.0, red);
        shape.transform = Transform::rotate(std::f32::consts::FRAC_PI_2);
        doc.push_shape(shape);

        let output = render_document(&doc).expect("render");
        assert_eq!(output.pixel(40, 15), Some([255, 0, 0, 255]));
        assert_eq!(output.pixel(40, 45), Some([255, 0, 0, 255]));
        // Points that were inside before rotation but outside after it.
        assert_eq!(output.pixel(22, 30), Some([0, 0, 0, 0]));
        assert_eq!(output.pixel(58, 30), Some([0, 0, 0, 0]));
    }

    #[test]
    fn ellipse_renders_at_center_and_full_size() {
        let mut doc = RenderDocument::new(200, 200);
        let red = doc.add_paint(Paint::Solid(RED));
        doc.push_shape(ShapeNode {
            geometry: Geometry::Ellipse(IrEllipse {
                cx: 100.0,
                cy: 100.0,
                width: 80.0,
                height: 40.0,
            }),
            fill: Some(red),
            stroke: None,
            transform: Transform::identity(),
            opacity: 1.0,
        });

        let output = render_document(&doc).expect("render");
        assert_eq!(output.pixel(100, 100), Some([255, 0, 0, 255]));
        assert_eq!(output.pixel(135, 100), Some([255, 0, 0, 255]));
        assert_eq!(output.pixel(100, 115), Some([255, 0, 0, 255]));
        // Outside the ellipse (full width 80, height 40).
        assert_eq!(output.pixel(100, 125), Some([0, 0, 0, 0]));
        assert_eq!(output.pixel(145, 100), Some([0, 0, 0, 0]));
    }

    #[test]
    fn square_renders_as_equal_sides_from_top_left() {
        let mut doc = document(100, 100);
        let red = doc.add_paint(Paint::Solid(RED));
        doc.push_shape(rect_shape(10.0, 20.0, 30.0, 30.0, red));

        let output = render_document(&doc).expect("render");
        assert_eq!(output.pixel(20, 30), Some([255, 0, 0, 255]));
        assert_eq!(output.pixel(5, 25), Some([255, 255, 255, 255]));
        assert_eq!(output.pixel(45, 55), Some([255, 255, 255, 255]));
    }

    #[test]
    fn triangle_renders_apex_up_inside_bounding_box() {
        let mut doc = RenderDocument::new(100, 100);
        let red = doc.add_paint(Paint::Solid(RED));
        doc.push_shape(ShapeNode {
            geometry: Geometry::Triangle(IrTriangle {
                x: 20.0,
                y: 20.0,
                width: 40.0,
                height: 40.0,
            }),
            fill: Some(red),
            stroke: None,
            transform: Transform::identity(),
            opacity: 1.0,
        });

        let output = render_document(&doc).expect("render");
        // Apex at (40,20); base from (20,60) to (60,60).
        assert_eq!(output.pixel(40, 25), Some([255, 0, 0, 255]));
        assert_eq!(output.pixel(40, 50), Some([255, 0, 0, 255]));
        assert_eq!(output.pixel(22, 50), Some([0, 0, 0, 0]));
        assert_eq!(output.pixel(40, 15), Some([0, 0, 0, 0]));
    }

    #[test]
    fn polygon_renders_as_closed_filled_path() {
        let mut doc = RenderDocument::new(100, 100);
        let red = doc.add_paint(Paint::Solid(RED));
        doc.push_shape(ShapeNode {
            geometry: Geometry::Polygon(IrPolygon {
                points: vec![
                    IrPoint { x: 20.0, y: 20.0 },
                    IrPoint { x: 60.0, y: 20.0 },
                    IrPoint { x: 40.0, y: 60.0 },
                ],
            }),
            fill: Some(red),
            stroke: None,
            transform: Transform::identity(),
            opacity: 1.0,
        });

        let output = render_document(&doc).expect("render");
        assert_eq!(output.pixel(40, 30), Some([255, 0, 0, 255]));
        assert_eq!(output.pixel(40, 50), Some([255, 0, 0, 255]));
        assert_eq!(output.pixel(20, 35), Some([0, 0, 0, 0]));
        assert_eq!(output.pixel(40, 65), Some([0, 0, 0, 0]));
    }

    #[test]
    fn degenerate_polygon_is_an_error() {
        let mut doc = RenderDocument::new(20, 20);
        let red = doc.add_paint(Paint::Solid(RED));
        doc.push_shape(ShapeNode {
            geometry: Geometry::Polygon(IrPolygon {
                points: vec![IrPoint { x: 0.0, y: 0.0 }, IrPoint { x: 10.0, y: 10.0 }],
            }),
            fill: Some(red),
            stroke: None,
            transform: Transform::identity(),
            opacity: 1.0,
        });
        assert!(matches!(
            render_document(&doc).unwrap_err(),
            RenderError::InvalidGeometry {
                shape: "polygon",
                ..
            }
        ));
    }

    #[test]
    fn line_renders_with_stroke_width_and_color() {
        let mut doc = RenderDocument::new(100, 50).with_background(WHITE);
        let black = doc.add_paint(Paint::Solid(Color::black()));
        doc.push_shape(ShapeNode {
            geometry: Geometry::Line(IrLine {
                x1: 10.0,
                y1: 25.0,
                x2: 90.0,
                y2: 25.0,
            }),
            fill: None,
            stroke: Some(IrStroke {
                paint: black,
                style: StrokeStyle::new(4.0),
            }),
            transform: Transform::identity(),
            opacity: 1.0,
        });

        let output = render_document(&doc).expect("render");
        assert_eq!(output.pixel(50, 25), Some([0, 0, 0, 255]));
        assert_eq!(output.pixel(50, 20), Some([255, 255, 255, 255]));
        assert_eq!(output.pixel(50, 15), Some([255, 255, 255, 255]));
        assert_eq!(output.pixel(50, 31), Some([255, 255, 255, 255]));
    }

    #[test]
    fn polyline_renders_open_stroked_segments() {
        let mut doc = RenderDocument::new(60, 60).with_background(WHITE);
        let blue = doc.add_paint(Paint::Solid(BLUE));
        doc.push_shape(ShapeNode {
            geometry: Geometry::Polyline(IrPolyline {
                points: vec![
                    IrPoint { x: 10.0, y: 40.0 },
                    IrPoint { x: 30.0, y: 20.0 },
                    IrPoint { x: 50.0, y: 40.0 },
                ],
            }),
            fill: None,
            stroke: Some(IrStroke {
                paint: blue,
                style: StrokeStyle::new(4.0),
            }),
            transform: Transform::identity(),
            opacity: 1.0,
        });

        let output = render_document(&doc).expect("render");
        assert_eq!(output.pixel(20, 30), Some([0, 0, 255, 255]));
        assert_eq!(output.pixel(40, 30), Some([0, 0, 255, 255]));
        assert_eq!(output.pixel(30, 20), Some([0, 0, 255, 255]));
        // Open polyline: no closing segment along the bottom.
        assert_eq!(output.pixel(30, 40), Some([255, 255, 255, 255]));
    }

    #[test]
    fn stroke_missing_paint_is_an_error() {
        let mut doc = RenderDocument::new(20, 20);
        doc.push_shape(ShapeNode {
            geometry: Geometry::Line(IrLine {
                x1: 0.0,
                y1: 0.0,
                x2: 10.0,
                y2: 10.0,
            }),
            fill: None,
            stroke: Some(IrStroke {
                paint: PaintId(99),
                style: StrokeStyle::new(1.0),
            }),
            transform: Transform::identity(),
            opacity: 1.0,
        });
        assert_eq!(
            render_document(&doc).unwrap_err(),
            RenderError::MissingPaint(PaintId(99))
        );
    }

    #[test]
    fn filled_path_with_lines_renders() {
        let mut doc = RenderDocument::new(100, 100);
        let red = doc.add_paint(Paint::Solid(RED));
        doc.push_shape(ShapeNode {
            geometry: Geometry::Path(IrPath {
                commands: vec![
                    IrPathCommand::MoveTo(IrPoint { x: 20.0, y: 20.0 }),
                    IrPathCommand::LineTo(IrPoint { x: 80.0, y: 20.0 }),
                    IrPathCommand::LineTo(IrPoint { x: 80.0, y: 80.0 }),
                    IrPathCommand::Close,
                ],
            }),
            fill: Some(red),
            stroke: None,
            transform: Transform::identity(),
            opacity: 1.0,
        });

        let output = render_document(&doc).expect("render");
        assert_eq!(output.pixel(50, 40), Some([255, 0, 0, 255]));
        assert_eq!(output.pixel(10, 10), Some([0, 0, 0, 0]));
    }

    #[test]
    fn cubic_path_renders_curve() {
        let mut doc = RenderDocument::new(100, 70).with_background(WHITE);
        let black = doc.add_paint(Paint::Solid(Color::black()));
        doc.push_shape(ShapeNode {
            geometry: Geometry::Path(IrPath {
                commands: vec![
                    IrPathCommand::MoveTo(IrPoint { x: 10.0, y: 50.0 }),
                    IrPathCommand::CubicTo {
                        control1: IrPoint { x: 30.0, y: 10.0 },
                        control2: IrPoint { x: 70.0, y: 10.0 },
                        to: IrPoint { x: 90.0, y: 50.0 },
                    },
                ],
            }),
            fill: None,
            stroke: Some(IrStroke {
                paint: black,
                style: StrokeStyle::new(3.0),
            }),
            transform: Transform::identity(),
            opacity: 1.0,
        });

        let output = render_document(&doc).expect("render");
        // Midpoint of the cubic is (50, 20).
        assert_eq!(output.pixel(50, 20), Some([0, 0, 0, 255]));
        assert_eq!(output.pixel(50, 50), Some([255, 255, 255, 255]));
    }

    #[test]
    fn quadratic_path_renders_curve() {
        let mut doc = RenderDocument::new(100, 70).with_background(WHITE);
        let black = doc.add_paint(Paint::Solid(Color::black()));
        doc.push_shape(ShapeNode {
            geometry: Geometry::Path(IrPath {
                commands: vec![
                    IrPathCommand::MoveTo(IrPoint { x: 10.0, y: 50.0 }),
                    IrPathCommand::QuadTo {
                        control: IrPoint { x: 50.0, y: 10.0 },
                        to: IrPoint { x: 90.0, y: 50.0 },
                    },
                ],
            }),
            fill: None,
            stroke: Some(IrStroke {
                paint: black,
                style: StrokeStyle::new(3.0),
            }),
            transform: Transform::identity(),
            opacity: 1.0,
        });

        let output = render_document(&doc).expect("render");
        // Midpoint of the quadratic is (50, 30).
        assert_eq!(output.pixel(50, 30), Some([0, 0, 0, 255]));
        assert_eq!(output.pixel(50, 45), Some([255, 255, 255, 255]));
    }

    #[test]
    fn path_command_before_start_is_an_error() {
        let mut doc = RenderDocument::new(20, 20);
        let red = doc.add_paint(Paint::Solid(RED));
        doc.push_shape(ShapeNode {
            geometry: Geometry::Path(IrPath {
                commands: vec![IrPathCommand::LineTo(IrPoint { x: 10.0, y: 10.0 })],
            }),
            fill: Some(red),
            stroke: None,
            transform: Transform::identity(),
            opacity: 1.0,
        });
        assert!(matches!(
            render_document(&doc).unwrap_err(),
            RenderError::InvalidGeometry { shape: "path", .. }
        ));
    }

    #[test]
    fn closed_shape_fill_and_stroke_render_independently() {
        let mut doc = RenderDocument::new(60, 60).with_background(WHITE);
        let blue = doc.add_paint(Paint::Solid(BLUE));
        let black = doc.add_paint(Paint::Solid(Color::black()));
        doc.push_shape(ShapeNode {
            geometry: Geometry::Rectangle(IrRectangle {
                x: 10.0,
                y: 10.0,
                width: 40.0,
                height: 40.0,
            }),
            fill: Some(blue),
            stroke: Some(IrStroke {
                paint: black,
                style: StrokeStyle::new(4.0),
            }),
            transform: Transform::identity(),
            opacity: 1.0,
        });

        let output = render_document(&doc).expect("render");
        assert_eq!(output.pixel(30, 30), Some([0, 0, 255, 255]));
        assert_eq!(output.pixel(10, 30), Some([0, 0, 0, 255]));
        assert_eq!(output.pixel(5, 5), Some([255, 255, 255, 255]));
    }

    #[test]
    fn stroke_only_closed_shape_renders_without_fill() {
        let mut doc = RenderDocument::new(60, 60);
        let red = doc.add_paint(Paint::Solid(RED));
        doc.push_shape(ShapeNode {
            geometry: Geometry::Circle(IrCircle {
                cx: 30.0,
                cy: 30.0,
                radius: 20.0,
            }),
            fill: None,
            stroke: Some(IrStroke {
                paint: red,
                style: StrokeStyle::new(3.0),
            }),
            transform: Transform::identity(),
            opacity: 1.0,
        });

        let output = render_document(&doc).expect("render");
        assert_eq!(output.pixel(30, 10), Some([255, 0, 0, 255]));
        assert_eq!(output.pixel(30, 30), Some([0, 0, 0, 0]));
    }

    #[test]
    fn opacity_applies_to_fill_and_stroke() {
        let mut doc = RenderDocument::new(60, 60).with_background(WHITE);
        let blue = doc.add_paint(Paint::Solid(BLUE));
        let black = doc.add_paint(Paint::Solid(Color::black()));
        doc.push_shape(ShapeNode {
            geometry: Geometry::Rectangle(IrRectangle {
                x: 10.0,
                y: 10.0,
                width: 40.0,
                height: 40.0,
            }),
            fill: Some(blue),
            stroke: Some(IrStroke {
                paint: black,
                style: StrokeStyle::new(4.0),
            }),
            transform: Transform::identity(),
            opacity: 0.5,
        });

        let output = render_document(&doc).expect("render");
        assert_near(output.pixel(30, 30).unwrap(), [127, 127, 255, 255], 1);
        assert_near(output.pixel(10, 30).unwrap(), [64, 64, 127, 255], 1);
    }

    #[test]
    fn square_cap_extends_beyond_line_endpoint() {
        let mut doc = RenderDocument::new(100, 60).with_background(WHITE);
        let black = doc.add_paint(Paint::Solid(Color::black()));
        let mut style = StrokeStyle::new(10.0);
        style.cap = rupa_ir::geometry::LineCap::Square;
        doc.push_shape(ShapeNode {
            geometry: Geometry::Line(IrLine {
                x1: 30.0,
                y1: 30.0,
                x2: 70.0,
                y2: 30.0,
            }),
            fill: None,
            stroke: Some(IrStroke {
                paint: black,
                style,
            }),
            transform: Transform::identity(),
            opacity: 1.0,
        });

        let output = render_document(&doc).expect("render");
        assert_eq!(output.pixel(26, 30), Some([0, 0, 0, 255]));
        assert_eq!(output.pixel(23, 30), Some([255, 255, 255, 255]));
    }

    #[test]
    fn stroke_style_maps_cap_and_join_to_skia() {
        let mut style = StrokeStyle::new(4.0);
        style.cap = rupa_ir::geometry::LineCap::Round;
        style.join = rupa_ir::geometry::LineJoin::Bevel;
        let skia = to_skia_stroke(&style);
        assert_eq!(skia.line_cap, tiny_skia::LineCap::Round);
        assert_eq!(skia.line_join, tiny_skia::LineJoin::Bevel);
        assert_eq!(skia.width, 4.0);
    }

    fn group_definition(name: &str) -> rupa_ir::document::GroupDefinition {
        rupa_ir::document::GroupDefinition {
            name: name.to_string(),
            bounds: rupa_ir::document::Bounds {
                x: 0.0,
                y: 0.0,
                width: 20.0,
                height: 20.0,
            },
            clip: true,
            children: vec![CompositionChild::Shape(ShapeNode {
                geometry: Geometry::Rectangle(IrRectangle {
                    x: 0.0,
                    y: 0.0,
                    width: 20.0,
                    height: 20.0,
                }),
                fill: Some(PaintId(0)),
                stroke: None,
                transform: Transform::identity(),
                opacity: 1.0,
            })],
        }
    }

    fn layer_instance(name: &str, x: f32, y: f32) -> rupa_ir::document::GroupInstance {
        rupa_ir::document::GroupInstance {
            group: name.to_string(),
            x,
            y,
            transform: Transform::identity(),
            opacity: 1.0,
            z_index: 0,
            blend_mode: rupa_ir::BlendMode::Normal,
        }
    }

    #[test]
    fn multiple_layer_instances_render_independently() {
        let mut doc = RenderDocument::new(100, 50).with_background(WHITE);
        doc.add_paint(Paint::Solid(RED));
        doc.push_group(group_definition("Dot"));
        doc.push_instance(layer_instance("Dot", 10.0, 10.0));
        doc.push_instance(layer_instance("Dot", 50.0, 10.0));

        let output = render_document(&doc).expect("render");
        assert_eq!(output.pixel(15, 15), Some([255, 0, 0, 255]));
        assert_eq!(output.pixel(55, 15), Some([255, 0, 0, 255]));
        assert_eq!(output.pixel(35, 15), Some([255, 255, 255, 255]));
    }

    #[test]
    fn instance_scale_pivots_on_measured_group_bounds() {
        let mut doc = RenderDocument::new(100, 100).with_background(WHITE);
        let red = doc.add_paint(Paint::Solid(RED));
        doc.push_group(rupa_ir::document::GroupDefinition {
            name: "Offset".to_string(),
            bounds: rupa_ir::document::Bounds {
                x: 5.0,
                y: 5.0,
                width: 10.0,
                height: 10.0,
            },
            clip: false,
            children: vec![CompositionChild::Shape(rect_shape(
                5.0, 5.0, 10.0, 10.0, red,
            ))],
        });
        let mut instance = layer_instance("Offset", 40.0, 40.0);
        instance.transform = Transform::scale(2.0, 2.0);
        doc.push_instance(instance);

        let output = render_document(&doc).expect("render");
        // The measured bounds center (10,10) stays fixed under scaling.
        assert_eq!(output.pixel(45, 45), Some([255, 0, 0, 255]));
        assert_eq!(output.pixel(41, 41), Some([255, 0, 0, 255]));
        assert_eq!(output.pixel(38, 38), Some([255, 255, 255, 255]));
    }

    #[test]
    fn instance_scale_pivots_on_layer_center() {
        let mut doc = RenderDocument::new(100, 100).with_background(WHITE);
        doc.add_paint(Paint::Solid(RED));
        doc.push_group(group_definition("Dot"));
        let mut instance = layer_instance("Dot", 10.0, 10.0);
        instance.transform = Transform::scale(0.5, 0.5);
        doc.push_instance(instance);

        let output = render_document(&doc).expect("render");
        // Local 20x20 scaled to 10x10 around local center (10,10), then placed
        // at (10,10): bounds (15,15)-(25,25).
        assert_eq!(output.pixel(20, 20), Some([255, 0, 0, 255]));
        assert_eq!(output.pixel(13, 13), Some([255, 255, 255, 255]));
        assert_eq!(output.pixel(27, 27), Some([255, 255, 255, 255]));
    }

    #[test]
    fn instance_opacity_multiplies_child_opacity() {
        let mut doc = RenderDocument::new(100, 100).with_background(WHITE);
        let paint = doc.add_paint(Paint::Solid(RED));
        let mut layer = group_definition("Dot");
        if let CompositionChild::Shape(shape) = &mut layer.children[0] {
            shape.fill = Some(paint);
            shape.opacity = 1.0;
        }
        doc.push_group(layer);
        let mut instance = layer_instance("Dot", 10.0, 10.0);
        instance.opacity = 0.5;
        doc.push_instance(instance);

        let output = render_document(&doc).expect("render");
        assert_near(output.pixel(20, 20).unwrap(), [255, 127, 127, 255], 1);
    }

    fn solid_layer(name: &str, paint: PaintId) -> rupa_ir::document::GroupDefinition {
        rupa_ir::document::GroupDefinition {
            name: name.to_string(),
            bounds: rupa_ir::document::Bounds {
                x: 0.0,
                y: 0.0,
                width: 20.0,
                height: 20.0,
            },
            clip: true,
            children: vec![CompositionChild::Shape(ShapeNode {
                geometry: Geometry::Rectangle(IrRectangle {
                    x: 0.0,
                    y: 0.0,
                    width: 20.0,
                    height: 20.0,
                }),
                fill: Some(paint),
                stroke: None,
                transform: Transform::identity(),
                opacity: 1.0,
            })],
        }
    }

    #[test]
    fn higher_z_index_renders_in_front_regardless_of_source_order() {
        let mut doc = RenderDocument::new(40, 40).with_background(WHITE);
        let red = doc.add_paint(Paint::Solid(RED));
        let blue = doc.add_paint(Paint::Solid(BLUE));
        doc.push_group(solid_layer("Red", red));
        doc.push_group(solid_layer("Blue", blue));

        // The red instance is declared first but has the higher z-index.
        let mut high = layer_instance("Red", 0.0, 0.0);
        high.z_index = 10;
        doc.push_instance(high);
        let mut low = layer_instance("Blue", 0.0, 0.0);
        low.z_index = 0;
        doc.push_instance(low);

        let output = render_document(&doc).expect("render");
        assert_eq!(output.pixel(10, 10), Some([255, 0, 0, 255]));
    }

    #[test]
    fn equal_z_index_renders_in_source_order() {
        let mut doc = RenderDocument::new(40, 40).with_background(WHITE);
        let red = doc.add_paint(Paint::Solid(RED));
        let blue = doc.add_paint(Paint::Solid(BLUE));
        doc.push_group(solid_layer("Red", red));
        doc.push_group(solid_layer("Blue", blue));

        let mut first = layer_instance("Red", 0.0, 0.0);
        first.z_index = 3;
        doc.push_instance(first);
        let mut second = layer_instance("Blue", 0.0, 0.0);
        second.z_index = 3;
        doc.push_instance(second);

        let output = render_document(&doc).expect("render");
        assert_eq!(output.pixel(10, 10), Some([0, 0, 255, 255]));
    }

    #[test]
    fn negative_z_index_instance_renders_behind_root_shape() {
        let mut doc = RenderDocument::new(40, 40).with_background(WHITE);
        let red = doc.add_paint(Paint::Solid(RED));
        let blue = doc.add_paint(Paint::Solid(BLUE));
        doc.push_group(solid_layer("Below", blue));

        let mut below = layer_instance("Below", 0.0, 0.0);
        below.z_index = -1;
        doc.push_instance(below);
        // Direct root shapes participate at z-index 0, even when declared
        // after the instance.
        doc.push_shape(rect_shape(0.0, 0.0, 20.0, 20.0, red));

        let output = render_document(&doc).expect("render");
        assert_eq!(output.pixel(10, 10), Some([255, 0, 0, 255]));
    }

    #[test]
    fn positive_z_index_instance_renders_in_front_of_later_root_shape() {
        let mut doc = RenderDocument::new(40, 40).with_background(WHITE);
        let red = doc.add_paint(Paint::Solid(RED));
        let blue = doc.add_paint(Paint::Solid(BLUE));
        doc.push_group(solid_layer("Above", blue));

        let mut above = layer_instance("Above", 0.0, 0.0);
        above.z_index = 5;
        doc.push_instance(above);
        doc.push_shape(rect_shape(0.0, 0.0, 20.0, 20.0, red));

        let output = render_document(&doc).expect("render");
        assert_eq!(output.pixel(10, 10), Some([0, 0, 255, 255]));
    }

    fn outer_layer(
        name: &str,
        children: Vec<CompositionChild>,
    ) -> rupa_ir::document::GroupDefinition {
        rupa_ir::document::GroupDefinition {
            name: name.to_string(),
            bounds: rupa_ir::document::Bounds {
                x: 0.0,
                y: 0.0,
                width: 40.0,
                height: 40.0,
            },
            clip: true,
            children,
        }
    }

    fn sized_layer(
        name: &str,
        width: u32,
        height: u32,
        children: Vec<CompositionChild>,
    ) -> rupa_ir::document::GroupDefinition {
        rupa_ir::document::GroupDefinition {
            name: name.to_string(),
            bounds: rupa_ir::document::Bounds {
                x: 0.0,
                y: 0.0,
                width: width as f32,
                height: height as f32,
            },
            clip: true,
            children,
        }
    }

    #[test]
    fn layer_content_is_clipped_to_declared_bounds() {
        let mut doc = RenderDocument::new(100, 100).with_background(WHITE);
        let red = doc.add_paint(Paint::Solid(RED));
        // The rectangle is four times wider than the 20px layer.
        let wide = sized_layer(
            "Wide",
            20,
            20,
            vec![CompositionChild::Shape(rect_shape(
                0.0, 0.0, 80.0, 20.0, red,
            ))],
        );
        doc.push_group(wide);
        doc.push_instance(layer_instance("Wide", 10.0, 10.0));

        let output = render_document(&doc).expect("render");
        assert_eq!(output.pixel(15, 15), Some([255, 0, 0, 255]));
        assert_eq!(output.pixel(29, 15), Some([255, 0, 0, 255]));
        // Past the layer's local x = 20 bound (device x = 30).
        assert_eq!(output.pixel(31, 15), Some([255, 255, 255, 255]));
        assert_eq!(output.pixel(9, 15), Some([255, 255, 255, 255]));
    }

    #[test]
    fn layer_strokes_are_clipped_to_declared_bounds() {
        let mut doc = RenderDocument::new(100, 60).with_background(WHITE);
        let black = doc.add_paint(Paint::Solid(Color::black()));
        let line = sized_layer(
            "Line",
            20,
            20,
            vec![CompositionChild::Shape(ShapeNode {
                geometry: Geometry::Line(IrLine {
                    x1: -10.0,
                    y1: 10.0,
                    x2: 30.0,
                    y2: 10.0,
                }),
                fill: None,
                stroke: Some(IrStroke {
                    paint: black,
                    style: StrokeStyle::new(4.0),
                }),
                transform: Transform::identity(),
                opacity: 1.0,
            })],
        );
        doc.push_group(line);
        doc.push_instance(layer_instance("Line", 10.0, 10.0));

        let output = render_document(&doc).expect("render");
        // Local y = 10 maps to device y = 20; the line spans the full layer.
        assert_eq!(output.pixel(12, 20), Some([0, 0, 0, 255]));
        assert_eq!(output.pixel(29, 20), Some([0, 0, 0, 255]));
        assert_eq!(output.pixel(32, 20), Some([255, 255, 255, 255]));
        assert_eq!(output.pixel(7, 20), Some([255, 255, 255, 255]));
    }

    #[test]
    fn rotated_instance_clips_transformed_bounds_not_aabb() {
        let mut doc = RenderDocument::new(100, 100).with_background(WHITE);
        let red = doc.add_paint(Paint::Solid(RED));
        doc.push_group(sized_layer(
            "Square",
            20,
            20,
            vec![CompositionChild::Shape(rect_shape(
                0.0, 0.0, 20.0, 20.0, red,
            ))],
        ));
        let mut instance = layer_instance("Square", 40.0, 40.0);
        instance.transform = Transform::rotate(std::f32::consts::FRAC_PI_4);
        doc.push_instance(instance);

        let output = render_document(&doc).expect("render");
        // Center is (50,50); the diamond stays inside its transformed bounds.
        assert_eq!(output.pixel(50, 42), Some([255, 0, 0, 255]));
        // Inside the rotated square's AABB but outside the rotated square.
        assert_eq!(output.pixel(40, 40), Some([255, 255, 255, 255]));
        assert_eq!(output.pixel(37, 37), Some([255, 255, 255, 255]));
    }

    #[test]
    fn nested_content_is_clipped_to_enclosing_layer() {
        let mut doc = RenderDocument::new(120, 120).with_background(WHITE);
        let red = doc.add_paint(Paint::Solid(RED));
        // Inner is larger than Outer and placed near its bottom-right corner.
        let inner = sized_layer(
            "Inner",
            40,
            40,
            vec![CompositionChild::Shape(rect_shape(
                0.0, 0.0, 40.0, 40.0, red,
            ))],
        );
        let outer = sized_layer(
            "Outer",
            40,
            40,
            vec![CompositionChild::Instance(layer_instance(
                "Inner", 30.0, 30.0,
            ))],
        );
        doc.push_group(inner);
        doc.push_group(outer);
        doc.push_instance(layer_instance("Outer", 10.0, 10.0));

        let output = render_document(&doc).expect("render");
        // Outer spans device (10,10)-(50,50); Inner's visible corner is
        // (40,40)-(50,50).
        assert_eq!(output.pixel(45, 45), Some([255, 0, 0, 255]));
        assert_eq!(output.pixel(35, 35), Some([255, 255, 255, 255]));
        // Beyond Outer's bounds even though Inner reaches further.
        assert_eq!(output.pixel(52, 52), Some([255, 255, 255, 255]));
    }

    #[test]
    fn sibling_instances_have_independent_clip_regions() {
        let mut doc = RenderDocument::new(140, 60).with_background(WHITE);
        let red = doc.add_paint(Paint::Solid(RED));
        doc.push_group(sized_layer(
            "Wide",
            20,
            20,
            vec![CompositionChild::Shape(rect_shape(
                0.0, 0.0, 60.0, 20.0, red,
            ))],
        ));
        doc.push_instance(layer_instance("Wide", 10.0, 10.0));
        doc.push_instance(layer_instance("Wide", 70.0, 10.0));

        let output = render_document(&doc).expect("render");
        assert_eq!(output.pixel(15, 15), Some([255, 0, 0, 255]));
        assert_eq!(output.pixel(31, 15), Some([255, 255, 255, 255]));
        assert_eq!(output.pixel(75, 15), Some([255, 0, 0, 255]));
        assert_eq!(output.pixel(91, 15), Some([255, 255, 255, 255]));
    }

    #[test]
    fn nested_instance_renders_in_parent_local_space() {
        let mut doc = RenderDocument::new(100, 100).with_background(WHITE);
        let red = doc.add_paint(Paint::Solid(RED));
        // `Inner` is defined after `Outer`: forward references work when nested.
        let outer = outer_layer(
            "Outer",
            vec![CompositionChild::Instance(layer_instance(
                "Inner", 5.0, 5.0,
            ))],
        );
        doc.push_group(outer);
        doc.push_group(solid_layer("Inner", red));
        doc.push_instance(layer_instance("Outer", 30.0, 40.0));

        let output = render_document(&doc).expect("render");
        // Inner 20x20 at Outer-local (5,5) -> document (35,45)-(55,65).
        assert_eq!(output.pixel(40, 50), Some([255, 0, 0, 255]));
        assert_eq!(output.pixel(33, 43), Some([255, 255, 255, 255]));
    }

    #[test]
    fn nested_instance_transforms_compose() {
        let mut doc = RenderDocument::new(100, 100).with_background(WHITE);
        let red = doc.add_paint(Paint::Solid(RED));
        let outer = outer_layer(
            "Outer",
            vec![CompositionChild::Instance(layer_instance(
                "Inner", 5.0, 5.0,
            ))],
        );
        doc.push_group(outer);
        doc.push_group(solid_layer("Inner", red));

        let mut outer_instance = layer_instance("Outer", 10.0, 10.0);
        outer_instance.transform = Transform::scale(2.0, 2.0);
        doc.push_instance(outer_instance);

        let output = render_document(&doc).expect("render");
        // Outer-local (5,5)-(25,25) scaled about Outer center (20,20) then
        // placed at (10,10) -> document (0,0)-(40,40).
        assert_eq!(output.pixel(20, 20), Some([255, 0, 0, 255]));
        assert_eq!(output.pixel(5, 5), Some([255, 0, 0, 255]));
        assert_eq!(output.pixel(42, 42), Some([255, 255, 255, 255]));
    }

    #[test]
    fn nested_z_index_orders_siblings_within_parent() {
        let mut doc = RenderDocument::new(60, 60).with_background(WHITE);
        let red = doc.add_paint(Paint::Solid(RED));
        let blue = doc.add_paint(Paint::Solid(BLUE));

        let mut high = layer_instance("Red", 0.0, 0.0);
        high.z_index = 10;
        let mut low = layer_instance("Blue", 0.0, 0.0);
        low.z_index = 0;
        // Red is declared first but has the higher z-index within Outer.
        let outer = outer_layer(
            "Outer",
            vec![
                CompositionChild::Instance(high),
                CompositionChild::Instance(low),
            ],
        );
        doc.push_group(outer);
        doc.push_group(solid_layer("Red", red));
        doc.push_group(solid_layer("Blue", blue));
        doc.push_instance(layer_instance("Outer", 10.0, 10.0));

        let output = render_document(&doc).expect("render");
        assert_eq!(output.pixel(20, 20), Some([255, 0, 0, 255]));
    }

    #[test]
    fn nested_instance_opacity_multiplies_through_hierarchy() {
        let mut doc = RenderDocument::new(60, 60).with_background(WHITE);
        let red = doc.add_paint(Paint::Solid(RED));
        let mut nested = layer_instance("Inner", 0.0, 0.0);
        nested.opacity = 0.5;
        let outer = outer_layer("Outer", vec![CompositionChild::Instance(nested)]);
        doc.push_group(outer);
        doc.push_group(solid_layer("Inner", red));

        let mut outer_instance = layer_instance("Outer", 10.0, 10.0);
        outer_instance.opacity = 0.5;
        doc.push_instance(outer_instance);

        let output = render_document(&doc).expect("render");
        // 0.5 * 0.5 red over white -> (255, 191, 191).
        assert_near(output.pixel(20, 20).unwrap(), [255, 191, 191, 255], 2);
    }

    #[test]
    fn nested_definition_instances_render_independently() {
        let mut doc = RenderDocument::new(100, 100).with_background(WHITE);
        let red = doc.add_paint(Paint::Solid(RED));
        let outer = outer_layer(
            "Outer",
            vec![CompositionChild::Instance(layer_instance(
                "Inner", 0.0, 0.0,
            ))],
        );
        doc.push_group(outer);
        doc.push_group(solid_layer("Inner", red));
        doc.push_instance(layer_instance("Outer", 10.0, 10.0));
        doc.push_instance(layer_instance("Outer", 60.0, 10.0));

        let output = render_document(&doc).expect("render");
        assert_eq!(output.pixel(20, 20), Some([255, 0, 0, 255]));
        assert_eq!(output.pixel(70, 20), Some([255, 0, 0, 255]));
        assert_eq!(output.pixel(45, 20), Some([255, 255, 255, 255]));
    }

    #[test]
    fn cyclic_layer_reference_is_a_render_error() {
        let mut doc = RenderDocument::new(20, 20);
        let a = outer_layer(
            "A",
            vec![CompositionChild::Instance(layer_instance("B", 0.0, 0.0))],
        );
        let b = outer_layer(
            "B",
            vec![CompositionChild::Instance(layer_instance("A", 0.0, 0.0))],
        );
        doc.push_group(a);
        doc.push_group(b);
        doc.push_instance(layer_instance("A", 0.0, 0.0));

        assert_eq!(
            render_document(&doc).unwrap_err(),
            RenderError::CyclicGroup("A".to_string())
        );
    }

    #[test]
    fn missing_group_definition_is_an_error() {
        let mut doc = RenderDocument::new(20, 20);
        doc.push_instance(layer_instance("Missing", 0.0, 0.0));
        assert_eq!(
            render_document(&doc).unwrap_err(),
            RenderError::MissingGroup("Missing".to_string())
        );
    }

    #[test]
    fn child_transform_composes_with_instance_transform() {
        let mut doc = RenderDocument::new(100, 100).with_background(WHITE);
        let red = doc.add_paint(Paint::Solid(RED));
        let layer = rupa_ir::document::GroupDefinition {
            name: "Rotated".to_string(),
            bounds: rupa_ir::document::Bounds {
                x: 0.0,
                y: 0.0,
                width: 20.0,
                height: 20.0,
            },
            clip: true,
            children: vec![CompositionChild::Shape(ShapeNode {
                geometry: Geometry::Rectangle(IrRectangle {
                    x: 0.0,
                    y: 0.0,
                    width: 20.0,
                    height: 10.0,
                }),
                fill: Some(red),
                stroke: None,
                transform: Transform::rotate(std::f32::consts::FRAC_PI_2),
                opacity: 1.0,
            })],
        };
        doc.push_group(layer);
        doc.push_instance(layer_instance("Rotated", 10.0, 10.0));

        let output = render_document(&doc).expect("render");
        // Local rect center (10,5), rotated 90deg -> local bounds (5,-5)-(15,15),
        // then translated by the instance to (15,5)-(25,25).
        assert_eq!(output.pixel(20, 10), Some([255, 0, 0, 255]));
        assert_eq!(output.pixel(20, 2), Some([255, 255, 255, 255]));
    }

    #[test]
    fn nested_translations_compose() {
        let mut doc = document(100, 40);
        let red = doc.add_paint(Paint::Solid(RED));
        let mut shape = rect_shape(0.0, 0.0, 10.0, 10.0, red);
        shape.transform =
            Transform::translate(30.0, 0.0).multiply(&Transform::translate(10.0, 0.0));
        doc.push_shape(shape);

        let output = render_document(&doc).expect("render");
        assert_eq!(output.pixel(45, 5), Some([255, 0, 0, 255]));
    }

    #[test]
    fn opacity_blends_with_background() {
        let mut doc = document(20, 20);
        let red = doc.add_paint(Paint::Solid(RED));
        let mut shape = rect_shape(0.0, 0.0, 20.0, 20.0, red);
        shape.opacity = 0.5;
        doc.push_shape(shape);

        let output = render_document(&doc).expect("render");
        assert_near(output.pixel(10, 10).unwrap(), [255, 128, 128, 255], 1);
    }

    #[test]
    fn zero_opacity_draws_nothing() {
        let mut doc = document(20, 20);
        let red = doc.add_paint(Paint::Solid(RED));
        let mut shape = rect_shape(0.0, 0.0, 20.0, 20.0, red);
        shape.opacity = 0.0;
        doc.push_shape(shape);

        let output = render_document(&doc).expect("render");
        assert_eq!(output.pixel(10, 10), Some([255, 255, 255, 255]));
    }

    #[test]
    fn brightness_filter_scales_rgb_and_preserves_alpha() {
        let mut pixmap = Pixmap::new(1, 1).expect("pixmap");
        pixmap.data_mut().copy_from_slice(&[100, 150, 200, 77]);

        apply_filters(&mut pixmap, &[Filter::Brightness(2.0)]);

        assert_eq!(pixmap.data(), &[200, 255, 255, 77]);
    }

    #[test]
    fn grayscale_filter_uses_luma_weights_and_preserves_alpha() {
        let mut pixmap = Pixmap::new(1, 1).expect("pixmap");
        pixmap.data_mut().copy_from_slice(&[100, 150, 200, 128]);
        let expected = (0.299f32 * 100.0 + 0.587 * 150.0 + 0.114 * 200.0).round() as u8;

        apply_filters(&mut pixmap, &[Filter::Grayscale]);

        assert_eq!(pixmap.data(), &[expected, expected, expected, 128]);
    }

    #[test]
    fn shapes_render_in_source_order() {
        let mut doc = document(20, 20);
        let red = doc.add_paint(Paint::Solid(RED));
        let blue = doc.add_paint(Paint::Solid(BLUE));
        doc.push_shape(rect_shape(0.0, 0.0, 20.0, 20.0, red));
        doc.push_shape(rect_shape(0.0, 0.0, 20.0, 20.0, blue));

        let output = render_document(&doc).expect("render");
        assert_eq!(output.pixel(10, 10), Some([0, 0, 255, 255]));
    }

    #[test]
    fn unfilled_shape_draws_nothing() {
        let mut doc = document(20, 20);
        let mut shape = rect_shape(0.0, 0.0, 20.0, 20.0, PaintId(0));
        shape.fill = None;
        doc.push_shape(shape);

        let output = render_document(&doc).expect("render");
        assert_eq!(output.pixel(10, 10), Some([255, 255, 255, 255]));
    }

    #[test]
    fn missing_paint_is_an_error() {
        let mut doc = document(20, 20);
        doc.push_shape(rect_shape(0.0, 0.0, 20.0, 20.0, PaintId(42)));
        assert_eq!(
            render_document(&doc).unwrap_err(),
            RenderError::MissingPaint(PaintId(42))
        );
    }

    #[test]
    fn invalid_geometry_is_an_error() {
        let mut doc = document(20, 20);
        let red = doc.add_paint(Paint::Solid(RED));
        doc.push_shape(rect_shape(0.0, 0.0, -1.0, 20.0, red));
        assert!(matches!(
            render_document(&doc).unwrap_err(),
            RenderError::InvalidGeometry { .. }
        ));
    }

    #[test]
    fn transform_about_origin_keeps_origin_fixed() {
        let transform = transform_about_origin(&Transform::rotate(0.5), (10.0, 20.0));
        let (x, y) = transform.transform_point(10.0, 20.0);
        assert!((x - 10.0).abs() < 1e-5);
        assert!((y - 20.0).abs() < 1e-5);
    }

    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_PICTURE_DIR: AtomicU64 = AtomicU64::new(0);

    /// Creates a temp directory holding a 2x1 image: red left, blue right.
    fn striped_store() -> (ResourceStore, PathBuf) {
        let id = NEXT_PICTURE_DIR.fetch_add(1, Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("rupa-renderer-picture-{}-{id}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let pixels = [255u8, 0, 0, 255, 0, 0, 255, 255];
        rupa_image::encode_rgba_png(&pixels, 2, 1, &dir.join("strip.png")).expect("encode PNG");
        (ResourceStore::new(&dir), dir)
    }

    fn picture_node(
        path: &str,
        x: f32,
        y: f32,
        size: Option<(f32, f32)>,
        fit: FitMode,
    ) -> PictureNode {
        PictureNode {
            path: path.to_string(),
            x,
            y,
            width: size.map(|(width, _)| width),
            height: size.map(|(_, height)| height),
            fit,
            transform: Transform::identity(),
            opacity: 1.0,
        }
    }

    fn picture_document(picture: PictureNode) -> RenderDocument {
        let mut doc = RenderDocument::new(60, 60).with_background(WHITE);
        doc.push_picture(picture);
        doc
    }

    #[test]
    fn picture_fill_stretches_image_to_frame() {
        let (mut store, dir) = striped_store();
        let doc = picture_document(picture_node(
            "strip.png",
            10.0,
            10.0,
            Some((20.0, 20.0)),
            FitMode::Fill,
        ));

        let output = render_document_with_resources(&doc, &mut store).expect("render");
        assert_eq!(output.pixel(15, 15), Some([255, 0, 0, 255]));
        assert_eq!(output.pixel(25, 15), Some([0, 0, 255, 255]));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn picture_contain_letterboxes_and_centers() {
        let (mut store, dir) = striped_store();
        let doc = picture_document(picture_node(
            "strip.png",
            10.0,
            10.0,
            Some((20.0, 20.0)),
            FitMode::Contain,
        ));

        let output = render_document_with_resources(&doc, &mut store).expect("render");
        // 2x1 scaled to 20x10, centered vertically at y = 15..25.
        assert_eq!(output.pixel(15, 20), Some([255, 0, 0, 255]));
        assert_eq!(output.pixel(15, 12), Some([255, 255, 255, 255]));
        assert_eq!(output.pixel(15, 27), Some([255, 255, 255, 255]));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn picture_cover_crops_to_frame() {
        let (mut store, dir) = striped_store();
        let doc = picture_document(picture_node(
            "strip.png",
            10.0,
            10.0,
            Some((20.0, 20.0)),
            FitMode::Cover,
        ));

        let output = render_document_with_resources(&doc, &mut store).expect("render");
        // 2x1 scaled to 40x20; the frame shows the middle of the image.
        assert_eq!(output.pixel(14, 20), Some([255, 0, 0, 255]));
        assert_eq!(output.pixel(25, 20), Some([0, 0, 255, 255]));
        assert_eq!(output.pixel(9, 20), Some([255, 255, 255, 255]));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn picture_none_uses_natural_size() {
        let (mut store, dir) = striped_store();
        let doc = picture_document(picture_node("strip.png", 10.0, 10.0, None, FitMode::None));

        let output = render_document_with_resources(&doc, &mut store).expect("render");
        assert_eq!(output.pixel(10, 10), Some([255, 0, 0, 255]));
        assert_eq!(output.pixel(11, 10), Some([0, 0, 255, 255]));
        assert_eq!(output.pixel(12, 10), Some([255, 255, 255, 255]));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn picture_is_clipped_by_enclosing_group() {
        let (mut store, dir) = striped_store();
        let mut doc = RenderDocument::new(60, 60).with_background(WHITE);
        let mut layer = rupa_ir::document::GroupDefinition {
            name: "Crop".to_string(),
            bounds: rupa_ir::document::Bounds {
                x: 0.0,
                y: 0.0,
                width: 10.0,
                height: 10.0,
            },
            clip: true,
            children: Vec::new(),
        };
        layer.children.push(CompositionChild::Picture(picture_node(
            "strip.png",
            0.0,
            0.0,
            Some((50.0, 50.0)),
            FitMode::Fill,
        )));
        doc.push_group(layer);
        doc.push_instance(layer_instance("Crop", 10.0, 10.0));

        let output = render_document_with_resources(&doc, &mut store).expect("render");
        assert_eq!(output.pixel(12, 12), Some([255, 0, 0, 255]));
        assert_eq!(output.pixel(25, 25), Some([255, 255, 255, 255]));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn picture_without_resource_store_is_an_error() {
        let doc = picture_document(picture_node("strip.png", 0.0, 0.0, None, FitMode::Contain));

        assert_eq!(
            render_document(&doc).unwrap_err(),
            RenderError::MissingResource("strip.png".to_string())
        );
    }

    #[test]
    fn missing_picture_resource_reports_path() {
        let (mut store, dir) = striped_store();
        let doc = picture_document(picture_node(
            "missing.png",
            0.0,
            0.0,
            None,
            FitMode::Contain,
        ));

        let error = render_document_with_resources(&doc, &mut store).expect_err("must fail");
        match error {
            RenderError::Resource { path, .. } => assert_eq!(path, "missing.png"),
            other => panic!("unexpected error: {other}"),
        }
        let _ = std::fs::remove_dir_all(dir);
    }
}

/// Convert an IR blend mode to tiny-skia's equivalent.
fn to_skia_blend_mode(mode: rupa_ir::BlendMode) -> tiny_skia::BlendMode {
    match mode {
        rupa_ir::BlendMode::Normal => tiny_skia::BlendMode::SourceOver,
        rupa_ir::BlendMode::Multiply => tiny_skia::BlendMode::Multiply,
        rupa_ir::BlendMode::Screen => tiny_skia::BlendMode::Screen,
        rupa_ir::BlendMode::Overlay => tiny_skia::BlendMode::Overlay,
        rupa_ir::BlendMode::Darken => tiny_skia::BlendMode::Darken,
        rupa_ir::BlendMode::Lighten => tiny_skia::BlendMode::Lighten,
        rupa_ir::BlendMode::ColorDodge => tiny_skia::BlendMode::ColorDodge,
        rupa_ir::BlendMode::ColorBurn => tiny_skia::BlendMode::ColorBurn,
        rupa_ir::BlendMode::SoftLight => tiny_skia::BlendMode::SoftLight,
        rupa_ir::BlendMode::HardLight => tiny_skia::BlendMode::HardLight,
        rupa_ir::BlendMode::Difference => tiny_skia::BlendMode::Difference,
        rupa_ir::BlendMode::Exclusion => tiny_skia::BlendMode::Exclusion,
    }
}

/// Apply an RGB operation to every pixel while leaving alpha untouched.
fn for_each_rgb(pixmap: &mut Pixmap, mut apply: impl FnMut(u32, u32, &mut [u8])) {
    let width = pixmap.width();
    if width == 0 {
        return;
    }
    for (index, pixel) in pixmap
        .data_mut()
        .as_chunks_mut::<4>()
        .0
        .iter_mut()
        .enumerate()
    {
        let index = index as u32;
        apply(index % width, index / width, &mut pixel[..3]);
    }
}

/// Apply the ordered filter effects to a rendered layer pixmap in place.
fn apply_filters(pixmap: &mut Pixmap, filters: &[Filter]) {
    for filter in filters {
        match filter {
            Filter::Blur(radius) => blur_pixmap(pixmap, *radius),
            Filter::Brightness(value) => brightness_pixmap(pixmap, *value),
            Filter::Contrast(value) => contrast_pixmap(pixmap, *value),
            Filter::Saturation(value) => saturation_pixmap(pixmap, *value),
            Filter::Grayscale => grayscale_pixmap(pixmap),
            Filter::HueRotate(value) => hue_rotate_pixmap(pixmap, *value),
            Filter::Sharpen(value) => sharpen_pixmap(pixmap, *value),
            Filter::Noise(value) => noise_pixmap(pixmap, *value),
            Filter::Pixelate(value) => pixelate_pixmap(pixmap, *value),
            Filter::Vignette(value) => vignette_pixmap(pixmap, *value),
        }
    }
}

fn brightness_pixmap(pixmap: &mut Pixmap, value: f32) {
    for_each_rgb(pixmap, |_, _, rgb| {
        for slot in rgb {
            *slot = (*slot as f32 * value).clamp(0.0, 255.0) as u8;
        }
    });
}

fn contrast_pixmap(pixmap: &mut Pixmap, value: f32) {
    for_each_rgb(pixmap, |_, _, rgb| {
        for slot in rgb {
            let normalized = *slot as f32 / 255.0;
            let adjusted = ((normalized - 0.5) * value + 0.5).clamp(0.0, 1.0);
            *slot = (adjusted * 255.0).round() as u8;
        }
    });
}

fn saturation_pixmap(pixmap: &mut Pixmap, value: f32) {
    for_each_rgb(pixmap, |_, _, rgb| {
        let r = rgb[0] as f32;
        let g = rgb[1] as f32;
        let b = rgb[2] as f32;
        let gray = 0.299 * r + 0.587 * g + 0.114 * b;
        rgb[0] = (gray + (r - gray) * value).clamp(0.0, 255.0) as u8;
        rgb[1] = (gray + (g - gray) * value).clamp(0.0, 255.0) as u8;
        rgb[2] = (gray + (b - gray) * value).clamp(0.0, 255.0) as u8;
    });
}

fn grayscale_pixmap(pixmap: &mut Pixmap) {
    for_each_rgb(pixmap, |_, _, rgb| {
        let gray =
            (0.299 * rgb[0] as f32 + 0.587 * rgb[1] as f32 + 0.114 * rgb[2] as f32).round() as u8;
        rgb.fill(gray);
    });
}

fn hue_rotate_pixmap(pixmap: &mut Pixmap, degrees: f32) {
    let radians = degrees.to_radians();
    let cos = radians.cos();
    let sin = radians.sin();
    let matrix = [
        [
            0.299 + 0.701 * cos + 0.168 * sin,
            0.587 - 0.587 * cos + 0.330 * sin,
            0.114 - 0.114 * cos - 0.497 * sin,
        ],
        [
            0.299 - 0.299 * cos - 0.328 * sin,
            0.587 + 0.413 * cos + 0.035 * sin,
            0.114 - 0.114 * cos + 0.292 * sin,
        ],
        [
            0.299 - 0.300 * cos + 1.250 * sin,
            0.587 - 0.588 * cos - 1.050 * sin,
            0.114 + 0.886 * cos - 0.203 * sin,
        ],
    ];
    for_each_rgb(pixmap, |_, _, rgb| {
        let r = rgb[0] as f32;
        let g = rgb[1] as f32;
        let b = rgb[2] as f32;
        let nr = (matrix[0][0] * r + matrix[0][1] * g + matrix[0][2] * b).clamp(0.0, 255.0);
        let ng = (matrix[1][0] * r + matrix[1][1] * g + matrix[1][2] * b).clamp(0.0, 255.0);
        let nb = (matrix[2][0] * r + matrix[2][1] * g + matrix[2][2] * b).clamp(0.0, 255.0);
        rgb[0] = nr as u8;
        rgb[1] = ng as u8;
        rgb[2] = nb as u8;
    });
}

fn sharpen_pixmap(pixmap: &mut Pixmap, value: f32) {
    let w = pixmap.width() as usize;
    let h = pixmap.height() as usize;
    if w == 0 || h == 0 {
        return;
    }
    let src = pixmap.data().to_vec();
    let mut out = src.clone();
    let kernel: [[f32; 3]; 3] = [[0.0, -1.0, 0.0], [-1.0, 5.0, -1.0], [0.0, -1.0, 0.0]];
    for y in 1..h.saturating_sub(1) {
        for x in 1..w.saturating_sub(1) {
            for channel in [0usize, 1, 2] {
                let mut acc = 0.0f32;
                for (ky, kernel_row) in kernel.iter().enumerate() {
                    for (kx, kernel_val) in kernel_row.iter().enumerate() {
                        let sx = x as i32 + kx as i32 - 1;
                        let sy = y as i32 + ky as i32 - 1;
                        let idx = (sy as usize * w + sx as usize) * 4 + channel;
                        acc += src[idx] as f32 * kernel_val;
                    }
                }
                let base = src[(y * w + x) * 4 + channel] as f32;
                let sharpened = base + (acc - base) * (value - 1.0).max(0.0);
                out[(y * w + x) * 4 + channel] = sharpened.clamp(0.0, 255.0) as u8;
            }
        }
    }
    pixmap.data_mut().copy_from_slice(&out);
}

fn noise_pixmap(pixmap: &mut Pixmap, value: f32) {
    for_each_rgb(pixmap, |x, y, rgb| {
        // simple deterministic hash
        let mut h =
            (x.wrapping_mul(374761393) ^ y.wrapping_mul(668265263)).wrapping_add(1274126177);
        h = (h ^ (h >> 13)).wrapping_mul(1274126177);
        let rand = (h as f32 / u32::MAX as f32) * 2.0 - 1.0;
        let delta = rand * value * 255.0;
        for channel in rgb {
            let v = *channel as f32 + delta;
            *channel = v.clamp(0.0, 255.0) as u8;
        }
    });
}

fn pixelate_pixmap(pixmap: &mut Pixmap, value: f32) {
    let block = value.round().max(1.0) as u32;
    if block <= 1 {
        return;
    }
    let w = pixmap.width();
    let h = pixmap.height();
    let data = pixmap.data_mut();
    let mut y = 0;
    while y < h {
        let mut x = 0;
        while x < w {
            let mut sum = [0u32; 4];
            let mut count = 0u32;
            for yy in y..(y + block).min(h) {
                for xx in x..(x + block).min(w) {
                    let idx = ((yy * w + xx) * 4) as usize;
                    for c in 0..4 {
                        sum[c] += data[idx + c] as u32;
                    }
                    count += 1;
                }
            }
            if count == 0 {
                x += block;
                continue;
            }
            for yy in y..(y + block).min(h) {
                for xx in x..(x + block).min(w) {
                    let idx = ((yy * w + xx) * 4) as usize;
                    for c in 0..4 {
                        data[idx + c] = (sum[c] / count) as u8;
                    }
                }
            }
            x += block;
        }
        y += block;
    }
}

fn vignette_pixmap(pixmap: &mut Pixmap, value: f32) {
    let w = pixmap.width() as f32;
    let h = pixmap.height() as f32;
    let cx = w / 2.0;
    let cy = h / 2.0;
    let max_dist = (cx * cx + cy * cy).sqrt();
    for_each_rgb(pixmap, |x, y, rgb| {
        let dx = x as f32 - cx;
        let dy = y as f32 - cy;
        let dist = (dx * dx + dy * dy).sqrt();
        let t = (dist / max_dist).clamp(0.0, 1.0);
        let factor = (1.0 - value * t * t).clamp(0.0, 1.0);
        for channel in rgb {
            *channel = (*channel as f32 * factor).round() as u8;
        }
    });
}

fn blur_pixmap(pixmap: &mut Pixmap, radius: f32) {
    if radius <= 0.0 {
        return;
    }
    let w = pixmap.width() as usize;
    let h = pixmap.height() as usize;
    if w == 0 || h == 0 {
        return;
    }
    let sigma = radius.max(0.1);
    let krad = (3.0 * sigma).ceil() as i32;
    let mut kernel: Vec<f32> = Vec::with_capacity((2 * krad + 1) as usize);
    let mut sum = 0.0f32;
    for i in -krad..=krad {
        let x = i as f32;
        let weight = (-(x * x) / (2.0 * sigma * sigma)).exp();
        kernel.push(weight);
        sum += weight;
    }
    if sum > 0.0 {
        for weight in &mut kernel {
            *weight /= sum;
        }
    }
    let src = pixmap.data().to_vec();
    let mut tmp = vec![0u8; src.len()];
    // Horizontal pass.

    for y in 0..h {
        for x in 0..w {
            let mut acc = [0.0f32; 4];
            for (ki, &kw) in kernel.iter().enumerate() {
                let dx = ki as i32 - krad;
                let sx = (x as i32 + dx).clamp(0, w as i32 - 1) as usize;
                let idx = (y * w + sx) * 4;
                for channel in 0..4 {
                    acc[channel] += src[idx + channel] as f32 * kw;
                }
            }
            let idx = (y * w + x) * 4;
            for channel in 0..4 {
                tmp[idx + channel] = acc[channel].round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    // Vertical pass.
    let mut out = vec![0u8; src.len()];
    for y in 0..h {
        for x in 0..w {
            let mut acc = [0.0f32; 4];
            for (ki, &kw) in kernel.iter().enumerate() {
                let dy = ki as i32 - krad;
                let sy = (y as i32 + dy).clamp(0, h as i32 - 1) as usize;
                let idx = (sy * w + x) * 4;
                for channel in 0..4 {
                    acc[channel] += tmp[idx + channel] as f32 * kw;
                }
            }
            let idx = (y * w + x) * 4;
            for channel in 0..4 {
                out[idx + channel] = acc[channel].round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    pixmap.data_mut().copy_from_slice(&out);
}
