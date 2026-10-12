use std::collections::BTreeMap;

use crate::ui::retained_host::primitives::Color;

use super::super::super::pane_value_conversion::{value_as_bool, value_as_color};

pub(super) struct ProjectedStateLayer {
    pub(super) enabled: bool,
    pub(super) color: Color,
}

pub(super) fn projected_state_layer(
    attributes: &BTreeMap<String, toml::Value>,
) -> ProjectedStateLayer {
    ProjectedStateLayer {
        enabled: attributes
            .get("state_layer_enabled")
            .or_else(|| attributes.get("display_state_layer"))
            .and_then(value_as_bool)
            .unwrap_or(false),
        color: attributes
            .get("state_layer_color")
            .or_else(|| attributes.get("thumb_halo_color"))
            .or_else(|| attributes.get("ripple_color"))
            .or_else(|| attributes.get("color"))
            .and_then(value_as_color)
            .unwrap_or_else(|| Color::from_argb_u8(0, 0, 0, 0)),
    }
}
