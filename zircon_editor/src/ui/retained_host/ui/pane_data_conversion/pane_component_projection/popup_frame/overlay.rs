use std::collections::BTreeMap;

use toml::Value;

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
        || borrowed_popup_placement(attributes).is_some_and(|placement| placement.contains('-'))
}

fn borrowed_popup_placement(attributes: &BTreeMap<String, Value>) -> Option<&str> {
    attributes.get("placement").and_then(Value::as_str)
}

#[cfg(test)]
#[path = "overlay/borrowed_placement_tests.rs"]
mod borrowed_placement_tests;
