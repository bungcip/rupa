//! Validation for layer definitions.
//!
//! A layer is a single-use composition plane: it is composited once, at its
//! `z-index`, and is never placed. Each definition is validated independently
//! of its content (specification §6.4).

use rupa_ir::document::ROOT_LAYER_NAME;
use rupa_syntax::ast::{LayerDefinition, Size, Unit};
use rupa_syntax::diagnostic::{DiagnosticBag, codes};
use std::collections::HashSet;

/// Validates layer definitions and duplicate names.
pub struct LayerValidator;

impl LayerValidator {
    /// Validate all layer definitions in a document, including unique names.
    pub fn validate_layers(layers: &[LayerDefinition]) -> DiagnosticBag {
        let mut bag = DiagnosticBag::new();
        let mut seen = HashSet::new();

        for layer in layers {
            if layer.name.is_empty() {
                bag.error_with_help(
                    codes::INVALID_LAYER_NAME,
                    "layer name must not be empty",
                    layer.span,
                    "use a non-empty name such as 'layer \"Background\"'",
                );
            } else if layer.name == ROOT_LAYER_NAME {
                bag.error_with_help(
                    codes::ROOT_LAYER_RESERVED,
                    "the layer name `root` is reserved for the implicit root layer",
                    layer.span,
                    "choose another layer name; the root layer is created automatically",
                );
            } else if !seen.insert(layer.name.as_str()) {
                bag.error_with_help(
                    codes::DUPLICATE_LAYER_NAME,
                    format!("duplicate layer definition `{}`", layer.name),
                    layer.span,
                    "layer names must be unique and case-sensitive",
                );
            }

            Self::validate_plane(layer, &mut bag);
        }

        bag
    }

    /// Validate one layer's declared size and plane properties.
    pub fn validate_layer(layer: &LayerDefinition) -> DiagnosticBag {
        let mut bag = DiagnosticBag::new();
        Self::validate_plane(layer, &mut bag);
        bag
    }

    fn validate_plane(layer: &LayerDefinition, bag: &mut DiagnosticBag) {
        // `size` is optional; an omitted size means the document canvas
        // (specification §6.4), so there is no diagnostic for it.
        if let Some(size) = &layer.size {
            Self::validate_size(size, "layer", bag);
        }

        if let Some(z_index) = layer.z_index
            && (!z_index.is_finite()
                || z_index.fract() != 0.0
                || z_index < f64::from(i32::MIN)
                || z_index > f64::from(i32::MAX))
        {
            bag.error_with_help(
                codes::INVALID_Z_INDEX,
                "layer z-index must be an integer",
                layer.span,
                "use an integer such as 'z-index 10'",
            );
        }

        if let Some(opacity) = layer.opacity
            && (!opacity.is_finite() || !(0.0..=1.0).contains(&opacity))
        {
            bag.error_with_help(
                codes::INVALID_TRANSFORM,
                "layer opacity must be between 0.0 and 1.0",
                layer.span,
                "use a value such as 'opacity 0.8'",
            );
        }
    }

    fn validate_size(size: &Size, label: &str, bag: &mut DiagnosticBag) {
        for (dimension, axis) in [(&size.width, "width"), (&size.height, "height")] {
            if dimension.unit != Unit::Px {
                bag.error_with_help(
                    codes::NON_PIXEL_SHAPE_DIMENSION,
                    format!("{label} {axis} must use pixel units"),
                    dimension.span,
                    "use a pixel value such as '240px'",
                );
            }
            if !dimension.value.is_finite() || dimension.value <= 0.0 {
                bag.error_with_help(
                    codes::INVALID_LAYER_SIZE,
                    format!("{label} {axis} must be a positive, finite number"),
                    dimension.span,
                    "use a value greater than zero",
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::codes_of;
    use crate::validation::Validator;
    use rupa_syntax::span::FileId;

    fn validate(source: &str) -> DiagnosticBag {
        Validator::validate_source(source, FileId(0))
    }

    #[test]
    fn valid_layer_passes() {
        let bag = validate(
            "size 300px x 200px\nlayer \"Background\"\n    size 300px x 200px\n    circle center at (60px, 60px)\n        radius 40px\n        fill white\n",
        );
        assert!(!bag.has_errors(), "diagnostics: {:?}", bag.as_slice());
    }

    #[test]
    fn layer_without_size_is_valid() {
        let bag = validate(
            "size 300px x 200px\nlayer \"Background\"\n    circle center at (60px, 60px)\n        radius 40px\n        fill white\n",
        );
        assert!(!bag.has_errors(), "diagnostics: {:?}", bag.as_slice());
    }

    #[test]
    fn empty_layer_is_valid() {
        let bag = validate("size 300px x 200px\nlayer \"Empty\"\n    z-index 3\n");
        assert!(!bag.has_errors(), "diagnostics: {:?}", bag.as_slice());
    }

    #[test]
    fn layer_background_fill_is_valid() {
        let bag = validate(
            "size 300px x 200px\nlayer \"Sky\"\n    fill \"#87CEEB\"\n    circle center at (60px, 60px)\n        radius 40px\n        fill white\n",
        );
        assert!(!bag.has_errors(), "diagnostics: {:?}", bag.as_slice());
    }

    #[test]
    fn duplicate_layer_background_fill_is_s036() {
        let bag = validate("size 300px x 200px\nlayer \"Sky\"\n    fill red\n    fill blue\n");
        assert!(codes_of(&bag).contains(&codes::DUPLICATE_LAYER_FILL));
    }

    #[test]
    fn invalid_layer_size_is_s017() {
        let bag = validate("size 300px x 200px\nlayer \"Background\"\n    size 0px x 120px\n");
        assert_eq!(codes_of(&bag), vec![codes::INVALID_LAYER_SIZE]);
    }

    #[test]
    fn non_pixel_layer_size_is_s007() {
        let bag = validate("size 300px x 200px\nlayer \"Background\"\n    size 50% x 120px\n");
        assert_eq!(codes_of(&bag), vec![codes::NON_PIXEL_SHAPE_DIMENSION]);
    }

    #[test]
    fn explicit_root_layer_is_s029() {
        let bag = validate("size 100px x 100px\nlayer \"root\"\n    size 10px x 10px\n");
        assert_eq!(codes_of(&bag), vec![codes::ROOT_LAYER_RESERVED]);
    }

    #[test]
    fn layer_names_are_case_sensitive_for_root_reservation() {
        let bag = validate("size 100px x 100px\nlayer \"Root\"\n    size 10px x 10px\n");
        assert!(!bag.has_errors(), "diagnostics: {:?}", bag.as_slice());
    }

    #[test]
    fn duplicate_layer_names_are_s016() {
        let bag = validate(
            "size 300px x 200px\nlayer \"Sky\"\n    size 10px x 10px\nlayer \"Sky\"\n    size 10px x 10px\n",
        );
        assert_eq!(codes_of(&bag), vec![codes::DUPLICATE_LAYER_NAME]);
    }

    #[test]
    fn layer_names_are_case_sensitive() {
        let bag = validate(
            "size 300px x 200px\nlayer \"Sky\"\n    size 10px x 10px\nlayer \"sky\"\n    size 10px x 10px\n",
        );
        assert!(!bag.has_errors(), "diagnostics: {:?}", bag.as_slice());
    }

    #[test]
    fn empty_layer_name_is_s018() {
        let bag = validate("size 300px x 200px\nlayer \"\"\n    size 10px x 10px\n");
        assert!(codes_of(&bag).contains(&codes::INVALID_LAYER_NAME));
    }

    #[test]
    fn layer_child_shapes_are_validated() {
        let bag = validate(
            "size 300px x 200px\nlayer \"Sky\"\n    size 10px x 10px\n    rectangle at (0px, 0px)\n        size -1px x 10px\n",
        );
        assert!(codes_of(&bag).contains(&codes::NEGATIVE_DIMENSION));
    }

    #[test]
    fn fractional_layer_z_index_is_s020() {
        let bag =
            validate("size 300px x 200px\nlayer \"Sky\"\n    size 10px x 10px\n    z-index 1.5\n");
        assert!(codes_of(&bag).contains(&codes::INVALID_Z_INDEX));
    }

    #[test]
    fn layer_opacity_out_of_range_is_rejected() {
        let bag =
            validate("size 300px x 200px\nlayer \"Sky\"\n    size 10px x 10px\n    opacity 2\n");
        assert!(codes_of(&bag).contains(&codes::INVALID_TRANSFORM));
    }
}
