use std::collections::BTreeMap;

use crate::ui::template_runtime::{RetainedUiHostNodeModel, RetainedUiHostValue};

use super::properties::{bool_property, first_string_property};

const WORKBENCH_SELECTION_SELECTED_SURFACE: &str = "#173942";
const WORKBENCH_SELECTION_ACCENT: &str = "#2aa6b8";
const WORKBENCH_RADIO_SELECTED_SURFACE: &str = "#1b272d";
const WORKBENCH_RADIO_SELECTED_BORDER: &str = "#4c5b63";
const WORKBENCH_TOGGLE_SELECTED_BORDER: &str = "#414b54";
const WORKBENCH_TOGGLE_SELECTED_THUMB: &str = "#a4aeb4";

pub(super) fn is_cleared_inspector_property_row(
    control_id: &str,
    properties: &BTreeMap<String, RetainedUiHostValue>,
) -> bool {
    if !matches!(
        control_id,
        "WorkbenchMeshRow"
            | "WorkbenchMaterialRow"
            | "WorkbenchComponentPropertySlot03Row"
            | "WorkbenchComponentPropertySlot04Row"
    ) && !control_id.starts_with("WorkbenchComponentPropertyVirtualRow")
    {
        return false;
    }

    first_string_property(properties, &["text"]).is_none_or(|text| text.is_empty())
        && first_string_property(properties, &["value_text"]).is_none_or(|value| value.is_empty())
}

pub(super) fn clear_button_surface_style_values(values: &mut BTreeMap<String, toml::Value>) {
    for key in ["background", "background_color", "border", "border_color"] {
        values.remove(key);
    }
}

pub(super) fn normalize_workbench_selection_control_style_values(
    values: &mut BTreeMap<String, toml::Value>,
    node: &RetainedUiHostNodeModel,
    component_role: &str,
) {
    if !active_workbench_selection_control(node) {
        return;
    }

    if is_workbench_checkbox_control(node, component_role) {
        set_toml_string_aliases(
            values,
            &["background", "background_color"],
            WORKBENCH_SELECTION_SELECTED_SURFACE,
        );
        set_toml_string_aliases(
            values,
            &["border", "border_color"],
            WORKBENCH_SELECTION_ACCENT,
        );
    } else if is_workbench_radio_control(node, component_role) {
        set_toml_string_aliases(
            values,
            &["background", "background_color"],
            WORKBENCH_RADIO_SELECTED_SURFACE,
        );
        set_toml_string_aliases(
            values,
            &["border", "border_color"],
            WORKBENCH_RADIO_SELECTED_BORDER,
        );
    } else if is_workbench_toggle_control(node, component_role) {
        set_toml_string_aliases(
            values,
            &["background", "background_color"],
            WORKBENCH_SELECTION_SELECTED_SURFACE,
        );
        set_toml_string_aliases(
            values,
            &["border", "border_color"],
            WORKBENCH_TOGGLE_SELECTED_BORDER,
        );
        set_toml_string_aliases(
            values,
            &["foreground", "foreground_color"],
            WORKBENCH_TOGGLE_SELECTED_THUMB,
        );
    }
}

fn active_workbench_selection_control(node: &RetainedUiHostNodeModel) -> bool {
    node.checked
        || bool_property(&node.properties, "checked")
        || bool_property(&node.properties, "selected")
}

fn is_workbench_checkbox_control(node: &RetainedUiHostNodeModel, component_role: &str) -> bool {
    component_role == "checkbox"
        || matches!(node.component.as_str(), "Checkbox" | "WorkbenchCheckbox")
        || node
            .control_id
            .as_deref()
            .is_some_and(|control_id| control_id.contains("Checkbox"))
}

fn is_workbench_radio_control(node: &RetainedUiHostNodeModel, component_role: &str) -> bool {
    component_role == "radio"
        || matches!(node.component.as_str(), "Radio" | "WorkbenchRadio")
        || node
            .control_id
            .as_deref()
            .is_some_and(|control_id| control_id.contains("Radio"))
}

fn is_workbench_toggle_control(node: &RetainedUiHostNodeModel, component_role: &str) -> bool {
    component_role == "toggle"
        || matches!(
            node.component.as_str(),
            "Toggle" | "Switch" | "WorkbenchToggle" | "WorkbenchSwitch"
        )
        || node
            .control_id
            .as_deref()
            .is_some_and(|control_id| control_id.contains("Toggle"))
}

fn set_toml_string_aliases(values: &mut BTreeMap<String, toml::Value>, keys: &[&str], value: &str) {
    for key in keys {
        match values.get_mut(*key) {
            Some(current) if current.as_str() == Some(value) => {}
            Some(current) => *current = toml::Value::String(value.to_string()),
            None => {
                values.insert((*key).to_string(), toml::Value::String(value.to_string()));
            }
        }
    }
}

#[cfg(test)]
#[path = "selection_style/stable_alias_tests.rs"]
mod stable_alias_tests;
