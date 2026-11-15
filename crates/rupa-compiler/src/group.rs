//! Validation for reusable group definitions and group placements.
//!
//! A group is a reusable visual template; every `place` references a group
//! definition by name. Placing a layer is a semantic error because layers are
//! single-use composition planes (specification §6.0, §6.2, §6.3).

use crate::shapes::ShapeValidator;
use rupa_ir::document::ROOT_LAYER_NAME;
use rupa_syntax::ast::{Document, DocumentChild, GroupDefinition, GroupInstance, Size, Unit};
use rupa_syntax::diagnostic::{DiagnosticBag, codes, did_you_mean_from};
use std::collections::{HashMap, HashSet};

/// Validates group definitions, placements, and group-reference cycles.
pub struct GroupValidator;

impl GroupValidator {
    /// Validate group definitions: names, size, and reference cycles.
    pub fn validate_groups(groups: &[GroupDefinition]) -> DiagnosticBag {
        let mut bag = DiagnosticBag::new();
        let mut seen = HashSet::new();

        for group in groups {
            if group.name.is_empty() {
                bag.error_with_help(
                    codes::INVALID_GROUP_NAME,
                    "group name must not be empty",
                    group.span,
                    "use a non-empty name such as 'group \"Cloud\"'",
                );
            } else if group.name == ROOT_LAYER_NAME {
                bag.error_with_help(
                    codes::ROOT_LAYER_RESERVED,
                    "the name `root` is reserved for the implicit root layer",
                    group.span,
                    "choose another group name; the root layer is created automatically",
                );
            } else if !seen.insert(group.name.as_str()) {
                bag.error_with_help(
                    codes::DUPLICATE_GROUP_NAME,
                    format!("duplicate group definition `{}`", group.name),
                    group.span,
                    "group names must be unique and case-sensitive",
                );
            }

            if let Some(size) = &group.size {
                Self::validate_size(size, &mut bag);
            }
        }

        bag.append(Self::validate_cycles(groups));
        bag
    }

    /// Validate every `place` in the document, including nested ones.
    ///
    /// Reference order does not matter: all definitions are collected first.
    /// A placement naming a layer reports `S034`; an unknown name reports
    /// `S033` with a close-match suggestion from the group definitions.
    pub fn validate_placements(document: &Document) -> DiagnosticBag {
        let mut bag = DiagnosticBag::new();
        let group_names: HashSet<&str> = document
            .groups
            .iter()
            .map(|group| group.name.as_str())
            .collect();
        let layer_names: HashSet<&str> = document
            .layers
            .iter()
            .map(|layer| layer.name.as_str())
            .collect();

        Self::validate_children(&document.children, &group_names, &layer_names, &mut bag);
        for group in &document.groups {
            Self::validate_children(&group.children, &group_names, &layer_names, &mut bag);
        }
        for layer in &document.layers {
            Self::validate_children(&layer.children, &group_names, &layer_names, &mut bag);
        }
        bag
    }

    fn validate_children(
        children: &[DocumentChild],
        group_names: &HashSet<&str>,
        layer_names: &HashSet<&str>,
        bag: &mut DiagnosticBag,
    ) {
        for child in children {
            match child {
                DocumentChild::Place(instance) => {
                    bag.append(Self::validate_instance(instance, group_names, layer_names));
                }
                DocumentChild::Repeat(repeat) => {
                    Self::validate_children(&repeat.body, group_names, layer_names, bag);
                }
                _ => {}
            }
        }
    }

    /// Validate one placement against the known group and layer names.
    pub fn validate_instance(
        instance: &GroupInstance,
        group_names: &HashSet<&str>,
        layer_names: &HashSet<&str>,
    ) -> DiagnosticBag {
        let mut bag = DiagnosticBag::new();

        if instance.name.is_empty() {
            bag.error_with_help(
                codes::INVALID_GROUP_NAME,
                "placement name must not be empty",
                instance.span,
                "use a defined group name such as 'place \"Cloud\" at (0px, 0px)'",
            );
        } else if layer_names.contains(instance.name.as_str()) {
            bag.error_with_help(
                codes::PLACE_TARGETS_LAYER,
                format!(
                    "`{}` is a layer, and `place` targets group definitions only",
                    instance.name
                ),
                instance.span,
                format!(
                    "define 'group \"{0}\"' and place that instead; a layer is composited once as a plane",
                    instance.name
                ),
            );
        } else if !group_names.contains(instance.name.as_str()) {
            let mut candidates: Vec<&str> = group_names.iter().copied().collect();
            candidates.sort_unstable();
            let suggestion = did_you_mean_from(&instance.name, &candidates);
            let help = match suggestion {
                Some(suggestion) => format!(
                    "define the group with 'group \"{suggestion}\"' before or after this placement; did you mean `{suggestion}`?"
                ),
                None => "define the group with 'group \"Name\"' before or after this placement"
                    .to_string(),
            };
            bag.error_with_help(
                codes::UNKNOWN_GROUP,
                format!("unknown group definition `{}`", instance.name),
                instance.span,
                help,
            );
        }

        if instance.position.is_none() {
            bag.error_with_help(
                codes::MISSING_POSITION,
                "placement is missing required 'at (x, y)' position",
                instance.span,
                "add 'at (x, y)' after the group name",
            );
        }

        if let Some(z_index) = instance.z_index
            && (!z_index.is_finite()
                || z_index.fract() != 0.0
                || z_index < f64::from(i32::MIN)
                || z_index > f64::from(i32::MAX))
        {
            bag.error_with_help(
                codes::INVALID_Z_INDEX,
                "placement z-index must be an integer",
                instance.span,
                "use an integer such as 'z-index 10'",
            );
        }

        bag.append(ShapeValidator::validate_transform(
            &instance.transform,
            "placement",
        ));

        bag
    }

    fn validate_size(size: &Size, bag: &mut DiagnosticBag) {
        for (dimension, axis) in [(&size.width, "width"), (&size.height, "height")] {
            if dimension.unit != Unit::Px {
                bag.error_with_help(
                    codes::NON_PIXEL_SHAPE_DIMENSION,
                    format!("group {axis} must use pixel units"),
                    dimension.span,
                    "use a pixel value such as '120px'",
                );
            }
            if !dimension.value.is_finite() || dimension.value <= 0.0 {
                bag.error_with_help(
                    codes::INVALID_GROUP_SIZE,
                    format!("group {axis} must be a positive, finite number"),
                    dimension.span,
                    "use a value greater than zero",
                );
            }
        }
    }

    /// Detect recursive group references (specification §6.7).
    ///
    /// The graph has one node per group definition and one edge per nested
    /// placement. A depth-first search reports one `S035` per back edge at the
    /// placement that closes the cycle. Edges to unknown definitions are
    /// skipped because `S033` already reports them.
    pub fn validate_cycles(groups: &[GroupDefinition]) -> DiagnosticBag {
        #[derive(Clone, Copy, PartialEq, Eq)]
        enum Mark {
            Visiting,
            Done,
        }

        fn visit<'a>(
            name: &'a str,
            by_name: &HashMap<&'a str, &'a GroupDefinition>,
            known: &HashSet<&'a str>,
            marks: &mut HashMap<&'a str, Mark>,
            stack: &mut Vec<&'a str>,
            bag: &mut DiagnosticBag,
        ) {
            marks.insert(name, Mark::Visiting);
            stack.push(name);

            if let Some(group) = by_name.get(name) {
                for target_instance in placements(&group.children) {
                    let target = target_instance.name.as_str();
                    if !known.contains(target) {
                        continue;
                    }
                    match marks.get(target).copied() {
                        Some(Mark::Visiting) => {
                            let start = stack
                                .iter()
                                .position(|candidate| *candidate == target)
                                .unwrap_or(0);
                            let mut path: Vec<String> = stack[start..]
                                .iter()
                                .map(|part| format!("`{part}`"))
                                .collect();
                            path.push(format!("`{target}`"));
                            bag.error_with_help(
                                codes::RECURSIVE_GROUP_REFERENCE,
                                format!("recursive group reference: {}", path.join(" -> ")),
                                target_instance.span,
                                "remove or redirect one of these placements to break the cycle",
                            );
                        }
                        Some(Mark::Done) => {}
                        None => visit(target, by_name, known, marks, stack, bag),
                    }
                }
            }

            stack.pop();
            marks.insert(name, Mark::Done);
        }

        let known: HashSet<&str> = groups.iter().map(|group| group.name.as_str()).collect();
        let mut by_name: HashMap<&str, &GroupDefinition> = HashMap::new();
        for group in groups {
            by_name.entry(group.name.as_str()).or_insert(group);
        }

        let mut marks: HashMap<&str, Mark> = HashMap::new();
        let mut stack: Vec<&str> = Vec::new();
        let mut bag = DiagnosticBag::new();
        for group in groups {
            if !marks.contains_key(group.name.as_str()) {
                visit(
                    group.name.as_str(),
                    &by_name,
                    &known,
                    &mut marks,
                    &mut stack,
                    &mut bag,
                );
            }
        }
        bag
    }
}

/// Direct placements of a composition, expanding `repeat` bodies.
fn placements(children: &[DocumentChild]) -> Vec<&GroupInstance> {
    let mut found = Vec::new();
    for child in children {
        match child {
            DocumentChild::Place(instance) => found.push(instance),
            DocumentChild::Repeat(repeat) => found.extend(placements(&repeat.body)),
            _ => {}
        }
    }
    found
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
    fn valid_group_and_placement_pass() {
        let bag = validate(
            "size 300px x 200px\ngroup \"Cloud\"\n    circle center at (40px, 40px)\n        radius 20px\n        fill white\nplace \"Cloud\" at (10px, 10px)\n",
        );
        assert!(!bag.has_errors(), "diagnostics: {:?}", bag.as_slice());
    }

    #[test]
    fn group_size_is_optional() {
        let bag = validate(
            "size 300px x 200px\ngroup \"Dot\"\n    circle center at (10px, 10px)\n        radius 5px\n        fill red\n",
        );
        assert!(!bag.has_errors(), "diagnostics: {:?}", bag.as_slice());
    }

    #[test]
    fn explicit_group_size_must_be_positive() {
        let bag = validate("size 300px x 200px\ngroup \"Cloud\"\n    size 0px x 10px\n");
        assert_eq!(codes_of(&bag), vec![codes::INVALID_GROUP_SIZE]);
    }

    #[test]
    fn duplicate_group_names_are_s030() {
        let bag = validate(
            "size 300px x 200px\ngroup \"Cloud\"\n    circle center at (0px, 0px)\n        radius 1px\n        fill red\ngroup \"Cloud\"\n    circle center at (0px, 0px)\n        radius 1px\n        fill red\n",
        );
        assert_eq!(codes_of(&bag), vec![codes::DUPLICATE_GROUP_NAME]);
    }

    #[test]
    fn empty_group_name_is_s031() {
        let bag = validate(
            "size 300px x 200px\ngroup \"\"\n    circle center at (0px, 0px)\n        radius 1px\n        fill red\n",
        );
        assert_eq!(codes_of(&bag), vec![codes::INVALID_GROUP_NAME]);
    }

    #[test]
    fn group_named_root_is_s029() {
        let bag = validate(
            "size 300px x 200px\ngroup \"root\"\n    circle center at (0px, 0px)\n        radius 1px\n        fill red\n",
        );
        assert_eq!(codes_of(&bag), vec![codes::ROOT_LAYER_RESERVED]);
    }

    #[test]
    fn unknown_group_is_s033_with_suggestion() {
        let bag = validate(
            "size 300px x 200px\nplace \"Clouds\" at (10px, 10px)\ngroup \"Cloud\"\n    circle center at (0px, 0px)\n        radius 1px\n        fill red\n",
        );
        assert_eq!(codes_of(&bag), vec![codes::UNKNOWN_GROUP]);
        let diagnostic = &bag.as_slice()[0];
        assert!(
            diagnostic
                .help
                .as_deref()
                .unwrap_or_default()
                .contains("did you mean `Cloud`?")
        );
    }

    #[test]
    fn placing_a_layer_is_s034() {
        let bag = validate(
            "size 300px x 200px\nlayer \"Plane\"\n    size 100px x 100px\nplace \"Plane\" at (10px, 10px)\n",
        );
        assert!(codes_of(&bag).contains(&codes::PLACE_TARGETS_LAYER));
        let diagnostic = bag
            .as_slice()
            .iter()
            .find(|d| d.code == codes::PLACE_TARGETS_LAYER)
            .expect("layer placement diagnostic");
        assert!(
            diagnostic
                .message
                .contains("`place` targets group definitions only")
        );
    }

    #[test]
    fn missing_position_is_s003() {
        let bag = validate(
            "size 300px x 200px\nplace \"Cloud\"\ngroup \"Cloud\"\n    circle center at (0px, 0px)\n        radius 1px\n        fill red\n",
        );
        assert!(codes_of(&bag).contains(&codes::MISSING_POSITION));
    }

    #[test]
    fn fractional_z_index_is_s020() {
        let bag = validate(
            "size 300px x 200px\nplace \"Cloud\" at (0px, 0px)\n    z-index 1.5\ngroup \"Cloud\"\n    circle center at (0px, 0px)\n        radius 1px\n        fill red\n",
        );
        assert!(codes_of(&bag).contains(&codes::INVALID_Z_INDEX));
    }

    #[test]
    fn invalid_instance_opacity_is_s014() {
        let bag = validate(
            "size 300px x 200px\nplace \"Cloud\" at (0px, 0px)\n    opacity 2\ngroup \"Cloud\"\n    circle center at (0px, 0px)\n        radius 1px\n        fill red\n",
        );
        assert!(codes_of(&bag).contains(&codes::INVALID_TRANSFORM));
    }

    #[test]
    fn nested_group_placements_are_validated() {
        let bag = validate(
            "size 300px x 200px\ngroup \"Outer\"\n    place \"Inner\" at (10px, 10px)\n        z-index 1\ngroup \"Inner\"\n    circle center at (0px, 0px)\n        radius 1px\n        fill red\n",
        );
        assert!(!bag.has_errors(), "diagnostics: {:?}", bag.as_slice());
    }

    #[test]
    fn self_referential_group_is_s035() {
        let bag = validate(
            "size 100px x 100px\ngroup \"Loop\"\n    place \"Loop\" at (0px, 0px)\n        z-index 0\n",
        );
        assert_eq!(codes_of(&bag), vec![codes::RECURSIVE_GROUP_REFERENCE]);
    }

    #[test]
    fn mutual_group_cycles_are_s035() {
        let bag = validate(
            "size 100px x 100px\ngroup \"A\"\n    place \"B\" at (0px, 0px)\ngroup \"B\"\n    place \"A\" at (0px, 0px)\n",
        );
        assert_eq!(codes_of(&bag), vec![codes::RECURSIVE_GROUP_REFERENCE]);
    }
}
