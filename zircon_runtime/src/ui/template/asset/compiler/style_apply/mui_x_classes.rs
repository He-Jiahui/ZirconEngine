use std::collections::BTreeMap;

use toml::Value;
use zircon_runtime_interface::ui::template::UiTemplateNode;

use super::{
    append_class, bool_attribute_any, bool_from_attributes_any, pascal_case, string_attribute_any,
};

pub(super) fn append_component_classes(
    node: &mut UiTemplateNode,
    component: &str,
    prefix: &str,
) -> bool {
    match component {
        "MaterialTreeView" => append_tree_view_classes(node),
        "DataGrid" => append_data_grid_classes(node, prefix),
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
    match (owner_component, slot_name) {
        ("DataGrid", "header") => {
            append_class(&mut child.classes, "MuiDataGrid-columnHeaders".to_string())
        }
        ("DataGrid", "columnHeader") => {
            append_class(&mut child.classes, "MuiDataGrid-columnHeader".to_string());
            if string_attribute_any_from_attributes(
                owner_attributes,
                &["sortingMode", "sorting_mode"],
            )
            .is_some()
                || array_attribute_any_non_empty_from_attributes(
                    owner_attributes,
                    &["sortModel", "sort_model"],
                )
            {
                append_class(
                    &mut child.classes,
                    "MuiDataGrid-columnHeader--sortable".to_string(),
                );
            }
            if array_attribute_any_non_empty_from_attributes(
                owner_attributes,
                &["sortModel", "sort_model"],
            ) {
                append_class(
                    &mut child.classes,
                    "MuiDataGrid-columnHeader--sorted".to_string(),
                );
            }
            if bool_from_attributes_any(
                owner_attributes,
                &["showColumnVerticalBorder", "show_column_vertical_border"],
            ) {
                append_class(
                    &mut child.classes,
                    "MuiDataGrid-columnHeader--withRightBorder".to_string(),
                );
            }
        }
        ("DataGrid", "row") => {
            append_class(&mut child.classes, "MuiDataGrid-row".to_string());
            if bool_from_attributes_any(owner_attributes, &["loading"]) {
                append_class(&mut child.classes, "MuiDataGrid-row--loading".to_string());
            }
        }
        ("DataGrid", "cell") => {
            append_class(&mut child.classes, "MuiDataGrid-cell".to_string());
            if string_attribute_any_from_attributes(owner_attributes, &["editMode", "edit_mode"])
                .is_some()
            {
                append_class(&mut child.classes, "MuiDataGrid-cell--editable".to_string());
            }
            if map_attribute_any_non_empty_from_attributes(
                owner_attributes,
                &[
                    "cellModesModel",
                    "cell_modes_model",
                    "rowModesModel",
                    "row_modes_model",
                ],
            ) {
                append_class(&mut child.classes, "MuiDataGrid-cell--editing".to_string());
            }
            if bool_from_attributes_any(
                owner_attributes,
                &["showCellVerticalBorder", "show_cell_vertical_border"],
            ) {
                append_class(
                    &mut child.classes,
                    "MuiDataGrid-cell--withRightBorder".to_string(),
                );
            }
        }
        ("DataGrid", "toolbar" | "footer" | "loadingOverlay" | "noRowsOverlay") => {
            append_class(
                &mut child.classes,
                format!("MuiDataGrid-{}", pascal_case(slot_name)),
            );
        }
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

fn append_data_grid_classes(node: &mut UiTemplateNode, prefix: &str) {
    append_class(&mut node.classes, format!("{prefix}-withBorderColor"));
    let density = string_attribute_any(node, &["density"])
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "standard".to_string());
    append_class(
        &mut node.classes,
        format!("{prefix}-root--density{}", pascal_case(&density)),
    );
    if bool_attribute_any(node, &["loading"]) {
        append_class(&mut node.classes, format!("{prefix}-loading"));
    }
    if array_attribute_any_non_empty(node, &["rows"]) {
        append_class(&mut node.classes, format!("{prefix}-hasRows"));
    } else if array_attribute_any_empty(node, &["rows"]) {
        append_class(&mut node.classes, format!("{prefix}-emptyRows"));
    }
    if array_attribute_any_non_empty(node, &["columns"]) {
        append_class(&mut node.classes, format!("{prefix}-hasColumns"));
    } else if array_attribute_any_empty(node, &["columns"]) {
        append_class(&mut node.classes, format!("{prefix}-emptyColumns"));
    }
    if bool_attribute_any(node, &["autoHeight", "auto_height"]) {
        append_class(&mut node.classes, format!("{prefix}-autoHeight"));
    }
    if bool_attribute_any(node, &["showToolbar", "show_toolbar"]) {
        append_class(&mut node.classes, format!("{prefix}-showToolbar"));
    }
    if bool_attribute_any(node, &["hideFooter", "hide_footer"]) {
        append_class(&mut node.classes, format!("{prefix}-hideFooter"));
    }
    if bool_attribute_any(node, &["hideFooterPagination", "hide_footer_pagination"]) {
        append_class(&mut node.classes, format!("{prefix}-hideFooterPagination"));
    }
    if bool_attribute_any(
        node,
        &[
            "hideFooterSelectedRowCount",
            "hide_footer_selected_row_count",
        ],
    ) {
        append_class(
            &mut node.classes,
            format!("{prefix}-hideFooterSelectedRowCount"),
        );
    }
    if bool_attribute_any(
        node,
        &["showCellVerticalBorder", "show_cell_vertical_border"],
    ) {
        append_class(
            &mut node.classes,
            format!("{prefix}-showCellVerticalBorder"),
        );
    }
    if bool_attribute_any(
        node,
        &["showColumnVerticalBorder", "show_column_vertical_border"],
    ) {
        append_class(&mut node.classes, format!("{prefix}-withVerticalBorder"));
    }
    if let Some(row_spacing_type) =
        string_attribute_any(node, &["rowSpacingType", "row_spacing_type"])
    {
        append_class(
            &mut node.classes,
            format!("{prefix}-rowSpacing{}", pascal_case(&row_spacing_type)),
        );
    }
    if number_attribute_any(node, &["scrollbarSize", "scrollbar_size"]) {
        append_class(&mut node.classes, format!("{prefix}-hasScrollbarSize"));
    }
    if string_attribute_any(node, &["label"])
        .filter(|label| !label.is_empty())
        .is_some()
    {
        append_class(&mut node.classes, format!("{prefix}-hasLabel"));
    }
    if bool_attribute_any(node, &["checkboxSelection", "checkbox_selection"]) {
        append_class(&mut node.classes, format!("{prefix}-checkboxSelection"));
    }
    if bool_attribute_any(node, &["disableColumnMenu", "disable_column_menu"]) {
        append_class(&mut node.classes, format!("{prefix}-disableColumnMenu"));
    }
    if bool_attribute_any(
        node,
        &[
            "disableRowSelectionOnClick",
            "disable_row_selection_on_click",
        ],
    ) {
        append_class(
            &mut node.classes,
            format!("{prefix}-disableRowSelectionOnClick"),
        );
    }
    if let Some(mode) = string_attribute_any(node, &["sortingMode", "sorting_mode"]) {
        append_class(
            &mut node.classes,
            format!("{prefix}-sortingMode{}", pascal_case(&mode)),
        );
    }
    if let Some(mode) = string_attribute_any(node, &["filterMode", "filter_mode"]) {
        append_class(
            &mut node.classes,
            format!("{prefix}-filterMode{}", pascal_case(&mode)),
        );
    }
    if array_attribute_any_non_empty(node, &["sortModel", "sort_model"]) {
        append_class(&mut node.classes, format!("{prefix}-hasSortModel"));
    }
    if map_attribute_any_non_empty(node, &["filterModel", "filter_model"]) {
        append_class(&mut node.classes, format!("{prefix}-hasFilterModel"));
    }
    if map_attribute_any_non_empty(node, &["paginationModel", "pagination_model"]) {
        append_class(&mut node.classes, format!("{prefix}-hasPaginationModel"));
    }
    if array_attribute_any_non_empty(node, &["quickFilterValues", "quick_filter_values"]) {
        append_class(&mut node.classes, format!("{prefix}-hasQuickFilter"));
    }
    if array_attribute_any_non_empty(node, &["rowSelectionModel", "row_selection_model"]) {
        append_class(&mut node.classes, format!("{prefix}-rowSelection"));
    }
    if let Some(mode) = string_attribute_any(node, &["editMode", "edit_mode"]) {
        append_class(
            &mut node.classes,
            format!("{prefix}-editMode{}", pascal_case(&mode)),
        );
    }
    if map_attribute_any_non_empty(node, &["cellModesModel", "cell_modes_model"]) {
        append_class(&mut node.classes, format!("{prefix}-hasCellModes"));
    }
    if map_attribute_any_non_empty(node, &["rowModesModel", "row_modes_model"]) {
        append_class(&mut node.classes, format!("{prefix}-hasRowModes"));
    }
    if map_attribute_any_non_empty(node, &["columnVisibilityModel", "column_visibility_model"]) {
        append_class(
            &mut node.classes,
            format!("{prefix}-hasColumnVisibilityModel"),
        );
    }
    if map_attribute_any_non_empty(node, &["pinnedColumns", "pinned_columns"]) {
        append_class(&mut node.classes, format!("{prefix}-hasPinnedColumns"));
    }
    if number_attribute_any(node, &["page"]) {
        append_class(&mut node.classes, format!("{prefix}-hasPage"));
    }
    if number_attribute_any(node, &["pageSize", "page_size"]) {
        append_class(&mut node.classes, format!("{prefix}-hasPageSize"));
    }
    if number_attribute_any(node, &["rowCount", "row_count", "total_count"]) {
        append_class(&mut node.classes, format!("{prefix}-hasRowCount"));
    }
    if number_attribute_any(
        node,
        &["rowHeight", "row_height", "itemSize", "item_extent"],
    ) {
        append_class(&mut node.classes, format!("{prefix}-hasRowHeight"));
    }
    if number_attribute_any(node, &["overscanCount", "overscan_count", "overscan"]) {
        append_class(&mut node.classes, format!("{prefix}-hasOverscan"));
    }
    if bool_attribute_any(node, &["disableVirtualization", "disable_virtualization"]) {
        append_class(&mut node.classes, format!("{prefix}-disableVirtualization"));
    }
    if number_attribute_any(node, &["viewport_start", "viewport_count", "visible_end"]) {
        append_class(&mut node.classes, format!("{prefix}-hasViewportRange"));
    }
    if number_attribute_any(node, &["requested_start", "requested_count"]) {
        append_class(&mut node.classes, format!("{prefix}-hasRequestedRange"));
    }
    if number_attribute_any(node, &["scrollTop", "scroll_offset"]) {
        append_class(&mut node.classes, format!("{prefix}-hasScrollOffset"));
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
                    format!("MuiPickersLayout-{}", pascal_case(&variant)),
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
                    format!("MuiChartsTooltip-interaction{}", pascal_case(&interaction)),
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

fn string_attribute_any_from_attributes(
    attributes: &BTreeMap<String, Value>,
    names: &[&str],
) -> Option<String> {
    names.iter().find_map(|name| {
        attributes
            .get(*name)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
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
