use std::collections::BTreeMap;

use super::super::super::pane_value_conversion::{value_as_bool, value_as_string};
use super::super::attribute_values::value_as_i32;

pub(super) struct ProjectedWorldRendering {
    pub(super) billboard: bool,
    pub(super) depth_test: bool,
    pub(super) render_order: i32,
    pub(super) camera_target: String,
}

pub(super) fn projected_world_rendering(
    attributes: &BTreeMap<String, toml::Value>,
) -> ProjectedWorldRendering {
    ProjectedWorldRendering {
        billboard: attributes
            .get("billboard")
            .and_then(value_as_bool)
            .unwrap_or(false),
        depth_test: attributes
            .get("depth_test")
            .and_then(value_as_bool)
            .unwrap_or(false),
        render_order: attributes
            .get("render_order")
            .and_then(value_as_i32)
            .unwrap_or(0),
        camera_target: attributes
            .get("camera_target")
            .and_then(value_as_string)
            .unwrap_or_default(),
    }
}
