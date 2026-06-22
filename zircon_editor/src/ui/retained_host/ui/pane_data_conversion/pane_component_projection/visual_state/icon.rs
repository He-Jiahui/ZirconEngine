use std::collections::BTreeMap;

use crate::ui::retained_host::primitives::Color;

use super::super::super::pane_value_conversion::{value_as_color, value_as_f64};

pub(super) struct ProjectedIconState {
    pub(super) color: Color,
    pub(super) stroke_width: f32,
}

pub(super) fn projected_icon_state(
    attributes: &BTreeMap<String, toml::Value>,
) -> ProjectedIconState {
    ProjectedIconState {
        color: attributes
            .get("icon_color")
            .or_else(|| attributes.get("thumb_color"))
            .or_else(|| attributes.get("icon_stroke"))
            .or_else(|| attributes.get("arrow_color"))
            .and_then(value_as_color)
            .unwrap_or_else(|| Color::from_argb_u8(0, 0, 0, 0)),
        stroke_width: attributes
            .get("icon_stroke_width")
            .or_else(|| attributes.get("stroke_width"))
            .and_then(value_as_f64)
            .unwrap_or(0.0) as f32,
    }
}
