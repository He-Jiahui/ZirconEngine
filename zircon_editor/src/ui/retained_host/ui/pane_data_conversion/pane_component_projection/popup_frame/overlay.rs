use std::collections::BTreeMap;

use toml::Value;

use super::super::super::pane_value_conversion::value_as_string;

pub(super) fn is_anchor_positioned_overlay(component_role: &str) -> bool {
    matches!(
        component_role,
        "popover"
            | "popper"
            | "tooltip"
            | "menu"
            | "context-menu"
            | "context-action-menu"
            | "dropdown-popup"
            | "notification-center"
    )
}

pub(super) fn uses_popper_placement(
    component_role: &str,
    attributes: &BTreeMap<String, Value>,
) -> bool {
    matches!(component_role, "popper" | "tooltip")
        || attributes
            .get("placement")
            .and_then(value_as_string)
            .is_some_and(|placement| placement.contains('-'))
}
