use std::collections::BTreeMap;

use super::super::super::pane_value_conversion::{value_as_bool, value_as_f64};
use super::super::attribute_values::value_as_i32;

pub(super) struct ProjectedVirtualization {
    pub(super) enabled: bool,
    pub(super) item_extent: f32,
    pub(super) overscan: i32,
    pub(super) total_count: i32,
    pub(super) visible_start: i32,
    pub(super) visible_count: i32,
}

pub(super) fn projected_virtualization(
    component: &str,
    attributes: &BTreeMap<String, toml::Value>,
) -> ProjectedVirtualization {
    ProjectedVirtualization {
        enabled: attributes
            .get("virtualization_enabled")
            .and_then(value_as_bool)
            .unwrap_or(component == "VirtualList"),
        item_extent: attributes
            .get("item_extent")
            .and_then(value_as_f64)
            .unwrap_or(0.0) as f32,
        overscan: attributes
            .get("overscan")
            .and_then(value_as_i32)
            .unwrap_or(0),
        total_count: attributes
            .get("total_count")
            .and_then(value_as_i32)
            .unwrap_or(0),
        visible_start: attributes
            .get("viewport_start")
            .and_then(value_as_i32)
            .unwrap_or(0),
        visible_count: attributes
            .get("viewport_count")
            .and_then(value_as_i32)
            .unwrap_or(0),
    }
}
