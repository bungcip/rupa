//! Paint validation and lowering for Rupa.
//!
//! Converts syntax-level colors ([`rupa_syntax::ast::Color`]) into IR paint
//! values ([`rupa_ir::paint::Paint`]). See specification §4.3 and §8.1.

#![allow(clippy::result_large_err)]

use rupa_ir::paint::{Color as IrColor, Paint, PaintId, PaintStore};
use rupa_syntax::ast::{Color as AstColor, Fill, GradientFill, PatternFill};
use rupa_syntax::diagnostic::{Diagnostic, DiagnosticBag, SuggestedEdit, codes, did_you_mean_from};
use rupa_syntax::span::Span;

/// Canonical named colors and their 8-bit RGBA values (specification §4.3).
pub const NAMED_COLORS: &[(&str, [u8; 4])] = &[
    ("white", [255, 255, 255, 255]),
    ("black", [0, 0, 0, 255]),
    ("red", [255, 0, 0, 255]),
    ("green", [0, 255, 0, 255]),
    ("blue", [0, 0, 255, 255]),
    ("yellow", [255, 255, 0, 255]),
    ("orange", [255, 165, 0, 255]),
    ("purple", [128, 0, 128, 255]),
    ("gray", [128, 128, 128, 255]),
    ("transparent", [0, 0, 0, 0]),
];

const HEX_FORMAT_HELP: &str = "use \"#RRGGBB\" or \"#RRGGBBAA\" with hex digits 0-9, A-F";

/// Validates and lowers paint from syntax to IR.
pub struct PaintValidator;

impl PaintValidator {
    /// Parse a syntax color into an IR color.
    ///
    /// Named colors are lowercase and case-sensitive; hex digits are
    /// case-insensitive.
    pub fn parse_color(color: &AstColor, span: Span) -> Result<IrColor, Diagnostic> {
        match color {
            AstColor::Named(name) => Self::parse_named_color(name, span),
            AstColor::Hex(hex) => Self::parse_hex_color(hex, span),
            AstColor::Rgb(r, g, b) => Ok(IrColor::from_rgb8(*r, *g, *b)),
            AstColor::Rgba(r, g, b, a) => Ok(IrColor::from_rgba8(*r, *g, *b, *a)),
        }
    }

    /// Lower a syntax color into a solid paint.
    pub fn lower_color(color: &AstColor, span: Span) -> Result<Paint, DiagnosticBag> {
        match Self::parse_color(color, span) {
            Ok(color) => Ok(Paint::Solid(color)),
            Err(diagnostic) => Err(bag_with(diagnostic)),
        }
    }

    /// Lower a `fill` declaration.
    ///
    /// `fill none` lowers to `None`; solid colors and gradients lower to paints.
    pub fn lower_fill(fill: &Fill) -> Result<Option<Paint>, DiagnosticBag> {
        if let Some(gradient) = &fill.gradient {
            return Self::lower_gradient(gradient).map(Some);
        }
        if let Some(pattern) = &fill.pattern {
            return Self::lower_pattern(pattern).map(Some);
        }
        fill.color.as_ref().map_or(Ok(None), |color| {
            Self::lower_color(color, fill.span).map(Some)
        })
    }

    /// Lower a `fill` declaration into a paint store, returning its ID.
    ///
    /// `fill none` returns `None` and does not allocate a paint.
    pub fn lower_fill_to_store(
        fill: &Fill,
        store: &mut PaintStore,
    ) -> Result<Option<PaintId>, DiagnosticBag> {
        Ok(Self::lower_fill(fill)?.map(|paint| store.add(paint)))
    }

    /// Lower a gradient fill into an IR paint.
    pub fn lower_gradient(gradient: &GradientFill) -> Result<Paint, DiagnosticBag> {
        use rupa_ir::paint::{GradientStop as IrGradientStop, LinearGradient, RadialGradient};
        use rupa_syntax::ast::Unit;

        let mut stops = Vec::new();
        if gradient.stops.is_empty() {
            for (color, position) in [(&gradient.from, 0.0), (&gradient.to, 1.0)] {
                if let Some(color) = color {
                    let color = Self::parse_color(color, gradient.span).map_err(bag_with)?;
                    stops.push(IrGradientStop { position, color });
                }
            }
        } else {
            for stop in &gradient.stops {
                let color = Self::parse_color(&stop.color, stop.span).map_err(bag_with)?;
                let position = match stop.offset.unit {
                    Unit::Percent => (stop.offset.value / 100.0) as f32,
                    _ => stop.offset.value as f32,
                }
                .clamp(0.0, 1.0);
                stops.push(IrGradientStop { position, color });
            }
        }

        if stops.is_empty() {
            let mut bag = DiagnosticBag::new();
            bag.error(
                codes::EXPECTED_TOKEN,
                "gradient requires at least one stop",
                gradient.span,
            );
            return Err(bag);
        }
        if let [stop] = stops.as_slice() {
            return Ok(Paint::Solid(stop.color));
        }

        stops.sort_by(|a, b| {
            a.position
                .partial_cmp(&b.position)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        if gradient.radial {
            return Ok(Paint::RadialGradient(RadialGradient { stops }));
        }

        let angle = gradient.angle.as_ref().map_or(0.0, |d| d.value as f32);

        Ok(Paint::LinearGradient(LinearGradient { stops, angle }))
    }

    /// Lower a pattern fill into an IR paint.
    pub fn lower_pattern(pattern: &PatternFill) -> Result<Paint, DiagnosticBag> {
        use rupa_ir::paint::PatternPaint;

        let color = match &pattern.color {
            Some(color) => Self::parse_color(color, pattern.span).map_err(bag_with)?,
            None => IrColor::black(),
        };
        let background = match &pattern.background {
            Some(color) => Self::parse_color(color, pattern.span).map_err(bag_with)?,
            None => IrColor::white(),
        };
        let size = pattern
            .size
            .as_ref()
            .map_or(8.0, |d| d.value as f32)
            .max(1.0);
        let angle = pattern.angle.as_ref().map_or(0.0, |d| d.value as f32);

        if !["dots", "grid", "stripes", "checkerboard"].contains(&pattern.kind.as_str()) {
            let mut bag = DiagnosticBag::new();
            bag.error_with_help(
                codes::INVALID_PROPERTY_VALUE,
                format!("unknown pattern kind `{}`", pattern.kind),
                pattern.span,
                "use dots, grid, stripes, or checkerboard",
            );
            return Err(bag);
        }

        Ok(Paint::Pattern(PatternPaint {
            kind: pattern.kind.to_string(),
            color,
            background,
            size,
            angle,
        }))
    }

    fn parse_named_color(name: &str, span: Span) -> Result<IrColor, Diagnostic> {
        let Some((_, [r, g, b, a])) = NAMED_COLORS
            .iter()
            .find(|(candidate, _)| *candidate == name)
        else {
            let names: Vec<&str> = NAMED_COLORS.iter().map(|(name, _)| *name).collect();
            let suggestion = did_you_mean_from(name, &names);
            let mut help = format!("valid named colors: {}", names.join(", "));
            if let Some(suggestion) = suggestion {
                help = format!("did you mean `{suggestion}`? {help}");
            }
            let mut diagnostic = Diagnostic::error(
                codes::UNKNOWN_NAMED_COLOR,
                format!("unknown named color `{name}`"),
                span,
            )
            .with_help(help);
            if let Some(suggestion) = suggestion {
                diagnostic.edits.push(SuggestedEdit {
                    span,
                    replacement: suggestion.to_string(),
                });
            }
            return Err(diagnostic);
        };
        Ok(IrColor::from_rgba8(*r, *g, *b, *a))
    }

    fn parse_hex_color(hex: &str, span: Span) -> Result<IrColor, Diagnostic> {
        let Some(digits) = hex.strip_prefix('#') else {
            return Err(Self::invalid_hex(
                span,
                "hex color must start with '#'",
                HEX_FORMAT_HELP,
            ));
        };

        let bytes = digits.as_bytes();
        if bytes.len() != 6 && bytes.len() != 8 {
            return Err(Self::invalid_hex(
                span,
                format!(
                    "hex color must have 6 or 8 digits after '#', found {}",
                    bytes.len()
                ),
                HEX_FORMAT_HELP,
            ));
        }
        if !bytes.iter().all(u8::is_ascii_hexdigit) {
            return Err(Self::invalid_hex(
                span,
                "hex color contains non-hexadecimal characters",
                HEX_FORMAT_HELP,
            ));
        }

        // Safe to slice: all bytes are ASCII hex digits at this point.
        let component = |index: usize| -> u8 {
            u8::from_str_radix(&digits[index..index + 2], 16)
                .expect("validated hexadecimal component")
        };
        let r = component(0);
        let g = component(2);
        let b = component(4);
        let a = if bytes.len() == 8 { component(6) } else { 255 };
        Ok(IrColor::from_rgba8(r, g, b, a))
    }

    fn invalid_hex(span: Span, detail: impl Into<String>, help: &str) -> Diagnostic {
        Diagnostic::error(
            codes::INVALID_HEX_COLOR_FORMAT,
            format!("invalid hex color: {}", detail.into()),
            span,
        )
        .with_help(help)
    }
}

fn bag_with(diagnostic: Diagnostic) -> DiagnosticBag {
    let mut bag = DiagnosticBag::new();
    bag.push(diagnostic);
    bag
}

#[cfg(test)]
mod tests {
    use super::*;
    use rupa_syntax::span::{FileId, Position};

    fn span() -> Span {
        Span::new(FileId(0), Position::new(0), Position::new(1))
    }

    fn named(name: &str) -> AstColor {
        AstColor::Named(name.into())
    }

    fn fill(color: Option<AstColor>) -> Fill {
        Fill {
            color,
            gradient: None,
            pattern: None,
            span: span(),
        }
    }

    fn expect_color(color: AstColor) -> IrColor {
        PaintValidator::parse_color(&color, span()).expect("valid color")
    }

    fn expect_error(color: AstColor) -> Diagnostic {
        PaintValidator::parse_color(&color, span()).expect_err("invalid color")
    }

    #[test]
    fn all_named_colors_parse_to_expected_rgba() {
        let expected = [
            ("white", [255, 255, 255, 255]),
            ("black", [0, 0, 0, 255]),
            ("red", [255, 0, 0, 255]),
            ("green", [0, 255, 0, 255]),
            ("blue", [0, 0, 255, 255]),
            ("yellow", [255, 255, 0, 255]),
            ("orange", [255, 165, 0, 255]),
            ("purple", [128, 0, 128, 255]),
            ("gray", [128, 128, 128, 255]),
            ("transparent", [0, 0, 0, 0]),
        ];
        for (name, rgba) in expected {
            assert_eq!(expect_color(named(name)).to_rgba8(), rgba, "color {name}");
        }
    }

    #[test]
    fn named_colors_are_case_sensitive() {
        let diagnostic = expect_error(named("Red"));
        assert_eq!(diagnostic.code, codes::UNKNOWN_NAMED_COLOR);
    }

    #[test]
    fn unknown_named_color_has_name_list_and_suggestion() {
        let diagnostic = expect_error(named("gren"));
        assert_eq!(diagnostic.code, codes::UNKNOWN_NAMED_COLOR);
        let help = diagnostic.help.expect("help");
        assert!(help.contains("did you mean `green`?"), "help: {help}");
        assert!(help.contains("transparent"));
        assert_eq!(
            diagnostic.edits,
            vec![SuggestedEdit {
                span: diagnostic.primary_span,
                replacement: "green".to_string(),
            }]
        );
    }

    #[test]
    fn completely_unknown_named_color_still_lists_valid_names() {
        let diagnostic = expect_error(named("colour"));
        assert_eq!(diagnostic.code, codes::UNKNOWN_NAMED_COLOR);
        let help = diagnostic.help.expect("help");
        assert!(help.contains("valid named colors"));
        assert!(diagnostic.edits.is_empty());
    }

    #[test]
    fn hex_rgb_sets_opaque_alpha() {
        let color = expect_color(AstColor::Hex("#FF6B6B".into()));
        assert_eq!(color.to_rgba8(), [255, 107, 107, 255]);
    }

    #[test]
    fn hex_rgba_preserves_alpha() {
        let color = expect_color(AstColor::Hex("#00FF0080".into()));
        assert_eq!(color.to_rgba8(), [0, 255, 0, 128]);
    }

    #[test]
    fn hex_is_case_insensitive() {
        let upper = expect_color(AstColor::Hex("#AABBCC".into()));
        let lower = expect_color(AstColor::Hex("#aabbcc".into()));
        assert_eq!(upper, lower);
        assert_eq!(upper.to_rgba8(), [170, 187, 204, 255]);
    }

    #[test]
    fn invalid_hex_length_is_t300() {
        let diagnostic = expect_error(AstColor::Hex("#ABC".into()));
        assert_eq!(diagnostic.code, codes::INVALID_HEX_COLOR_FORMAT);
        assert!(diagnostic.help.as_deref().unwrap().contains("#RRGGBB"));
    }

    #[test]
    fn invalid_hex_characters_is_t300() {
        let diagnostic = expect_error(AstColor::Hex("#GGGGGG".into()));
        assert_eq!(diagnostic.code, codes::INVALID_HEX_COLOR_FORMAT);
        assert!(diagnostic.message.contains("non-hexadecimal"));
    }

    #[test]
    fn hex_without_hash_is_t300() {
        let diagnostic = expect_error(AstColor::Hex("FF0000".into()));
        assert_eq!(diagnostic.code, codes::INVALID_HEX_COLOR_FORMAT);
        assert!(diagnostic.message.contains("start with '#'"));
    }

    #[test]
    fn rgb_ast_variant_lowers() {
        let color = expect_color(AstColor::Rgb(10, 20, 30));
        assert_eq!(color.to_rgba8(), [10, 20, 30, 255]);
    }

    #[test]
    fn rgba_ast_variant_lowers() {
        let color = expect_color(AstColor::Rgba(10, 20, 30, 40));
        assert_eq!(color.to_rgba8(), [10, 20, 30, 40]);
    }

    #[test]
    fn fill_none_lowers_to_no_paint() {
        let lower = PaintValidator::lower_fill(&fill(None)).expect("valid fill");
        assert_eq!(lower, None);
    }

    #[test]
    fn fill_solid_lowers_to_paint() {
        let lower = PaintValidator::lower_fill(&fill(Some(named("red")))).expect("valid fill");
        assert_eq!(lower, Some(Paint::Solid(IrColor::from_rgb8(255, 0, 0))));
    }

    #[test]
    fn paint_store_receives_fill_and_skips_none() {
        let mut store = PaintStore::new();
        let none_id =
            PaintValidator::lower_fill_to_store(&fill(None), &mut store).expect("valid fill");
        assert_eq!(none_id, None);
        assert!(store.is_empty());

        let red_id = PaintValidator::lower_fill_to_store(&fill(Some(named("red"))), &mut store)
            .expect("valid fill");
        assert_eq!(red_id, Some(PaintId(0)));
        assert_eq!(
            store.get(PaintId(0)),
            Some(&Paint::Solid(IrColor::from_rgb8(255, 0, 0)))
        );
    }

    #[test]
    fn invalid_fill_color_returns_diagnostic_bag() {
        let bag =
            PaintValidator::lower_fill(&fill(Some(named("colour")))).expect_err("invalid fill");
        assert_eq!(bag.as_slice().len(), 1);
        assert_eq!(bag.as_slice()[0].code, codes::UNKNOWN_NAMED_COLOR);
    }
}
