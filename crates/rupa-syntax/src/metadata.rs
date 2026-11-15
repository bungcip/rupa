//! Canonical language metadata shared by the lexer, diagnostics, CLI, and
//! browser bindings.
//!
//! Keyword spellings and their highlighter class live in one table so adding a
//! keyword cannot silently leave the playground or the language reference
//! behind. Named colours stay in `rupa-compiler::paint::NAMED_COLORS`, which is
//! the only place that defines their RGBA values.

use crate::token::{TokenKind, Unit};
use serde::Serialize;

/// How a documented keyword is presented by tooling such as the playground.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum KeywordClass {
    /// Structural keyword, document setting, primitive, or literal operator.
    Keyword,
    /// Property name, property value, or operator word.
    Property,
    /// Boolean literal.
    Literal,
}

impl KeywordClass {
    /// Stable lowercase name used as the JSON key for this class.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Keyword => "keyword",
            Self::Property => "property",
            Self::Literal => "literal",
        }
    }
}

/// One lexer keyword together with the class tooling should display it as.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeywordSpec {
    /// Exact source spelling.
    pub spelling: &'static str,
    /// Presentation class for editors and references.
    pub class: KeywordClass,
}

macro_rules! keywords {
    ($(($spelling:literal, $variant:ident, $class:ident)),* $(,)?) => {
        /// Every keyword the lexer recognizes.
        pub const KEYWORDS: &[KeywordSpec] = &[
            $(KeywordSpec {
                spelling: $spelling,
                class: KeywordClass::$class,
            }),*
        ];

        /// Every keyword spelling the lexer recognizes, in table order.
        pub const ALL_KEYWORDS: &[&str] = &[$($spelling),*];

        /// Map a source word to its keyword token, or `None` for an identifier.
        pub fn keyword_token(value: &str) -> Option<TokenKind> {
            match value {
                $($spelling => Some(TokenKind::$variant),)*
                _ => None,
            }
        }
    };
}

keywords! {
    ("size", Size, Keyword),
    ("background", Background, Keyword),
    ("rectangle", Rectangle, Keyword),
    ("circle", Circle, Keyword),
    ("at", At, Property),
    ("center", Center, Property),
    ("radius", Radius, Property),
    ("fill", Fill, Property),
    ("stroke", Stroke, Property),
    ("x", X, Property),
    ("layer", Layer, Keyword),
    ("group", Group, Keyword),
    ("place", Place, Keyword),
    ("z-index", ZIndex, Property),
    ("scale", Scale, Property),
    ("rotate", Rotate, Property),
    ("opacity", Opacity, Property),
    ("repeat", Repeat, Keyword),
    ("times", Times, Property),
    ("as", As, Property),
    ("let", Let, Keyword),
    ("in", In, Property),
    ("gradient", Gradient, Property),
    ("pattern", Pattern, Property),
    ("radial", Radial, Property),
    ("from", From, Property),
    ("to", To, Property),
    ("angle", Angle, Property),
    ("stop", Stop, Property),
    ("picture", Picture, Keyword),
    ("fit", Fit, Property),
    ("contain", Contain, Property),
    ("cover", Cover, Property),
    ("none", None_, Property),
    ("text", Text, Keyword),
    ("font", Font, Property),
    ("weight", Weight, Property),
    ("align", Align, Property),
    ("bold", Bold, Property),
    ("normal", Normal, Property),
    ("left", Left, Property),
    ("right", Right, Property),
    ("raster", Raster, Property),
    ("mode", Mode, Property),
    ("clear", Clear, Keyword),
    ("pixel", Pixel, Keyword),
    ("brush", Brush, Keyword),
    ("path", Path, Keyword),
    ("start", Start, Property),
    ("line", Line, Keyword),
    ("curve", Curve, Property),
    ("quad", Quad, Property),
    ("close", Close, Property),
    ("control", Control, Property),
    ("and", And, Keyword),
    ("or", OrKw, Keyword),
    ("not", NotKw, Keyword),
    ("ellipse", Ellipse, Keyword),
    ("square", Square, Keyword),
    ("triangle", Triangle, Keyword),
    ("polygon", Polygon, Keyword),
    ("polyline", Polyline, Keyword),
    ("points", Points, Property),
    ("width", Width, Property),
    ("cap", Cap, Property),
    ("join", Join, Property),
    ("dash", Dash, Property),
    ("miter", Miter, Property),
    ("round", Round, Property),
    ("butt", Butt, Property),
    ("bevel", Bevel, Property),
    ("miter-limit", MiterLimit, Property),
    ("effect", Effect, Property),
    ("blur", Blur, Property),
    ("brightness", Brightness, Property),
    ("contrast", Contrast, Property),
    ("saturation", Saturation, Property),
    ("grayscale", Grayscale, Property),
    ("hue-rotate", HueRotate, Property),
    ("sharpen", Sharpen, Property),
    ("noise", Noise, Property),
    ("pixelate", Pixelate, Property),
    ("vignette", Vignette, Property),
    ("displacement-map", DisplacementMap, Property),
    ("clip", Clip, Property),
    ("blend", Blend, Property),
    ("true", True, Literal),
    ("false", False, Literal),
    ("fn", Fn, Keyword),
    ("return", Return, Keyword),
    ("random", Random, Keyword),
    ("seed", Seed, Keyword),
}

/// Unit suffixes recognized after a number, in reference order.
pub const UNITS: &[&str] = &["px", "pt", "%", "deg"];

/// Map a numeric unit suffix to its token, or `None` for an unknown suffix.
pub fn unit_token(value: &str) -> Option<Unit> {
    match value {
        "px" => Some(Unit::Px),
        "pt" => Some(Unit::Pt),
        "deg" => Some(Unit::Deg),
        "%" => Some(Unit::Percent),
        _ => None,
    }
}

/// One drawing primitive and the properties it accepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrimitiveSpec {
    /// Primitive keyword.
    pub name: &'static str,
    /// Property spellings accepted by the primitive.
    pub properties: &'static [&'static str],
}

/// Drawing primitives and their documented properties.
pub const PRIMITIVES: &[PrimitiveSpec] = &[
    PrimitiveSpec {
        name: "rectangle",
        properties: &[
            "at", "size", "fill", "stroke", "width", "rotate", "scale", "opacity",
        ],
    },
    PrimitiveSpec {
        name: "circle",
        properties: &[
            "center at",
            "radius",
            "fill",
            "stroke",
            "width",
            "rotate",
            "scale",
            "opacity",
        ],
    },
    PrimitiveSpec {
        name: "ellipse",
        properties: &[
            "center at",
            "size",
            "fill",
            "stroke",
            "width",
            "rotate",
            "scale",
            "opacity",
        ],
    },
    PrimitiveSpec {
        name: "square",
        properties: &[
            "at", "size", "fill", "stroke", "width", "rotate", "scale", "opacity",
        ],
    },
    PrimitiveSpec {
        name: "triangle",
        properties: &[
            "at", "size", "fill", "stroke", "width", "rotate", "scale", "opacity",
        ],
    },
    PrimitiveSpec {
        name: "polygon",
        properties: &[
            "points", "fill", "stroke", "width", "rotate", "scale", "opacity",
        ],
    },
    PrimitiveSpec {
        name: "line",
        properties: &[
            "from", "to", "stroke", "width", "rotate", "scale", "opacity",
        ],
    },
    PrimitiveSpec {
        name: "polyline",
        properties: &["points", "stroke", "width", "rotate", "scale", "opacity"],
    },
    PrimitiveSpec {
        name: "path",
        properties: &[
            "start", "line", "curve", "quad", "close", "fill", "stroke", "width", "rotate",
            "scale", "opacity",
        ],
    },
    PrimitiveSpec {
        name: "picture",
        properties: &["path", "at", "size", "fit", "rotate", "scale", "opacity"],
    },
    PrimitiveSpec {
        name: "text",
        properties: &[
            "at",
            "center at",
            "font",
            "size",
            "weight",
            "fill",
            "align",
            "rotate",
            "scale",
            "opacity",
        ],
    },
];

/// Keywords for one presentation class, alphabetically sorted.
pub fn keywords_in_class(class: KeywordClass) -> Vec<&'static str> {
    let mut words: Vec<&'static str> = KEYWORDS
        .iter()
        .filter(|keyword| keyword.class == class)
        .map(|keyword| keyword.spelling)
        .collect();
    words.sort_unstable();
    words
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn keyword_spellings_are_unique() {
        let unique: BTreeSet<_> = KEYWORDS.iter().map(|keyword| keyword.spelling).collect();
        assert_eq!(unique.len(), KEYWORDS.len());
    }

    #[test]
    fn units_map_both_ways() {
        for unit in UNITS {
            assert!(unit_token(unit).is_some(), "unit {unit} is not lexed");
        }
    }

    #[test]
    fn legacy_wrapper_words_are_not_keywords() {
        assert!(keyword_token("document").is_none());
        assert!(keyword_token("export").is_none());
    }
}
