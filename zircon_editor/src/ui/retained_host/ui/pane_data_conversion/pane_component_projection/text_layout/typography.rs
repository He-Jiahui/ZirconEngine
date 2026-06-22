use std::collections::BTreeMap;

use super::super::super::pane_value_conversion::value_as_string;
use super::super::attribute_values::value_as_i32;
use super::attributes::f32_attribute;

pub(super) struct ProjectedTypography {
    pub(super) font_size: f32,
    pub(super) font_weight: i32,
    pub(super) text_align: String,
    pub(super) overflow: String,
}

pub(super) fn projected_typography(
    attributes: &BTreeMap<String, toml::Value>,
    component_role: &str,
) -> ProjectedTypography {
    ProjectedTypography {
        font_size: f32_attribute(attributes, "font_size", 0.0),
        font_weight: attributes
            .get("font_weight")
            .and_then(value_as_i32)
            .unwrap_or(0),
        text_align: attributes
            .get("text_align")
            .or_else(|| attributes.get("textAlign"))
            .and_then(value_as_string)
            .unwrap_or_else(|| {
                if component_role == "divider" {
                    "center".to_string()
                } else {
                    "left".to_string()
                }
            }),
        overflow: attributes
            .get("overflow")
            .and_then(value_as_string)
            .unwrap_or_default(),
    }
}
