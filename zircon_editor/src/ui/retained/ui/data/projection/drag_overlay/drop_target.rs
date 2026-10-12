use std::collections::BTreeMap;

use toml::Value;

use super::attributes::{bool_attribute, f32_attribute};

pub(super) struct ProjectedDropTarget {
    pub(super) allowed: bool,
    pub(super) has_target: bool,
    pub(super) x: f32,
    pub(super) y: f32,
    pub(super) width: f32,
    pub(super) height: f32,
}

pub(super) fn projected_drop_target(attributes: &BTreeMap<String, Value>) -> ProjectedDropTarget {
    let x = f32_attribute(attributes, "drop_target_x");
    let y = f32_attribute(attributes, "drop_target_y");
    let width = f32_attribute(attributes, "drop_target_width");
    let height = f32_attribute(attributes, "drop_target_height");

    ProjectedDropTarget {
        allowed: bool_attribute(attributes, "drop_allowed").unwrap_or(true),
        has_target: x.is_some() && y.is_some() && width.is_some() && height.is_some(),
        x: x.unwrap_or(0.0),
        y: y.unwrap_or(0.0),
        width: width.unwrap_or(0.0),
        height: height.unwrap_or(0.0),
    }
}
