use std::collections::BTreeMap;

use super::super::super::pane_value_conversion::value_as_options;
use super::super::collection_window::visible_collection_items;
use super::virtualization::ProjectedVirtualization;

pub(super) fn projected_collection_items(
    attributes: &BTreeMap<String, toml::Value>,
    virtualization: &ProjectedVirtualization,
) -> Vec<String> {
    let mut items = attributes
        .get("collection_items")
        .and_then(value_as_options)
        .unwrap_or_default();

    if virtualization.enabled {
        items = visible_collection_items(
            items,
            virtualization.visible_start,
            virtualization.visible_count,
            virtualization.overscan,
        );
    }

    items
}
