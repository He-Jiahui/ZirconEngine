use std::collections::BTreeMap;

use toml::Value;
use zircon_runtime_interface::ui::template::UiTemplateNode;

use super::{
    append_class, bool_attribute, bool_attribute_any, bool_from_attributes_any, mui_slot_name,
    pascal_case, string_attribute_any, string_from_attributes_any,
};

pub(super) fn append_bottom_navigation_action_classes(node: &mut UiTemplateNode, prefix: &str) {
    if is_bottom_navigation_action_icon_only(&node.attributes) {
        append_class(&mut node.classes, format!("{prefix}-iconOnly"));
    }
}

pub(super) fn append_link_classes(node: &mut UiTemplateNode, prefix: &str) {
    let underline = string_attribute_any(node, &["underline"])
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "always".to_string());
    append_class(
        &mut node.classes,
        format!("{prefix}-underline{}", pascal_case(&underline)),
    );

    if string_attribute_any(node, &["component"]).as_deref() == Some("button") {
        append_class(&mut node.classes, format!("{prefix}-button"));
    }
    if bool_attribute_any(node, &["focusVisible", "focus_visible"]) {
        append_class(&mut node.classes, "Mui-focusVisible".to_string());
    }
}

pub(super) fn append_menu_item_classes(node: &mut UiTemplateNode, prefix: &str) {
    if bool_attribute(node, "dense") {
        append_class(&mut node.classes, format!("{prefix}-dense"));
    }
    if !bool_attribute_any(node, &["disableGutters", "disable_gutters"]) {
        append_class(&mut node.classes, format!("{prefix}-gutters"));
    }
    if bool_attribute(node, "divider") {
        append_class(&mut node.classes, format!("{prefix}-divider"));
    }
}

pub(super) fn append_pagination_classes(node: &mut UiTemplateNode, prefix: &str) {
    let variant = string_attribute_any(node, &["variant", "mui_variant"])
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "text".to_string());
    append_class(&mut node.classes, format!("{prefix}-{variant}"));
}

pub(super) fn append_pagination_item_classes(node: &mut UiTemplateNode, prefix: &str) {
    let size = string_attribute_any(node, &["size", "mui_size"])
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "medium".to_string());
    if matches!(size.as_str(), "small" | "large") {
        append_class(
            &mut node.classes,
            format!("{prefix}-size{}", pascal_case(&size)),
        );
    }

    let variant = string_attribute_any(node, &["variant", "mui_variant"])
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "text".to_string());
    append_class(&mut node.classes, format!("{prefix}-{variant}"));

    if string_attribute_any(node, &["shape"]).as_deref() == Some("rounded") {
        append_class(&mut node.classes, format!("{prefix}-rounded"));
    }

    if let Some(color) = string_attribute_any(node, &["color", "mui_color"])
        .filter(|value| !value.is_empty() && value != "standard")
    {
        append_class(
            &mut node.classes,
            format!("{prefix}-color{}", pascal_case(&color)),
        );
    }

    let item_type = string_attribute_any(node, &["type"])
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "page".to_string());
    let class_suffix = match item_type.as_str() {
        "first" | "last" => Some("firstLast"),
        "next" | "previous" => Some("previousNext"),
        "start-ellipsis" | "end-ellipsis" => Some("ellipsis"),
        "page" => Some("page"),
        _ => None,
    };
    if let Some(class_suffix) = class_suffix {
        append_class(&mut node.classes, format!("{prefix}-{class_suffix}"));
    }
}

pub(super) fn append_stepper_classes(node: &mut UiTemplateNode, prefix: &str) {
    append_orientation_class(node, prefix);
    if bool_attribute_any(node, &["nonLinear", "non_linear"]) {
        append_class(&mut node.classes, format!("{prefix}-nonLinear"));
    }
    if bool_attribute_any(node, &["alternativeLabel", "alternative_label"]) {
        append_class(&mut node.classes, format!("{prefix}-alternativeLabel"));
    }
}

pub(super) fn append_step_classes(node: &mut UiTemplateNode, prefix: &str) {
    append_orientation_class(node, prefix);
    if bool_attribute_any(node, &["alternativeLabel", "alternative_label"]) {
        append_class(&mut node.classes, format!("{prefix}-alternativeLabel"));
    }
}

pub(super) fn append_step_button_classes(node: &mut UiTemplateNode, prefix: &str) {
    append_orientation_class(node, prefix);
}

pub(super) fn append_step_connector_classes(node: &mut UiTemplateNode, prefix: &str) {
    append_orientation_class(node, prefix);
    if bool_attribute_any(node, &["alternativeLabel", "alternative_label"]) {
        append_class(&mut node.classes, format!("{prefix}-alternativeLabel"));
    }
}

pub(super) fn append_step_content_classes(node: &mut UiTemplateNode, prefix: &str) {
    if bool_attribute(node, "last") {
        append_class(&mut node.classes, format!("{prefix}-last"));
    }
}

pub(super) fn append_step_label_classes(node: &mut UiTemplateNode, prefix: &str) {
    append_orientation_class(node, prefix);
    if bool_attribute_any(node, &["alternativeLabel", "alternative_label"]) {
        append_class(&mut node.classes, format!("{prefix}-alternativeLabel"));
    }
}

pub(super) fn append_tabs_classes(node: &mut UiTemplateNode, prefix: &str) {
    if string_attribute_any(node, &["orientation"]).as_deref() == Some("vertical") {
        append_class(&mut node.classes, format!("{prefix}-vertical"));
    }
}

pub(super) fn append_transfer_list_classes(node: &mut UiTemplateNode, prefix: &str) {
    if array_attribute_any_non_empty(node, &["source_items", "sourceItems"]) {
        append_class(&mut node.classes, format!("{prefix}-hasSourceItems"));
    }
    if array_attribute_any_non_empty(node, &["target_items", "targetItems"]) {
        append_class(&mut node.classes, format!("{prefix}-hasTargetItems"));
    }
    if array_attribute_any_non_empty(
        node,
        &[
            "selected_items",
            "selectedItems",
            "source_selected_items",
            "sourceSelectedItems",
            "target_selected_items",
            "targetSelectedItems",
        ],
    ) {
        append_class(&mut node.classes, format!("{prefix}-hasSelectedItems"));
    }
    if array_attribute_any_non_empty(node, &["disabled_items", "disabledItems"]) {
        append_class(&mut node.classes, format!("{prefix}-hasDisabledItems"));
    }
    if array_attribute_any_non_empty(node, &["disabled_actions", "disabledActions"]) {
        append_class(&mut node.classes, format!("{prefix}-hasDisabledActions"));
    }
}

pub(super) fn append_tab_classes(node: &mut UiTemplateNode, prefix: &str) {
    if tab_has_icon(node) && tab_has_label(node) {
        append_class(&mut node.classes, format!("{prefix}-labelIcon"));
    }
    let text_color = string_attribute_any(node, &["textColor", "text_color"])
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "inherit".to_string());
    append_class(
        &mut node.classes,
        format!("{prefix}-textColor{}", pascal_case(&text_color)),
    );
    if bool_attribute_any(node, &["fullWidth", "full_width"]) {
        append_class(&mut node.classes, format!("{prefix}-fullWidth"));
    }
    if bool_attribute(node, "wrapped") {
        append_class(&mut node.classes, format!("{prefix}-wrapped"));
    }
}

pub(super) fn append_bottom_navigation_action_label_slot_classes(
    child: &mut UiTemplateNode,
    owner_attributes: &BTreeMap<String, Value>,
) {
    if is_bottom_navigation_action_icon_only(owner_attributes) {
        append_class(
            &mut child.classes,
            "MuiBottomNavigationAction-iconOnly".to_string(),
        );
    }
    append_owner_state_classes(child, owner_attributes, &["selected"]);
}

pub(super) fn append_step_connector_line_slot_classes(
    child: &mut UiTemplateNode,
    owner_attributes: &BTreeMap<String, Value>,
) {
    let orientation = string_from_attributes_any(owner_attributes, &["orientation"])
        .unwrap_or_else(|| "horizontal".to_string());
    append_class(
        &mut child.classes,
        format!("MuiStepConnector-line{}", pascal_case(&orientation)),
    );
}

pub(super) fn append_step_label_slot_classes(
    child: &mut UiTemplateNode,
    owner_attributes: &BTreeMap<String, Value>,
    slot_name: &str,
) {
    if matches!(slot_name, "label" | "iconContainer") {
        append_owner_state_classes(
            child,
            owner_attributes,
            &["active", "completed", "disabled", "error"],
        );
    }
    if bool_from_attributes_any(owner_attributes, &["alternativeLabel", "alternative_label"]) {
        append_class(
            &mut child.classes,
            "MuiStepLabel-alternativeLabel".to_string(),
        );
    }
}

pub(super) fn append_tabs_slot_classes(
    child: &mut UiTemplateNode,
    owner_attributes: &BTreeMap<String, Value>,
    slot_name: &str,
) {
    let orientation = string_from_attributes_any(owner_attributes, &["orientation"])
        .unwrap_or_else(|| "horizontal".to_string());
    let variant = string_from_attributes_any(owner_attributes, &["variant", "mui_variant"])
        .unwrap_or_else(|| "standard".to_string());

    match slot_name {
        "scroller" if variant == "scrollable" => {
            if orientation == "vertical" {
                append_class(&mut child.classes, "MuiTabs-scrollableY".to_string());
            } else {
                append_class(&mut child.classes, "MuiTabs-scrollableX".to_string());
            }
            if !bool_from_attributes_any(
                owner_attributes,
                &["visibleScrollbar", "visible_scrollbar"],
            ) {
                append_class(&mut child.classes, "MuiTabs-hideScrollbar".to_string());
            }
        }
        "scroller" => append_class(&mut child.classes, "MuiTabs-fixed".to_string()),
        "list" => {
            if orientation == "vertical" {
                append_class(&mut child.classes, "MuiTabs-vertical".to_string());
            }
            if bool_from_attributes_any(owner_attributes, &["centered"]) {
                append_class(&mut child.classes, "MuiTabs-centered".to_string());
            }
        }
        "scrollButtons" => {
            if !bool_from_attributes_any(owner_attributes, &["allowScrollButtonsMobile"]) {
                append_class(
                    &mut child.classes,
                    "MuiTabs-scrollButtonsHideMobile".to_string(),
                );
            }
        }
        _ => {}
    }
}

pub(super) fn append_transfer_list_slot_classes(
    child: &mut UiTemplateNode,
    owner_attributes: &BTreeMap<String, Value>,
    slot_name: &str,
) {
    match slot_name {
        "source" => {
            if array_attribute_any_non_empty_from_attributes(
                owner_attributes,
                &["source_items", "sourceItems"],
            ) {
                append_class(
                    &mut child.classes,
                    "MuiTransferList-sourcePopulated".to_string(),
                );
            }
            if array_attribute_any_non_empty_from_attributes(
                owner_attributes,
                &["source_selected_items", "sourceSelectedItems"],
            ) {
                append_class(
                    &mut child.classes,
                    "MuiTransferList-sourceSelected".to_string(),
                );
            }
        }
        "target" => {
            if array_attribute_any_non_empty_from_attributes(
                owner_attributes,
                &["target_items", "targetItems"],
            ) {
                append_class(
                    &mut child.classes,
                    "MuiTransferList-targetPopulated".to_string(),
                );
            }
            if array_attribute_any_non_empty_from_attributes(
                owner_attributes,
                &["target_selected_items", "targetSelectedItems"],
            ) {
                append_class(
                    &mut child.classes,
                    "MuiTransferList-targetSelected".to_string(),
                );
            }
        }
        "actions" => {
            if bool_from_attributes_any(owner_attributes, &["disabled"])
                || array_attribute_any_non_empty_from_attributes(
                    owner_attributes,
                    &["disabled_actions", "disabledActions"],
                )
            {
                append_class(
                    &mut child.classes,
                    "MuiTransferList-actionsDisabled".to_string(),
                );
            }
        }
        _ => {}
    }
}

fn append_orientation_class(node: &mut UiTemplateNode, prefix: &str) {
    let orientation = string_attribute_any(node, &["orientation"])
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "horizontal".to_string());
    append_class(&mut node.classes, format!("{prefix}-{orientation}"));
}

fn append_owner_state_classes(
    child: &mut UiTemplateNode,
    owner_attributes: &BTreeMap<String, Value>,
    state_names: &[&str],
) {
    for state_name in state_names {
        if bool_from_attributes_any(owner_attributes, &[*state_name]) {
            append_class(
                &mut child.classes,
                mui_state_class_name(*state_name).to_string(),
            );
        }
    }
}

fn is_bottom_navigation_action_icon_only(attributes: &BTreeMap<String, Value>) -> bool {
    !bool_from_attributes_any(attributes, &["showLabel", "show_label"])
        && !bool_from_attributes_any(attributes, &["selected"])
}

fn array_attribute_any_non_empty(node: &UiTemplateNode, names: &[&str]) -> bool {
    names.iter().any(|name| {
        node.attributes
            .get(*name)
            .and_then(Value::as_array)
            .is_some_and(|values| !values.is_empty())
    })
}

fn array_attribute_any_non_empty_from_attributes(
    attributes: &BTreeMap<String, Value>,
    names: &[&str],
) -> bool {
    names.iter().any(|name| {
        attributes
            .get(*name)
            .and_then(Value::as_array)
            .is_some_and(|values| !values.is_empty())
    })
}

fn mui_state_class_name(state_name: &str) -> &'static str {
    match state_name {
        "active" => "Mui-active",
        "completed" => "Mui-completed",
        "disabled" => "Mui-disabled",
        "error" => "Mui-error",
        "focused" => "Mui-focused",
        "selected" => "Mui-selected",
        _ => unreachable!("known MUI state class"),
    }
}

fn tab_has_icon(node: &UiTemplateNode) -> bool {
    string_attribute_any(node, &["icon"]).is_some_and(|value| !value.is_empty())
        || node
            .children
            .iter()
            .any(|child| mui_slot_name(child).as_deref() == Some("icon"))
        || node
            .slots
            .get("icon")
            .is_some_and(|children| !children.is_empty())
}

fn tab_has_label(node: &UiTemplateNode) -> bool {
    string_attribute_any(node, &["label", "text"]).is_some_and(|value| !value.is_empty())
}
