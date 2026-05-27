use std::collections::BTreeMap;

use toml::Value;
use zircon_runtime_interface::ui::template::UiTemplateNode;

use super::{
    append_class, bool_attribute, bool_attribute_any, pascal_case, string_attribute_any,
    string_from_attributes_any,
};

pub(super) fn append_component_classes(
    node: &mut UiTemplateNode,
    component: &str,
    prefix: &str,
) -> bool {
    match component {
        "ImageListItem" => append_image_list_item_classes(node, prefix),
        "ImageListItemBar" => append_image_list_item_bar_classes(node, prefix),
        "ListItem" => append_list_item_classes(node, prefix),
        "ListItemAvatar" | "ListItemIcon" => append_align_items_classes(node, prefix),
        "ListItemButton" => append_list_item_button_classes(node, prefix),
        "ListItemSecondaryAction" => append_list_item_secondary_action_classes(node, prefix),
        "ListItemText" => append_list_item_text_classes(node, prefix),
        "ListSubheader" => append_list_subheader_classes(node, prefix),
        "TableCell" => append_table_cell_classes(node, prefix),
        "TableRow" => append_table_row_classes(node, prefix),
        "TableSortLabel" => append_table_sort_label_classes(node, prefix),
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
    match (owner_component, slot_name) {
        ("TableSortLabel", "icon") => {
            append_class(&mut child.classes, "MuiTableSortLabel-icon".to_string())
        }
        ("ImageListItemBar", "titleWrap" | "title" | "subtitle" | "actionIcon") => append_class(
            &mut child.classes,
            format!("MuiImageListItemBar-{slot_name}"),
        ),
        ("ListItemText", "primary" | "secondary")
        | ("ImageListItem", "img")
        | ("ListItem", "secondaryAction")
        | ("TablePagination", _) => {}
        _ => return false,
    }

    if owner_component == "TablePagination" {
        append_class(
            &mut child.classes,
            format!("MuiTablePagination-{slot_name}"),
        );
    }
    if owner_component == "ListItemText" && has_text_slot(owner_attributes, slot_name) {
        append_class(&mut child.classes, format!("MuiListItemText-{slot_name}"));
    }
    true
}

pub(super) fn suppresses_generic_classes(component: &str) -> bool {
    matches!(
        component,
        "ImageListItem"
            | "ImageListItemBar"
            | "ListItem"
            | "ListItemAvatar"
            | "ListItemButton"
            | "ListItemIcon"
            | "ListItemSecondaryAction"
            | "ListItemText"
            | "ListSubheader"
            | "TableBody"
            | "TableCell"
            | "TableContainer"
            | "TableFooter"
            | "TableHead"
            | "TablePagination"
            | "TablePaginationActions"
            | "TableRow"
            | "TableSortLabel"
    )
}

fn append_list_item_classes(node: &mut UiTemplateNode, prefix: &str) {
    append_list_density_classes(node, prefix);
    if !bool_attribute_any(node, &["disablePadding", "disable_padding"]) {
        append_class(&mut node.classes, format!("{prefix}-padding"));
    }
}

fn append_list_item_button_classes(node: &mut UiTemplateNode, prefix: &str) {
    append_list_density_classes(node, prefix);
    if bool_attribute(node, "disabled") {
        append_class(&mut node.classes, format!("{prefix}-disabled"));
    }
    if bool_attribute(node, "selected") {
        append_class(&mut node.classes, format!("{prefix}-selected"));
    }
    if bool_attribute_any(node, &["focusVisible", "focus_visible", "focused"]) {
        append_class(&mut node.classes, format!("{prefix}-focusVisible"));
    }
}

fn append_list_density_classes(node: &mut UiTemplateNode, prefix: &str) {
    if bool_attribute(node, "dense") {
        append_class(&mut node.classes, format!("{prefix}-dense"));
    }
    if !bool_attribute_any(node, &["disableGutters", "disable_gutters"]) {
        append_class(&mut node.classes, format!("{prefix}-gutters"));
    }
    if bool_attribute(node, "divider") {
        append_class(&mut node.classes, format!("{prefix}-divider"));
    }
    append_align_items_classes(node, prefix);
}

fn append_align_items_classes(node: &mut UiTemplateNode, prefix: &str) {
    if string_attribute_any(node, &["alignItems", "align_items"]).as_deref() == Some("flex-start") {
        append_class(&mut node.classes, format!("{prefix}-alignItemsFlexStart"));
    }
}

fn append_list_item_secondary_action_classes(node: &mut UiTemplateNode, prefix: &str) {
    if bool_attribute_any(node, &["disableGutters", "disable_gutters"]) {
        append_class(&mut node.classes, format!("{prefix}-disableGutters"));
    }
}

fn append_list_item_text_classes(node: &mut UiTemplateNode, prefix: &str) {
    if bool_attribute(node, "inset") {
        append_class(&mut node.classes, format!("{prefix}-inset"));
    }
    if bool_attribute(node, "dense") {
        append_class(&mut node.classes, format!("{prefix}-dense"));
    }
    if has_text_node(node, &["primary", "text"]) && has_text_node(node, &["secondary"]) {
        append_class(&mut node.classes, format!("{prefix}-multiline"));
    }
}

fn append_list_subheader_classes(node: &mut UiTemplateNode, prefix: &str) {
    if let Some(color) = string_attribute_any(node, &["color"]).filter(|value| value != "default") {
        append_class(
            &mut node.classes,
            format!("{prefix}-color{}", pascal_case(&color)),
        );
    }
    if !bool_attribute_any(node, &["disableGutters", "disable_gutters"]) {
        append_class(&mut node.classes, format!("{prefix}-gutters"));
    }
    if bool_attribute(node, "inset") {
        append_class(&mut node.classes, format!("{prefix}-inset"));
    }
    if !bool_attribute_any(node, &["disableSticky", "disable_sticky"]) {
        append_class(&mut node.classes, format!("{prefix}-sticky"));
    }
}

fn append_image_list_item_classes(node: &mut UiTemplateNode, prefix: &str) {
    let variant =
        string_attribute_any(node, &["variant"]).unwrap_or_else(|| "standard".to_string());
    append_class(&mut node.classes, format!("{prefix}-{variant}"));
}

fn append_image_list_item_bar_classes(node: &mut UiTemplateNode, prefix: &str) {
    let position =
        string_attribute_any(node, &["position"]).unwrap_or_else(|| "bottom".to_string());
    let action_position = string_attribute_any(node, &["actionPosition", "action_position"])
        .unwrap_or_else(|| "right".to_string());
    append_class(
        &mut node.classes,
        format!("{prefix}-position{}", pascal_case(&position)),
    );
    append_class(
        &mut node.classes,
        format!("{prefix}-actionPosition{}", pascal_case(&action_position)),
    );
}

fn append_table_cell_classes(node: &mut UiTemplateNode, prefix: &str) {
    let variant = string_attribute_any(node, &["variant"]).unwrap_or_else(|| "body".to_string());
    append_class(&mut node.classes, format!("{prefix}-{variant}"));
    if bool_attribute_any(node, &["stickyHeader", "sticky_header"]) {
        append_class(&mut node.classes, format!("{prefix}-stickyHeader"));
    }
    if let Some(align) = string_attribute_any(node, &["align"]).filter(|value| value != "inherit") {
        append_class(
            &mut node.classes,
            format!("{prefix}-align{}", pascal_case(&align)),
        );
    }
    if let Some(padding) =
        string_attribute_any(node, &["padding"]).filter(|value| value != "normal")
    {
        append_class(
            &mut node.classes,
            format!("{prefix}-padding{}", pascal_case(&padding)),
        );
    }
    let size = string_attribute_any(node, &["size"]).unwrap_or_else(|| "medium".to_string());
    append_class(
        &mut node.classes,
        format!("{prefix}-size{}", pascal_case(&size)),
    );
}

fn append_table_row_classes(node: &mut UiTemplateNode, prefix: &str) {
    if bool_attribute(node, "selected") {
        append_class(&mut node.classes, format!("{prefix}-selected"));
    }
    if bool_attribute_any(node, &["hover", "hovered"]) {
        append_class(&mut node.classes, format!("{prefix}-hover"));
    }
    match string_attribute_any(node, &["variant"]).as_deref() {
        Some("head") => append_class(&mut node.classes, format!("{prefix}-head")),
        Some("footer") => append_class(&mut node.classes, format!("{prefix}-footer")),
        _ => {}
    }
}

fn append_table_sort_label_classes(node: &mut UiTemplateNode, prefix: &str) {
    if bool_attribute(node, "active") {
        append_class(&mut node.classes, format!("{prefix}-active"));
    }
    let direction = string_attribute_any(node, &["direction"]).unwrap_or_else(|| "asc".to_string());
    append_class(
        &mut node.classes,
        format!("{prefix}-direction{}", pascal_case(&direction)),
    );
}

fn has_text_node(node: &UiTemplateNode, attributes: &[&str]) -> bool {
    string_attribute_any(node, attributes).is_some_and(|value| !value.is_empty())
}

fn has_text_slot(owner_attributes: &BTreeMap<String, Value>, slot_name: &str) -> bool {
    match slot_name {
        "primary" => string_from_attributes_any(owner_attributes, &["primary", "text"])
            .is_some_and(|value| !value.is_empty()),
        "secondary" => string_from_attributes_any(owner_attributes, &["secondary"])
            .is_some_and(|value| !value.is_empty()),
        _ => false,
    }
}
