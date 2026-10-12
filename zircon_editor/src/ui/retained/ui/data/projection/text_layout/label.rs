use std::collections::BTreeMap;

use crate::ui::retained_host::primitives::Color;

use super::super::super::pane_value_conversion::{value_as_color, value_as_f64, value_as_string};

pub(super) struct ProjectedLabelFields {
    pub(super) label_text: String,
    pub(super) label_color: Color,
    pub(super) label_brightness: f32,
}

pub(super) fn projected_label_fields(
    attributes: &BTreeMap<String, toml::Value>,
) -> ProjectedLabelFields {
    ProjectedLabelFields {
        label_text: attributes
            .get("label_text")
            .and_then(value_as_string)
            .unwrap_or_default(),
        label_color: attributes
            .get("label_color")
            .or_else(|| attributes.get("icon_fill"))
            .or_else(|| attributes.get("status_mark_color"))
            .and_then(value_as_color)
            .unwrap_or_default(),
        label_brightness: attributes
            .get("label_brightness")
            .or_else(|| attributes.get("visual_brightness"))
            .and_then(value_as_f64)
            .unwrap_or(1.0) as f32,
    }
}
