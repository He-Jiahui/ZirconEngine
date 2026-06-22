use std::collections::BTreeMap;

use super::super::super::pane_value_conversion::value_as_bool;

pub(super) struct ProjectedInteractionFlags {
    pub(super) checked: bool,
    pub(super) expanded: bool,
    pub(super) focused: bool,
    pub(super) hovered: bool,
    pub(super) pressed: bool,
    pub(super) dragging: bool,
    pub(super) enter_pressed: bool,
    pub(super) drop_hovered: bool,
    pub(super) active_drag_target: bool,
}

pub(super) fn projected_interaction_flags(
    attributes: &BTreeMap<String, toml::Value>,
) -> ProjectedInteractionFlags {
    ProjectedInteractionFlags {
        checked: attributes
            .get("checked")
            .or_else(|| attributes.get("value"))
            .and_then(value_as_bool)
            .unwrap_or(false),
        expanded: bool_attribute(attributes, "expanded"),
        focused: bool_attribute(attributes, "focused"),
        hovered: bool_attribute(attributes, "hovered"),
        pressed: bool_attribute(attributes, "pressed"),
        dragging: bool_attribute(attributes, "dragging"),
        enter_pressed: bool_attribute(attributes, "enter_pressed"),
        drop_hovered: bool_attribute(attributes, "drop_hovered"),
        active_drag_target: bool_attribute(attributes, "active_drag_target"),
    }
}

fn bool_attribute(attributes: &BTreeMap<String, toml::Value>, name: &str) -> bool {
    attributes
        .get(name)
        .and_then(value_as_bool)
        .unwrap_or(false)
}
