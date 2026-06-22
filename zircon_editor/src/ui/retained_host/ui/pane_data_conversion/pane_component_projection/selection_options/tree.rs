use std::collections::BTreeMap;

use super::super::super::pane_value_conversion::value_as_f64;
use super::super::attribute_values::value_as_i32;

pub(super) struct ProjectedTreeState {
    pub(super) depth: i32,
    pub(super) indent_px: f32,
}

pub(super) fn projected_tree_state(
    attributes: &BTreeMap<String, toml::Value>,
) -> ProjectedTreeState {
    let depth = attributes
        .get("tree_depth")
        .and_then(value_as_i32)
        .unwrap_or(0);
    let indent_px = attributes
        .get("tree_indent_px")
        .and_then(value_as_f64)
        .unwrap_or_else(|| f64::from(depth) * 12.0) as f32;

    ProjectedTreeState { depth, indent_px }
}
