use super::super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::{component_variant_contains, first_non_empty};

pub(super) fn badge_color_token(node: &TemplatePaneNodeData) -> &str {
    for token in [
        "primary",
        "secondary",
        "error",
        "danger",
        "info",
        "success",
        "warning",
        "default",
    ] {
        if component_variant_contains(node, token) {
            return token;
        }
    }
    first_non_empty(&[node.validation_level.as_str(), node.text_tone.as_str()])
}
