//! Document validation for Rupa.
//!
//! See specification §5. The document coordinate system has its origin at the
//! top-left corner of the canvas: X increases to the right and Y increases
//! downward. For the first implementation all document dimensions are pixels.
//!
//! A `.rupa` file is exactly one document. This module validates its optional
//! positive pixel `size` (see [`crate::validation::Validator::validate_source`]).
//!
//! `size` is optional; omitting it selects the 256x256 default canvas (§5), so
//! a missing declaration is not a diagnostic. The output path is not part of the
//! document: it comes from the CLI (§21, DECISION-031).

use rupa_syntax::ast::{Dimension, Document, Unit};
use rupa_syntax::diagnostic::{DiagnosticBag, codes};

/// Validates the document-level constructs of a parsed document.
pub struct DocumentValidator;

impl DocumentValidator {
    /// Validate a document. All document-level errors are collected;
    /// validation does not stop at the first one.
    pub fn validate(document: &Document) -> DiagnosticBag {
        let mut bag = DiagnosticBag::new();

        // `size` is optional; an omitted size is valid and selects the default
        // canvas (specification §5).
        if let Some(size) = &document.size {
            Self::validate_dimension(&size.width, "width", &mut bag);
            Self::validate_dimension(&size.height, "height", &mut bag);
        }

        bag
    }

    fn validate_dimension(dimension: &Dimension, name: &str, bag: &mut DiagnosticBag) {
        if dimension.unit != Unit::Px {
            bag.error_with_help(
                codes::NON_PIXEL_DIMENSION,
                format!("document {name} must use pixel units, found {name}"),
                dimension.span,
                format!(
                    "use a pixel value such as '{} px' for the document {name}",
                    dimension.value
                ),
            );
        }

        if !dimension.value.is_finite() || dimension.value <= 0.0 {
            bag.error_with_help(
                codes::INVALID_DIMENSIONS,
                format!("document {name} must be a positive, finite number"),
                dimension.span,
                "use a value greater than zero, for example '800px'",
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::codes_of;
    use rupa_syntax::ast::Size;
    use rupa_syntax::span::{FileId, Position, Span};

    fn span(start: u32, end: u32) -> Span {
        Span::new(FileId(0), Position::new(start), Position::new(end))
    }

    fn dimension(value: f64, unit: Unit) -> Dimension {
        Dimension {
            value,
            unit,
            span: span(0, 1),
        }
    }

    fn size(width: f64, height: f64) -> Size {
        Size {
            width: dimension(width, Unit::Px),
            height: dimension(height, Unit::Px),
            span: span(0, 1),
        }
    }

    fn document() -> Document {
        Document {
            span: span(0, 1),
            size: Some(size(100.0, 50.0)),
            background: None,
            seed: None,
            children: Vec::new(),
            groups: Vec::new(),
            layers: Vec::new(),
            functions: Vec::new(),
        }
    }

    #[test]
    fn valid_document_passes() {
        let mut doc = document();
        doc.size = Some(size(800.0, 600.0));
        let bag = DocumentValidator::validate(&doc);
        assert!(!bag.has_errors(), "diagnostics: {:?}", bag.as_slice());
        assert!(bag.warnings().is_empty());
    }

    #[test]
    fn missing_size_is_valid_and_defaults_to_256x256() {
        let mut doc = document();
        doc.size = None;
        let bag = DocumentValidator::validate(&doc);
        assert!(!bag.has_errors(), "diagnostics: {:?}", bag.as_slice());
        assert!(bag.as_slice().is_empty());
    }

    #[test]
    fn zero_width_is_invalid() {
        let mut doc = document();
        doc.size = Some(size(0.0, 50.0));
        let bag = DocumentValidator::validate(&doc);
        assert_eq!(codes_of(&bag), vec![codes::INVALID_DIMENSIONS]);
    }

    #[test]
    fn negative_height_is_invalid() {
        let mut doc = document();
        doc.size = Some(size(100.0, -1.0));
        let bag = DocumentValidator::validate(&doc);
        assert_eq!(codes_of(&bag), vec![codes::INVALID_DIMENSIONS]);
    }

    #[test]
    fn non_finite_dimensions_are_invalid() {
        let mut doc = document();
        doc.size = Some(size(f64::INFINITY, 50.0));
        let bag = DocumentValidator::validate(&doc);
        assert_eq!(codes_of(&bag), vec![codes::INVALID_DIMENSIONS]);
    }

    #[test]
    fn both_dimensions_can_fail() {
        let mut doc = document();
        doc.size = Some(size(0.0, 0.0));
        let bag = DocumentValidator::validate(&doc);
        assert_eq!(
            codes_of(&bag),
            vec![codes::INVALID_DIMENSIONS, codes::INVALID_DIMENSIONS]
        );
    }

    #[test]
    fn non_pixel_dimensions_are_rejected() {
        let mut doc = document();
        doc.size = Some(Size {
            width: dimension(100.0, Unit::Percent),
            height: dimension(50.0, Unit::Px),
            span: span(0, 1),
        });
        let bag = DocumentValidator::validate(&doc);
        assert_eq!(codes_of(&bag), vec![codes::NON_PIXEL_DIMENSION]);
    }

    #[test]
    fn background_omitted_is_valid_and_defaults_to_transparent() {
        let doc = document();
        assert!(doc.background.is_none());
        let bag = DocumentValidator::validate(&doc);
        assert!(!bag.has_errors(), "diagnostics: {:?}", bag.as_slice());
    }

    #[test]
    fn all_document_errors_are_collected() {
        let mut doc = document();
        doc.size = Some(Size {
            width: dimension(0.0, Unit::Percent),
            height: dimension(-1.0, Unit::Px),
            span: span(0, 1),
        });
        let bag = DocumentValidator::validate(&doc);
        assert_eq!(
            codes_of(&bag),
            vec![
                codes::NON_PIXEL_DIMENSION,
                codes::INVALID_DIMENSIONS,
                codes::INVALID_DIMENSIONS,
            ]
        );
    }
}
