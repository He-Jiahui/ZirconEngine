use std::collections::BTreeMap;

use super::super::super::pane_value_conversion::value_as_bool;

pub(super) fn projected_world_space_enabled(
    component: &str,
    attributes: &BTreeMap<String, toml::Value>,
) -> bool {
    attributes
        .get("world_space_enabled")
        .and_then(value_as_bool)
        .unwrap_or(component == "WorldSpaceSurface")
}
