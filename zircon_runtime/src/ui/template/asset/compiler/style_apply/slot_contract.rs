use std::collections::BTreeMap;

use toml::Value;
use zircon_runtime_interface::ui::template::UiTemplateNode;

use super::super::value_normalizer::merge_value_maps;
use super::{
    append_class, map_attribute, map_attribute_any, mui_collection_classes,
    mui_display_surface_classes, mui_form_classes, mui_layout_classes, mui_navigation_classes,
    mui_selection_classes, mui_surface_child_classes, mui_x_classes, nested_map_attribute_any,
    string_from_map, string_list_from_value, value_as_map,
};

pub(in super::super) fn apply_mui_root_slot_props_to_node(node: &mut UiTemplateNode) {
    let root_props = nested_map_attribute_any(node, &["mui_slot_props", "slotProps"], "root");
    if root_props.is_empty() {
        return;
    };
    merge_value_maps(&mut node.attributes, &root_props);
}

pub(in super::super) fn apply_mui_child_slot_props(node: &mut UiTemplateNode) {
    let slot_props = map_attribute_any(node, &["mui_slot_props", "slotProps"]);
    let slot_components = map_attribute_any(node, &["mui_slots", "slots"]);
    let slot_classes = map_attribute(node, "classes");
    let owner_prefix = node
        .component
        .as_deref()
        .filter(|component| !component.is_empty())
        .map(|component| format!("Mui{component}"));
    let owner_component = node.component.clone().unwrap_or_default();
    let owner_attributes = node.attributes.clone();
    if slot_props.is_empty()
        && slot_components.is_empty()
        && slot_classes.is_empty()
        && owner_prefix.is_none()
    {
        return;
    }

    for child in &mut node.children {
        if owner_component == "Skeleton" {
            mui_display_surface_classes::append_skeleton_child_metadata(child);
        }
        let Some(slot_name) = mui_slot_name(child) else {
            continue;
        };
        apply_mui_slot_contract_to_child(
            child,
            &slot_name,
            &owner_component,
            &owner_attributes,
            owner_prefix.as_deref(),
            &slot_props,
            &slot_components,
            &slot_classes,
        );
    }

    for (slot_name, children) in &mut node.slots {
        for child in children {
            if owner_component == "Skeleton" {
                mui_display_surface_classes::append_skeleton_child_metadata(child);
            }
            child
                .slot_attributes
                .entry("mui_slot".to_string())
                .or_insert_with(|| Value::String(slot_name.clone()));
            apply_mui_slot_contract_to_child(
                child,
                slot_name,
                &owner_component,
                &owner_attributes,
                owner_prefix.as_deref(),
                &slot_props,
                &slot_components,
                &slot_classes,
            );
        }
    }
}

fn apply_mui_slot_contract_to_child(
    child: &mut UiTemplateNode,
    slot_name: &str,
    owner_component: &str,
    owner_attributes: &BTreeMap<String, Value>,
    owner_prefix: Option<&str>,
    slot_props: &BTreeMap<String, Value>,
    slot_components: &BTreeMap<String, Value>,
    slot_classes: &BTreeMap<String, Value>,
) {
    if let Some(prefix) = owner_prefix {
        append_class(&mut child.classes, format!("{prefix}-{slot_name}"));
    }
    append_mui_owner_slot_utility_classes(child, owner_component, owner_attributes, slot_name);
    if let Some(component) = slot_components
        .get(slot_name)
        .and_then(Value::as_str)
        .filter(|component| !component.trim().is_empty())
    {
        let _ = child.attributes.insert(
            "mui_slot_component".to_string(),
            Value::String(component.trim().to_string()),
        );
    }
    if let Some(props) = slot_props.get(slot_name).and_then(value_as_map) {
        merge_value_maps(&mut child.attributes, &props);
    }
    for class_name in string_list_from_value(slot_classes.get(slot_name)) {
        append_class(&mut child.classes, class_name);
    }
}

fn append_mui_owner_slot_utility_classes(
    child: &mut UiTemplateNode,
    owner_component: &str,
    owner_attributes: &BTreeMap<String, Value>,
    slot_name: &str,
) {
    if mui_layout_classes::append_layout_slot_classes(
        child,
        owner_component,
        owner_attributes,
        slot_name,
    ) {
        return;
    }
    if mui_form_classes::append_form_slot_classes(
        child,
        owner_component,
        owner_attributes,
        slot_name,
    ) {
        return;
    }
    if mui_selection_classes::append_slot_classes(
        child,
        owner_component,
        owner_attributes,
        slot_name,
    ) {
        return;
    }
    if mui_collection_classes::append_slot_classes(
        child,
        owner_component,
        owner_attributes,
        slot_name,
    ) {
        return;
    }
    if mui_x_classes::append_slot_classes(child, owner_component, owner_attributes, slot_name) {
        return;
    }
    if mui_surface_child_classes::append_slot_classes(
        child,
        owner_component,
        owner_attributes,
        slot_name,
    ) {
        return;
    }
    if mui_display_surface_classes::append_slot_classes(
        child,
        owner_component,
        owner_attributes,
        slot_name,
    ) {
        return;
    }

    match (owner_component, slot_name) {
        ("BottomNavigationAction", "label") => {
            mui_navigation_classes::append_bottom_navigation_action_label_slot_classes(
                child,
                owner_attributes,
            )
        }
        ("StepConnector", "line") => {
            mui_navigation_classes::append_step_connector_line_slot_classes(child, owner_attributes)
        }
        ("StepLabel", "label" | "iconContainer" | "labelContainer") => {
            mui_navigation_classes::append_step_label_slot_classes(
                child,
                owner_attributes,
                slot_name,
            )
        }
        ("Tabs", "list" | "scroller" | "scrollButtons") => {
            mui_navigation_classes::append_tabs_slot_classes(child, owner_attributes, slot_name)
        }
        ("TransferList", "source" | "target" | "actions") => {
            mui_navigation_classes::append_transfer_list_slot_classes(
                child,
                owner_attributes,
                slot_name,
            )
        }
        _ => {}
    }
}

pub(in super::super) fn mui_slot_name(node: &UiTemplateNode) -> Option<String> {
    string_from_map(&node.slot_attributes, "mui_slot")
        .or_else(|| string_from_map(&node.attributes, "mui_slot"))
}
