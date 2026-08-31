use std::collections::BTreeMap;

use toml::Value;
use zircon_runtime_interface::ui::template::UiTemplateNode;

use super::{
    append_class, bool_attribute_any, bool_from_attributes_any, pascal_case, string_attribute_any,
};

mod data_grid;

pub(super) fn append_component_classes(
    node: &mut UiTemplateNode,
    component: &str,
    prefix: &str,
) -> bool {
    match component {
        "MaterialTreeView" => append_tree_view_classes(node),
        "DataGrid" => data_grid::append_component_classes(node, prefix),
        "DateTimePickers" => append_date_time_picker_classes(node),
        "Charts" | "LineChart" | "BarChart" | "PieChart" | "SparkLineChart" | "Gauge" => {
            append_chart_classes(node, component)
        }
        "AgentChat" => append_agent_chat_classes(node),
        "ChatConversationList" => append_chat_conversation_list_classes(node),
        "ChatMessageList" => append_chat_message_list_classes(node),
        "ChatComposer" => append_chat_composer_classes(node),
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
    if owner_component == "DataGrid"
        && data_grid::append_slot_classes(child, owner_attributes, slot_name)
    {
        return true;
    }

    match (owner_component, slot_name) {
        ("MaterialTreeView", "item") => {
            append_class(&mut child.classes, "MuiTreeItem-root".to_string());
            append_tree_item_state_classes(child, owner_attributes);
        }
        ("MaterialTreeView", "content") => {
            append_class(&mut child.classes, "MuiTreeItem-content".to_string());
            append_tree_item_state_classes(child, owner_attributes);
        }
        ("MaterialTreeView", "label") => {
            append_class(&mut child.classes, "MuiTreeItem-label".to_string());
            if bool_from_attributes_any(owner_attributes, &["editable"]) {
                append_class(&mut child.classes, "MuiTreeItem-labelInput".to_string());
            }
        }
        ("MaterialTreeView", "icon") => {
            append_class(&mut child.classes, "MuiTreeItem-iconContainer".to_string());
        }
        ("MaterialTreeView", "checkbox") => {
            append_class(&mut child.classes, "MuiTreeItem-checkbox".to_string());
            if bool_from_attributes_any(
                owner_attributes,
                &["checkboxSelection", "checkbox_selection"],
            ) {
                append_class(
                    &mut child.classes,
                    "MuiTreeItem-checkboxSelection".to_string(),
                );
            }
        }
        ("DateTimePickers", "field" | "layout" | "toolbar" | "popper") => {
            append_class(
                &mut child.classes,
                format!("MuiPickers{}", pascal_case(slot_name)),
            );
            append_picker_slot_state_classes(child, owner_attributes, slot_name);
        }
        ("AgentChat", "messages") => {
            append_class(&mut child.classes, "MuiAgentChat-messages".to_string());
            append_agent_chat_slot_state_classes(child, owner_attributes, slot_name);
        }
        ("AgentChat", "composer") => {
            append_class(&mut child.classes, "MuiAgentChat-composer".to_string());
            append_agent_chat_slot_state_classes(child, owner_attributes, slot_name);
        }
        ("ChatMessageList", "messages") => {
            append_class(&mut child.classes, "MuiChatMessage-root".to_string());
            append_chat_message_slot_state_classes(child, owner_attributes);
        }
        (
            "Charts" | "LineChart" | "BarChart" | "PieChart" | "SparkLineChart" | "Gauge",
            "legend" | "tooltip",
        ) => {
            append_class(
                &mut child.classes,
                format!("MuiCharts{}", pascal_case(slot_name)),
            );
            append_chart_slot_state_classes(child, owner_attributes, owner_component, slot_name);
        }
        _ => return false,
    }
    true
}

pub(super) fn suppresses_generic_classes(component: &str) -> bool {
    matches!(
        component,
        "MaterialTreeView"
            | "DataGrid"
            | "DateTimePickers"
            | "Charts"
            | "LineChart"
            | "BarChart"
            | "PieChart"
            | "SparkLineChart"
            | "Gauge"
            | "AgentChat"
            | "ChatConversationList"
            | "ChatMessageList"
            | "ChatComposer"
    )
}

fn append_tree_view_classes(node: &mut UiTemplateNode) {
    append_class(&mut node.classes, "MuiTreeView-root".to_string());
    if bool_attribute_any(node, &["multiSelect", "multi_select"]) {
        append_class(&mut node.classes, "MuiTreeView-multiSelect".to_string());
    }
    if bool_attribute_any(node, &["checkboxSelection", "checkbox_selection"]) {
        append_class(
            &mut node.classes,
            "MuiTreeView-checkboxSelection".to_string(),
        );
    }
    if bool_attribute_any(
        node,
        &["disabledItemsFocusable", "disabled_items_focusable"],
    ) {
        append_class(
            &mut node.classes,
            "MuiTreeView-disabledItemsFocusable".to_string(),
        );
    }
    if bool_attribute_any(node, &["editable"]) {
        append_class(&mut node.classes, "MuiTreeView-editable".to_string());
    }
    if array_attribute_any_non_empty(
        node,
        &[
            "defaultExpandedItems",
            "default_expanded_items",
            "expandedItems",
            "expanded_items",
        ],
    ) {
        append_class(
            &mut node.classes,
            "MuiTreeView-hasExpandedItems".to_string(),
        );
    }
    if array_attribute_any_non_empty(node, &["selectedItems", "selected_items"]) {
        append_class(
            &mut node.classes,
            "MuiTreeView-hasSelectedItems".to_string(),
        );
    }
    if number_attribute_any(
        node,
        &["itemChildrenIndentation", "item_children_indentation"],
    ) {
        append_class(
            &mut node.classes,
            "MuiTreeView-hasItemIndentation".to_string(),
        );
    }
}

fn append_tree_item_state_classes(
    child: &mut UiTemplateNode,
    owner_attributes: &BTreeMap<String, Value>,
) {
    if array_attribute_any_non_empty_from_attributes(
        owner_attributes,
        &[
            "defaultExpandedItems",
            "default_expanded_items",
            "expandedItems",
            "expanded_items",
        ],
    ) {
        append_class(&mut child.classes, "MuiTreeItem-expanded".to_string());
    }
    if array_attribute_any_non_empty_from_attributes(
        owner_attributes,
        &["selectedItems", "selected_items"],
    ) {
        append_class(&mut child.classes, "MuiTreeItem-selected".to_string());
    }
    if bool_from_attributes_any(owner_attributes, &["editable"]) {
        append_class(&mut child.classes, "MuiTreeItem-editable".to_string());
    }
    if bool_from_attributes_any(
        owner_attributes,
        &["disabledItemsFocusable", "disabled_items_focusable"],
    ) {
        append_class(
            &mut child.classes,
            "MuiTreeItem-disabledItemsFocusable".to_string(),
        );
    }
}

fn append_picker_slot_state_classes(
    child: &mut UiTemplateNode,
    owner_attributes: &BTreeMap<String, Value>,
    slot_name: &str,
) {
    match slot_name {
        "field" => {
            if bool_from_attributes_any(owner_attributes, &["readOnly", "read_only"]) {
                append_class(&mut child.classes, "MuiPickersField-readOnly".to_string());
            }
            if picker_has_value(owner_attributes) {
                append_class(&mut child.classes, "MuiPickersField-hasValue".to_string());
            }
            if string_attribute_any_from_attributes(owner_attributes, &["format"]).is_some() {
                append_class(&mut child.classes, "MuiPickersField-hasFormat".to_string());
            }
        }
        "layout" => {
            if let Some(variant) =
                string_attribute_any_from_attributes(owner_attributes, &["variant"])
            {
                append_class(
                    &mut child.classes,
                    format!("MuiPickersLayout-{}", pascal_case(variant)),
                );
            }
            if array_attribute_any_non_empty_from_attributes(owner_attributes, &["views"]) {
                append_class(&mut child.classes, "MuiPickersLayout-hasViews".to_string());
            }
        }
        "toolbar" => {
            if bool_from_attributes_any(owner_attributes, &["ampm"]) {
                append_class(&mut child.classes, "MuiPickersToolbar-ampm".to_string());
            }
            if array_attribute_any_non_empty_from_attributes(owner_attributes, &["views"]) {
                append_class(&mut child.classes, "MuiPickersToolbar-hasViews".to_string());
            }
        }
        "popper" => {
            if bool_from_attributes_any(owner_attributes, &["open", "popup_open", "popupOpen"]) {
                append_class(&mut child.classes, "MuiPickersPopper-open".to_string());
            }
            if picker_has_date_bounds(owner_attributes) {
                append_class(
                    &mut child.classes,
                    "MuiPickersPopper-hasDateBounds".to_string(),
                );
            }
        }
        _ => {}
    }
}

fn append_chart_slot_state_classes(
    child: &mut UiTemplateNode,
    owner_attributes: &BTreeMap<String, Value>,
    owner_component: &str,
    slot_name: &str,
) {
    match slot_name {
        "legend" => {
            if array_attribute_any_non_empty_from_attributes(owner_attributes, &["series"]) {
                append_class(&mut child.classes, "MuiChartsLegend-hasSeries".to_string());
            }
            if array_attribute_any_non_empty_from_attributes(owner_attributes, &["colors"]) {
                append_class(
                    &mut child.classes,
                    "MuiChartsLegend-hasCustomColors".to_string(),
                );
            }
            if bool_from_attributes_any(owner_attributes, &["loading"]) {
                append_class(&mut child.classes, "MuiChartsLegend-loading".to_string());
            }
        }
        "tooltip" => {
            if bool_from_attributes_any(owner_attributes, &["loading"]) {
                append_class(&mut child.classes, "MuiChartsTooltip-loading".to_string());
            }
            if array_attribute_any_non_empty_from_attributes(owner_attributes, &["series"]) {
                append_class(&mut child.classes, "MuiChartsTooltip-hasSeries".to_string());
            }
            if chart_has_axes(owner_attributes) {
                append_class(&mut child.classes, "MuiChartsTooltip-hasAxes".to_string());
            }
            if map_attribute_any_non_empty_from_attributes(owner_attributes, &["margin"]) {
                append_class(&mut child.classes, "MuiChartsTooltip-hasMargin".to_string());
            }
            if let Some(interaction) =
                string_attribute_any_from_attributes(owner_attributes, &["interaction"])
            {
                append_class(
                    &mut child.classes,
                    format!("MuiChartsTooltip-interaction{}", pascal_case(interaction)),
                );
            }
            if owner_component == "Gauge"
                && number_attribute_any_from_attributes(owner_attributes, &["value"])
            {
                append_class(&mut child.classes, "MuiChartsTooltip-hasValue".to_string());
            }
        }
        _ => {}
    }
}

fn picker_has_value(owner_attributes: &BTreeMap<String, Value>) -> bool {
    string_attribute_any_from_attributes(
        owner_attributes,
        &[
            "value",
            "date_value",
            "dateValue",
            "time_value",
            "timeValue",
        ],
    )
    .is_some()
}

fn picker_has_date_bounds(owner_attributes: &BTreeMap<String, Value>) -> bool {
    string_attribute_any_from_attributes(
        owner_attributes,
        &["minDate", "min_date", "maxDate", "max_date"],
    )
    .is_some()
}

fn chart_has_axes(owner_attributes: &BTreeMap<String, Value>) -> bool {
    array_attribute_any_non_empty_from_attributes(
        owner_attributes,
        &["x_axis", "xAxis", "y_axis", "yAxis"],
    )
}

fn append_agent_chat_slot_state_classes(
    child: &mut UiTemplateNode,
    owner_attributes: &BTreeMap<String, Value>,
    slot_name: &str,
) {
    match slot_name {
        "messages" => {
            if array_attribute_any_non_empty_from_attributes(owner_attributes, &["messages"]) {
                append_class(
                    &mut child.classes,
                    "MuiAgentChat-messagesPopulated".to_string(),
                );
            }
            if bool_from_attributes_any(owner_attributes, &["streaming"]) {
                append_class(
                    &mut child.classes,
                    "MuiAgentChat-messagesStreaming".to_string(),
                );
            }
            if bool_from_attributes_any(owner_attributes, &["error"]) {
                append_class(&mut child.classes, "MuiAgentChat-messagesError".to_string());
            }
        }
        "composer" => {
            if bool_from_attributes_any(owner_attributes, &["streaming"]) {
                append_class(
                    &mut child.classes,
                    "MuiAgentChat-composerStreaming".to_string(),
                );
            }
            if bool_from_attributes_any(owner_attributes, &["error"]) {
                append_class(&mut child.classes, "MuiAgentChat-composerError".to_string());
            }
            if string_attribute_any_from_attributes(
                owner_attributes,
                &["composer_text", "composerText"],
            )
            .is_some()
            {
                append_class(
                    &mut child.classes,
                    "MuiAgentChat-composerHasText".to_string(),
                );
            }
        }
        _ => {}
    }
}

fn append_chat_message_slot_state_classes(
    child: &mut UiTemplateNode,
    owner_attributes: &BTreeMap<String, Value>,
) {
    if array_attribute_any_non_empty_from_attributes(owner_attributes, &["messages"]) {
        append_class(&mut child.classes, "MuiChatMessage-populated".to_string());
    }
}

fn append_date_time_picker_classes(node: &mut UiTemplateNode) {
    match string_attribute_any(node, &["picker_mode", "pickerMode"]).as_deref() {
        Some("date") | Some("date_range") => {
            append_class(&mut node.classes, "MuiDatePicker-root".to_string());
        }
        Some("time") => append_class(&mut node.classes, "MuiTimePicker-root".to_string()),
        _ => append_class(&mut node.classes, "MuiDateTimePicker-root".to_string()),
    }
    if let Some(variant) = string_attribute_any(node, &["variant"]) {
        append_class(
            &mut node.classes,
            format!("MuiPickersLayout-{}", pascal_case(&variant)),
        );
    }
    if bool_attribute_any(node, &["readOnly", "read_only"]) {
        append_class(&mut node.classes, "MuiPickers-readOnly".to_string());
    }
    if bool_attribute_any(node, &["ampm"]) {
        append_class(&mut node.classes, "MuiPickers-ampm".to_string());
    }
    if string_attribute_any(node, &["minDate", "min_date"]).is_some()
        || string_attribute_any(node, &["maxDate", "max_date"]).is_some()
    {
        append_class(&mut node.classes, "MuiPickers-hasDateBounds".to_string());
    }
    if array_attribute_any_non_empty(node, &["views"]) {
        append_class(&mut node.classes, "MuiPickers-hasViews".to_string());
    }
    if string_attribute_any(node, &["value"]).is_some() {
        append_class(&mut node.classes, "MuiPickers-hasValue".to_string());
    }
    if string_attribute_any(node, &["view"]).is_some() {
        append_class(&mut node.classes, "MuiPickers-hasView".to_string());
    }
    if string_attribute_any(node, &["format"]).is_some() {
        append_class(&mut node.classes, "MuiPickers-hasFormat".to_string());
    }
}

fn append_chart_classes(node: &mut UiTemplateNode, component: &str) {
    append_class(&mut node.classes, "MuiChartsSurface-root".to_string());
    append_class(&mut node.classes, format!("Mui{component}-root"));
    if bool_attribute_any(node, &["loading"]) {
        append_class(&mut node.classes, "MuiCharts-loading".to_string());
    }
    if array_attribute_any_non_empty(node, &["series"]) {
        append_class(&mut node.classes, "MuiCharts-hasSeries".to_string());
    }
    if array_attribute_any_non_empty(node, &["x_axis", "xAxis", "y_axis", "yAxis"]) {
        append_class(&mut node.classes, "MuiCharts-hasAxes".to_string());
    }
    if array_attribute_any_non_empty(node, &["colors"]) {
        append_class(&mut node.classes, "MuiCharts-hasCustomColors".to_string());
    }
    if map_attribute_any_non_empty(node, &["margin"]) {
        append_class(&mut node.classes, "MuiCharts-hasMargin".to_string());
    }
    if component == "Gauge" && number_attribute_any(node, &["value"]) {
        append_class(&mut node.classes, "MuiGauge-hasValue".to_string());
    }
}

fn append_agent_chat_classes(node: &mut UiTemplateNode) {
    append_class(&mut node.classes, "MuiAgentChat-root".to_string());
    if bool_attribute_any(node, &["streaming"]) {
        append_class(&mut node.classes, "MuiAgentChat-streaming".to_string());
    }
    if bool_attribute_any(node, &["error"]) {
        append_class(&mut node.classes, "MuiAgentChat-error".to_string());
    }
    if array_attribute_any_non_empty(node, &["messages"]) {
        append_class(&mut node.classes, "MuiAgentChat-hasMessages".to_string());
    }
    if string_attribute_any(node, &["composer_text", "composerText"]).is_some() {
        append_class(
            &mut node.classes,
            "MuiAgentChat-hasComposerText".to_string(),
        );
    }
}

fn append_chat_conversation_list_classes(node: &mut UiTemplateNode) {
    append_class(
        &mut node.classes,
        "MuiChatConversationList-root".to_string(),
    );
    if array_attribute_any_non_empty(node, &["conversations"]) {
        append_class(
            &mut node.classes,
            "MuiChatConversationList-populated".to_string(),
        );
    }
}

fn append_chat_message_list_classes(node: &mut UiTemplateNode) {
    append_class(&mut node.classes, "MuiChatMessageList-root".to_string());
    if array_attribute_any_non_empty(node, &["messages"]) {
        append_class(
            &mut node.classes,
            "MuiChatMessageList-populated".to_string(),
        );
    }
}

fn append_chat_composer_classes(node: &mut UiTemplateNode) {
    append_class(&mut node.classes, "MuiChatComposer-root".to_string());
    if bool_attribute_any(node, &["streaming"]) {
        append_class(&mut node.classes, "MuiChatComposer-streaming".to_string());
    }
    if string_attribute_any(node, &["composer_text", "composerText"]).is_some() {
        append_class(&mut node.classes, "MuiChatComposer-hasText".to_string());
    }
}

fn array_attribute_any_non_empty(node: &UiTemplateNode, names: &[&str]) -> bool {
    names.iter().any(|name| {
        node.attributes
            .get(*name)
            .and_then(Value::as_array)
            .is_some_and(|values| !values.is_empty())
    })
}

fn array_attribute_any_empty(node: &UiTemplateNode, names: &[&str]) -> bool {
    names.iter().any(|name| {
        node.attributes
            .get(*name)
            .and_then(Value::as_array)
            .is_some_and(Vec::is_empty)
    })
}

fn map_attribute_any_non_empty(node: &UiTemplateNode, names: &[&str]) -> bool {
    names.iter().any(|name| {
        node.attributes
            .get(*name)
            .and_then(Value::as_table)
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

fn map_attribute_any_non_empty_from_attributes(
    attributes: &BTreeMap<String, Value>,
    names: &[&str],
) -> bool {
    names.iter().any(|name| {
        attributes
            .get(*name)
            .and_then(Value::as_table)
            .is_some_and(|values| !values.is_empty())
    })
}

fn string_attribute_any_from_attributes<'a>(
    attributes: &'a BTreeMap<String, Value>,
    names: &[&str],
) -> Option<&'a str> {
    names.iter().find_map(|name| {
        attributes
            .get(*name)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
    })
}

fn number_attribute_any(node: &UiTemplateNode, names: &[&str]) -> bool {
    names.iter().any(|name| {
        node.attributes
            .get(*name)
            .is_some_and(|value| value.as_float().is_some() || value.as_integer().is_some())
    })
}

fn number_attribute_any_from_attributes(
    attributes: &BTreeMap<String, Value>,
    names: &[&str],
) -> bool {
    names.iter().any(|name| {
        attributes
            .get(*name)
            .is_some_and(|value| value.as_float().is_some() || value.as_integer().is_some())
    })
}

#[cfg(test)]
#[path = "mui_x_classes/borrowed_owner_attribute_tests.rs"]
mod borrowed_owner_attribute_tests;
