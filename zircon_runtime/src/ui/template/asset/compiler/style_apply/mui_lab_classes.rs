use zircon_runtime_interface::ui::template::UiTemplateNode;

use super::{append_class, bool_attribute_any, mui_slot_name, pascal_case, string_attribute_any};

pub(super) fn append_component_classes(
    node: &mut UiTemplateNode,
    component: &str,
    prefix: &str,
) -> bool {
    match component {
        "TabPanel" => append_tab_panel_classes(node, prefix),
        "Timeline" => append_position_class(node, prefix, "right"),
        "TimelineContent" => append_position_class(node, prefix, "right"),
        "TimelineDot" => append_timeline_dot_classes(node, prefix),
        "TimelineItem" => append_timeline_item_classes(node, prefix),
        "TimelineOppositeContent" => append_position_class(node, prefix, "left"),
        "TabContext" | "TabList" | "TimelineConnector" | "TimelineSeparator" | "TreeItem" => {}
        _ => return false,
    }
    true
}

pub(super) fn suppresses_generic_classes(component: &str) -> bool {
    matches!(
        component,
        "TabContext"
            | "TabList"
            | "TabPanel"
            | "Timeline"
            | "TimelineConnector"
            | "TimelineContent"
            | "TimelineDot"
            | "TimelineItem"
            | "TimelineOppositeContent"
            | "TimelineSeparator"
            | "TreeItem"
    )
}

fn append_tab_panel_classes(node: &mut UiTemplateNode, prefix: &str) {
    if bool_attribute_any(node, &["hidden"]) || has_mismatched_tab_value(node) {
        append_class(&mut node.classes, format!("{prefix}-hidden"));
    }
}

fn append_timeline_item_classes(node: &mut UiTemplateNode, prefix: &str) {
    append_position_class(node, prefix, "right");
    if !has_opposite_content(node) {
        append_class(
            &mut node.classes,
            format!("{prefix}-missingOppositeContent"),
        );
    }
}

fn append_timeline_dot_classes(node: &mut UiTemplateNode, prefix: &str) {
    let variant = string_attribute_any(node, &["variant", "mui_variant"])
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "filled".to_string());
    append_class(&mut node.classes, format!("{prefix}-{variant}"));

    let color = string_attribute_any(node, &["color"])
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "grey".to_string());
    if color != "inherit" {
        append_class(
            &mut node.classes,
            format!("{prefix}-{variant}{}", pascal_case(&color)),
        );
    }
}

fn append_position_class(node: &mut UiTemplateNode, prefix: &str, default_position: &str) {
    let position = string_attribute_any(node, &["position"])
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| default_position.to_string());
    append_class(
        &mut node.classes,
        format!("{prefix}-{}", timeline_position_class(&position)),
    );
}

fn timeline_position_class(position: &str) -> String {
    if position == "alternate-reverse" {
        "positionAlternateReverse".to_string()
    } else {
        format!("position{}", pascal_case(position))
    }
}

fn has_mismatched_tab_value(node: &UiTemplateNode) -> bool {
    let Some(value) = string_attribute_any(node, &["value", "value_text"]) else {
        return false;
    };
    string_attribute_any(
        node,
        &[
            "context_value",
            "contextValue",
            "selected_value",
            "selectedValue",
        ],
    )
    .is_some_and(|selected| selected != value)
}

fn has_opposite_content(node: &UiTemplateNode) -> bool {
    bool_attribute_any(
        node,
        &[
            "hasOppositeContent",
            "has_opposite_content",
            "oppositeContent",
            "opposite_content",
        ],
    ) || node
        .slots
        .get("oppositeContent")
        .is_some_and(|children| !children.is_empty())
        || node.children.iter().any(|child| {
            child.component.as_deref() == Some("TimelineOppositeContent")
                || mui_slot_name(child).as_deref() == Some("oppositeContent")
        })
}
