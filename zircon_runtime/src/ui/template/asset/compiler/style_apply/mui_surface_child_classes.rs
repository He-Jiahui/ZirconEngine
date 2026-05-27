use std::collections::BTreeMap;

use toml::Value;
use zircon_runtime_interface::ui::template::UiTemplateNode;

use super::{
    append_class, bool_attribute, bool_attribute_any, bool_from_attributes_any, pascal_case,
    string_attribute_any, string_from_attributes_any,
};

pub(super) fn append_component_classes(
    node: &mut UiTemplateNode,
    component: &str,
    prefix: &str,
) -> bool {
    match component {
        "AccordionActions" => append_spacing_class(node, prefix),
        "AccordionSummary" => append_accordion_summary_classes(node, prefix),
        "DialogActions" => append_spacing_class(node, prefix),
        "DialogContent" => append_dialog_content_classes(node, prefix),
        "MobileStepper" => append_mobile_stepper_classes(node, prefix),
        "SpeedDialAction" => append_speed_dial_action_classes(node, prefix),
        "SwipeableDrawer" => append_swipeable_drawer_classes(node),
        "TabScrollButton" => append_tab_scroll_button_classes(node, prefix),
        "AccordionDetails" | "DialogContentText" | "DialogTitle" | "MenuList" | "SpeedDialIcon" => {
        }
        _ => return false,
    }
    true
}

pub(super) fn append_slot_classes(
    child: &mut UiTemplateNode,
    owner_component: &str,
    owner_attributes: &BTreeMap<String, Value>,
    slot_name: &str,
) -> bool {
    match owner_component {
        "AccordionSummary" if matches!(slot_name, "content" | "expandIconWrapper") => {
            append_accordion_summary_slot_classes(child, owner_attributes)
        }
        "MobileStepper" if matches!(slot_name, "dots" | "dot" | "dotActive" | "progress") => {
            append_mobile_stepper_slot_classes(child, owner_attributes, slot_name)
        }
        "SpeedDialAction" if matches!(slot_name, "fab" | "staticTooltipLabel") => {
            append_speed_dial_action_slot_classes(child, owner_attributes, slot_name)
        }
        "SpeedDialIcon" if matches!(slot_name, "icon" | "openIcon") => {
            append_speed_dial_icon_slot_classes(child, owner_attributes, slot_name)
        }
        "SwipeableDrawer" if matches!(slot_name, "root" | "docked" | "paper" | "swipeArea") => {
            append_swipeable_drawer_slot_classes(child, owner_attributes, slot_name)
        }
        _ => return false,
    }
    true
}

pub(super) fn suppresses_generic_classes(component: &str) -> bool {
    matches!(
        component,
        "AccordionActions"
            | "AccordionDetails"
            | "AccordionSummary"
            | "DialogActions"
            | "DialogContent"
            | "DialogContentText"
            | "DialogTitle"
            | "MenuList"
            | "MobileStepper"
            | "SpeedDialAction"
            | "SpeedDialIcon"
            | "SwipeableDrawer"
            | "TabScrollButton"
    )
}

fn append_spacing_class(node: &mut UiTemplateNode, prefix: &str) {
    if !bool_attribute_any(node, &["disableSpacing", "disable_spacing"]) {
        append_class(&mut node.classes, format!("{prefix}-spacing"));
    }
}

fn append_accordion_summary_classes(node: &mut UiTemplateNode, prefix: &str) {
    if !bool_attribute_any(node, &["disableGutters", "disable_gutters"]) {
        append_class(&mut node.classes, format!("{prefix}-gutters"));
    }
    if bool_attribute(node, "expanded") {
        append_class(&mut node.classes, format!("{prefix}-expanded"));
    }
    if bool_attribute(node, "disabled") {
        append_class(&mut node.classes, format!("{prefix}-disabled"));
    }
    if bool_attribute_any(node, &["focusVisible", "focus_visible", "focused"]) {
        append_class(&mut node.classes, format!("{prefix}-focusVisible"));
        for class_name in super::string_list_attribute(node, "focusVisibleClassName") {
            append_class(&mut node.classes, class_name);
        }
    }
}

fn append_dialog_content_classes(node: &mut UiTemplateNode, prefix: &str) {
    if bool_attribute(node, "dividers") {
        append_class(&mut node.classes, format!("{prefix}-dividers"));
    }
}

fn append_mobile_stepper_classes(node: &mut UiTemplateNode, prefix: &str) {
    let position = string_attribute_any(node, &["position"])
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "bottom".to_string());
    append_class(
        &mut node.classes,
        format!("{prefix}-position{}", pascal_case(&position)),
    );
}

fn append_speed_dial_action_classes(node: &mut UiTemplateNode, prefix: &str) {
    if !bool_attribute_any(node, &["tooltipOpen", "tooltip_open"]) {
        return;
    }

    append_class(&mut node.classes, format!("{prefix}-staticTooltip"));
    if !bool_attribute(node, "open") {
        append_class(&mut node.classes, format!("{prefix}-staticTooltipClosed"));
    }

    let placement = string_attribute_any(node, &["tooltipPlacement", "tooltip_placement"])
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "left".to_string());
    append_class(
        &mut node.classes,
        format!("{prefix}-tooltipPlacement{}", pascal_case(&placement)),
    );
}

fn append_swipeable_drawer_classes(node: &mut UiTemplateNode) {
    let anchor = drawer_anchor(&node.attributes);
    let variant = drawer_variant(&node.attributes);
    append_drawer_root_classes(&mut node.classes, &anchor, &variant);
}

fn append_tab_scroll_button_classes(node: &mut UiTemplateNode, prefix: &str) {
    let orientation = string_attribute_any(node, &["orientation"])
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "horizontal".to_string());
    append_class(&mut node.classes, format!("{prefix}-{orientation}"));
    if bool_attribute(node, "disabled") {
        append_class(&mut node.classes, format!("{prefix}-disabled"));
    }
}

fn append_accordion_summary_slot_classes(
    child: &mut UiTemplateNode,
    owner_attributes: &BTreeMap<String, Value>,
) {
    if bool_from_attributes_any(owner_attributes, &["expanded"]) {
        append_class(&mut child.classes, "Mui-expanded".to_string());
    }
}

fn append_mobile_stepper_slot_classes(
    child: &mut UiTemplateNode,
    _owner_attributes: &BTreeMap<String, Value>,
    slot_name: &str,
) {
    if slot_name == "dotActive" {
        append_class(&mut child.classes, "MuiMobileStepper-dot".to_string());
        append_class(&mut child.classes, "MuiMobileStepper-dotActive".to_string());
    } else if slot_name == "dot"
        && bool_attribute_any(child, &["active", "dotActive", "dot_active", "selected"])
    {
        append_class(&mut child.classes, "MuiMobileStepper-dotActive".to_string());
    }
}

fn append_speed_dial_action_slot_classes(
    child: &mut UiTemplateNode,
    owner_attributes: &BTreeMap<String, Value>,
    slot_name: &str,
) {
    match slot_name {
        "fab" if !bool_from_attributes_any(owner_attributes, &["open"]) => {
            append_class(
                &mut child.classes,
                "MuiSpeedDialAction-fabClosed".to_string(),
            );
        }
        "staticTooltipLabel" => {
            append_class(
                &mut child.classes,
                "MuiSpeedDialAction-staticTooltipLabel".to_string(),
            );
        }
        _ => {}
    }
}

fn append_speed_dial_icon_slot_classes(
    child: &mut UiTemplateNode,
    owner_attributes: &BTreeMap<String, Value>,
    slot_name: &str,
) {
    let open = bool_from_attributes_any(owner_attributes, &["open"]);
    match slot_name {
        "icon" if open => {
            append_class(&mut child.classes, "MuiSpeedDialIcon-iconOpen".to_string());
            if has_open_icon(owner_attributes) {
                append_class(
                    &mut child.classes,
                    "MuiSpeedDialIcon-iconWithOpenIconOpen".to_string(),
                );
            }
        }
        "openIcon" if open => {
            append_class(
                &mut child.classes,
                "MuiSpeedDialIcon-openIconOpen".to_string(),
            );
        }
        _ => {}
    }
}

fn append_swipeable_drawer_slot_classes(
    child: &mut UiTemplateNode,
    owner_attributes: &BTreeMap<String, Value>,
    slot_name: &str,
) {
    let anchor = drawer_anchor(owner_attributes);
    let variant = drawer_variant(owner_attributes);
    match slot_name {
        "root" => append_drawer_root_classes(&mut child.classes, &anchor, &variant),
        "docked" if variant != "temporary" => {
            append_class(&mut child.classes, "MuiDrawer-docked".to_string());
        }
        "paper" => append_class(&mut child.classes, "MuiDrawer-paper".to_string()),
        "swipeArea" => {
            append_class(&mut child.classes, "PrivateSwipeArea-root".to_string());
            append_class(
                &mut child.classes,
                format!("PrivateSwipeArea-anchor{}", pascal_case(&anchor)),
            );
        }
        _ => {}
    }
}

fn append_drawer_root_classes(classes: &mut Vec<String>, anchor: &str, variant: &str) {
    append_class(classes, "MuiDrawer-root".to_string());
    append_class(classes, format!("MuiDrawer-anchor{}", pascal_case(anchor)));
    if variant == "temporary" {
        append_class(classes, "MuiDrawer-modal".to_string());
    } else {
        append_class(classes, "MuiDrawer-docked".to_string());
    }
}

fn drawer_anchor(attributes: &BTreeMap<String, Value>) -> String {
    string_from_attributes_any(attributes, &["anchor"]).unwrap_or_else(|| "left".to_string())
}

fn drawer_variant(attributes: &BTreeMap<String, Value>) -> String {
    string_from_attributes_any(attributes, &["variant", "mui_variant"])
        .unwrap_or_else(|| "temporary".to_string())
}

fn has_open_icon(attributes: &BTreeMap<String, Value>) -> bool {
    string_from_attributes_any(attributes, &["openIcon", "open_icon"])
        .is_some_and(|value| !value.is_empty())
}
