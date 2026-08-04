use std::collections::BTreeMap;

use crate::ui::template_runtime::RetainedUiHostNodeModel;

use super::super::pane_data_conversion::projected_notification_center_value_text;
use super::properties::first_string_property;

pub(super) fn projected_workbench_text(
    node: &RetainedUiHostNodeModel,
    component_role: &str,
) -> String {
    let authored_text = first_string_property(&node.properties, &["text", "label"]);
    let authored_text = authored_text.or_else(|| {
        matches!(node.component.as_str(), "SearchField")
            .then(|| first_string_property(&node.properties, &["placeholder"]))
            .flatten()
    });
    if prefers_authored_text_over_rendered_text(node.component.as_str(), component_role) {
        authored_text
            .or_else(|| node.text.clone())
            .unwrap_or_default()
    } else {
        node.text.clone().or(authored_text).unwrap_or_default()
    }
}

fn prefers_authored_text_over_rendered_text(component: &str, component_role: &str) -> bool {
    matches!(
        component_role,
        "button"
            | "toggle"
            | "tab"
            | "tabs"
            | "tab-list"
            | "segmented-control"
            | "checkbox"
            | "radio"
            | "icon-button"
    ) || matches!(
        component,
        "Button"
            | "Toggle"
            | "ToggleButton"
            | "Switch"
            | "Checkbox"
            | "Radio"
            | "RadioField"
            | "SegmentedControl"
            | "Tab"
            | "Tabs"
            | "TabList"
            | "IconButton"
    )
}

pub(super) fn projected_workbench_value_text(
    node: &RetainedUiHostNodeModel,
    component_role: &str,
    button_style_values: &BTreeMap<String, toml::Value>,
) -> String {
    display_node_value_text(node, component_role)
        .or_else(|| projected_notification_center_value_text(component_role, button_style_values))
        .or_else(|| first_string_property(&node.properties, &["value_text"]))
        .or_else(|| display_value_property_for_node(node, component_role))
        .unwrap_or_default()
}

fn display_node_value_text(node: &RetainedUiHostNodeModel, component_role: &str) -> Option<String> {
    if !uses_value_property_as_display_text(node.component.as_str(), component_role) {
        return None;
    }

    node.value_text.clone()
}

fn display_value_property_for_node(
    node: &RetainedUiHostNodeModel,
    component_role: &str,
) -> Option<String> {
    if !uses_value_property_as_display_text(node.component.as_str(), component_role) {
        return None;
    }

    first_string_property(&node.properties, &["value"])
}

fn uses_value_property_as_display_text(component: &str, component_role: &str) -> bool {
    matches!(
        component_role,
        "input-field"
            | "number-field"
            | "range-field"
            | "slider"
            | "range-slider"
            | "segmented-control"
            | "combo-box"
            | "dropdown"
            | "enum-field"
            | "flags-field"
            | "search-select"
            | "asset-field"
            | "object-field"
            | "instance-field"
            | "property-row"
    ) || matches!(
        component,
        "InputField"
            | "TextField"
            | "LineEdit"
            | "NumberField"
            | "RangeField"
            | "Slider"
            | "RangeSlider"
            | "SegmentedControl"
            | "ComboBox"
            | "Dropdown"
            | "EnumField"
            | "FlagsField"
            | "SearchSelect"
            | "AssetField"
            | "ObjectField"
            | "InstanceField"
            | "PropertyRow"
    )
}

pub(super) fn resolve_workbench_role(component: &str) -> &'static str {
    match component {
        "Button" => "Button",
        "IconButton" => "IconButton",
        "ComboBox" | "Dropdown" | "SearchSelect" => "Dropdown",
        "ContextActionMenu" | "ContextMenu" | "Menu" | "PopupMenu" => "Menu",
        "InputField" | "TextField" | "NumberField" => "InputField",
        "SearchField" => "SearchField",
        "Checkbox" => "Checkbox",
        "Radio" => "Radio",
        "RangeField" | "Slider" => "Slider",
        "Toggle" | "Switch" => "Toggle",
        "Table" | "EditableTable" => "Table",
        "Image" => "Image",
        "SvgIcon" => "SvgIcon",
        "Icon" => "Icon",
        "Tooltip" => "Tooltip",
        "NotificationCenter" => "NotificationCenter",
        "Label" | "Text" => "Label",
        _ => "Mount",
    }
}

pub(super) fn default_workbench_surface_variant(
    component: &str,
    component_role: &str,
) -> Option<String> {
    match (component, component_role) {
        (_, "button") | ("Button", _) | ("IconButton", _) => Some("panel".to_string()),
        ("InputField", _) | ("TextField", _) | ("NumberField", _) | ("SearchField", _) => {
            Some("inset".to_string())
        }
        ("Label", _) | ("Text", _) => None,
        _ => Some("panel".to_string()),
    }
}

pub(super) fn is_workbench_command_palette_mount(component: &str, control_id: &str) -> bool {
    component == "WorkbenchCommandPalette" || control_id == "WorkbenchCommandPalette"
}

pub(super) fn is_workbench_notification_center_mount(component: &str, control_id: &str) -> bool {
    component == "NotificationCenter"
        || component == "WorkbenchNotificationCenter"
        || control_id == "WorkbenchNotificationCenter"
}

pub(super) fn default_text_tone(
    component: &str,
    component_role: &str,
    surface_variant: &str,
) -> String {
    if matches!(component, "Image" | "SvgIcon" | "Icon" | "IconButton") {
        "muted".to_string()
    } else if matches!(component_role, "button") || matches!(surface_variant, "accent" | "primary")
    {
        "primary".to_string()
    } else {
        String::new()
    }
}

pub(super) fn default_corner_radius(component: &str, component_role: &str) -> f64 {
    match (component, component_role) {
        ("Button", _)
        | ("IconButton", _)
        | ("InputField", _)
        | ("SearchField", _)
        | (_, "button") => 5.0,
        ("Label", _) | ("Text", _) => 0.0,
        _ => 4.0,
    }
}

pub(super) fn default_border_width(
    component: &str,
    component_role: &str,
    surface_variant: &str,
) -> Option<f64> {
    if matches!(component, "Label" | "Text") && surface_variant.is_empty() {
        return None;
    }
    if matches!(component_role, "button")
        || matches!(
            component,
            "Button" | "IconButton" | "InputField" | "TextField" | "NumberField" | "SearchField"
        )
        || !surface_variant.is_empty()
    {
        Some(1.0)
    } else {
        None
    }
}
