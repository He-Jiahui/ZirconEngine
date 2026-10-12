use std::collections::BTreeMap;

use super::super::super::pane_value_conversion::{value_as_bool, value_as_f64};

pub(super) struct ProjectedRippleState {
    pub(super) enabled: bool,
    pub(super) pressed_x: f32,
    pub(super) pressed_y: f32,
    pub(super) unclipped: bool,
}

pub(super) fn projected_ripple_state(
    attributes: &BTreeMap<String, toml::Value>,
) -> ProjectedRippleState {
    let clip_ripple = attributes
        .get("clip_ripple")
        .and_then(value_as_bool)
        .unwrap_or(true);

    ProjectedRippleState {
        enabled: attributes
            .get("ripple_enabled")
            .or_else(|| attributes.get("ripple"))
            .and_then(value_as_bool)
            .unwrap_or(false),
        pressed_x: attributes
            .get("ripple_pressed_x")
            .or_else(|| attributes.get("pressed_x"))
            .and_then(value_as_f64)
            .unwrap_or(0.0) as f32,
        pressed_y: attributes
            .get("ripple_pressed_y")
            .or_else(|| attributes.get("pressed_y"))
            .and_then(value_as_f64)
            .unwrap_or(0.0) as f32,
        unclipped: !clip_ripple,
    }
}
