use std::collections::BTreeMap;

use crate::ui::retained_host::primitives::Color;

use super::super::super::pane_value_conversion::{value_as_color, value_as_f64};

pub(super) struct ProjectedSelectedSegment {
    pub(super) border_width: Option<f64>,
    pub(super) underline_height: f32,
    pub(super) underline_color: Color,
}

pub(super) fn projected_selected_segment(
    attributes: &BTreeMap<String, toml::Value>,
) -> ProjectedSelectedSegment {
    ProjectedSelectedSegment {
        border_width: attributes
            .get("selected_segment_border_width")
            .or_else(|| attributes.get("selected_border_width"))
            .and_then(value_as_f64),
        underline_height: attributes
            .get("selected_segment_underline_height")
            .or_else(|| attributes.get("selected_underline_height"))
            .and_then(value_as_f64)
            .unwrap_or(0.0) as f32,
        underline_color: attributes
            .get("selected_segment_underline_color")
            .or_else(|| attributes.get("selected_underline_color"))
            .and_then(value_as_color)
            .unwrap_or_else(|| Color::from_argb_u8(0, 0, 0, 0)),
    }
}
