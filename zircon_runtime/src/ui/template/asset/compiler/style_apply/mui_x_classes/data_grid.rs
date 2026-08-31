use std::collections::BTreeMap;

use toml::Value;
use zircon_runtime_interface::ui::template::UiTemplateNode;

use super::super::{
    append_class, bool_attribute_any, bool_from_attributes_any, string_attribute_any,
};
use super::{
    array_attribute_any_empty, array_attribute_any_non_empty,
    array_attribute_any_non_empty_from_attributes, map_attribute_any_non_empty,
    map_attribute_any_non_empty_from_attributes, number_attribute_any,
    string_attribute_any_from_attributes,
};

fn prefixed_pascal_class(prefix: &str, infix: &str, value: &str) -> String {
    let mut class = String::with_capacity(prefix.len() + infix.len() + value.len());
    class.push_str(prefix);
    class.push_str(infix);

    let mut uppercase_next = true;
    for character in value.chars() {
        if matches!(character, '-' | '_' | ' ') {
            uppercase_next = true;
            continue;
        }
        class.push(if uppercase_next {
            character.to_ascii_uppercase()
        } else {
            character
        });
        uppercase_next = false;
    }
    class
}

pub(super) fn append_component_classes(node: &mut UiTemplateNode, prefix: &str) {
    append_class(&mut node.classes, format!("{prefix}-withBorderColor"));
    let density = string_attribute_any(node, &["density"])
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "standard".to_string());
    append_class(
        &mut node.classes,
        prefixed_pascal_class(prefix, "-root--density", &density),
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
            prefixed_pascal_class(prefix, "-rowSpacing", &row_spacing_type),
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
            prefixed_pascal_class(prefix, "-sortingMode", &mode),
        );
    }
    if let Some(mode) = string_attribute_any(node, &["filterMode", "filter_mode"]) {
        append_class(
            &mut node.classes,
            prefixed_pascal_class(prefix, "-filterMode", &mode),
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
            prefixed_pascal_class(prefix, "-editMode", &mode),
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

pub(super) fn append_slot_classes(
    child: &mut UiTemplateNode,
    owner_attributes: &BTreeMap<String, Value>,
    slot_name: &str,
) -> bool {
    match slot_name {
        "header" => append_class(&mut child.classes, "MuiDataGrid-columnHeaders".to_string()),
        "columnHeader" => append_column_header_slot_classes(child, owner_attributes),
        "row" => append_row_slot_classes(child, owner_attributes),
        "cell" => append_cell_slot_classes(child, owner_attributes),
        "toolbar" | "footer" | "loadingOverlay" | "noRowsOverlay" => {
            append_class(
                &mut child.classes,
                prefixed_pascal_class("MuiDataGrid", "-", slot_name),
            );
        }
        _ => return false,
    }
    true
}

fn append_column_header_slot_classes(
    child: &mut UiTemplateNode,
    owner_attributes: &BTreeMap<String, Value>,
) {
    append_class(&mut child.classes, "MuiDataGrid-columnHeader".to_string());
    if string_attribute_any_from_attributes(owner_attributes, &["sortingMode", "sorting_mode"])
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
    if array_attribute_any_non_empty_from_attributes(owner_attributes, &["sortModel", "sort_model"])
    {
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

fn append_row_slot_classes(child: &mut UiTemplateNode, owner_attributes: &BTreeMap<String, Value>) {
    append_class(&mut child.classes, "MuiDataGrid-row".to_string());
    if bool_from_attributes_any(owner_attributes, &["loading"]) {
        append_class(&mut child.classes, "MuiDataGrid-row--loading".to_string());
    }
}

fn append_cell_slot_classes(
    child: &mut UiTemplateNode,
    owner_attributes: &BTreeMap<String, Value>,
) {
    append_class(&mut child.classes, "MuiDataGrid-cell".to_string());
    if string_attribute_any_from_attributes(owner_attributes, &["editMode", "edit_mode"]).is_some()
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

#[cfg(test)]
#[path = "data_grid/single_buffer_pascal_class_tests.rs"]
mod single_buffer_pascal_class_tests;
