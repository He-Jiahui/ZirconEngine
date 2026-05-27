use crate::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime_interface::ui::component::{UiComponentEventKind, UiValue};

use super::super::{assert_has_event, assert_has_prop};
use super::assert_enum_options;

pub(super) fn assert_descriptors(registry: &UiComponentDescriptorRegistry) {
    assert_tree_view(registry);
    assert_data_grid(registry);
    assert_date_time_pickers(registry);
    assert_charts(registry);
    assert_chat(registry);
}

fn assert_tree_view(registry: &UiComponentDescriptorRegistry) {
    let tree = descriptor(registry, "MaterialTreeView");
    for prop in [
        "query",
        "expanded",
        "editable",
        "checkboxSelection",
        "multiSelect",
        "disabledItemsFocusable",
        "defaultExpandedItems",
        "selectedItems",
        "itemChildrenIndentation",
    ] {
        assert_has_prop(tree, prop);
    }
    for slot in ["items", "item", "content", "label", "icon", "checkbox"] {
        assert_has_slot(tree, slot);
    }
}

fn assert_data_grid(registry: &UiComponentDescriptorRegistry) {
    let grid = descriptor(registry, "DataGrid");
    assert_enum_options(grid, "sortingMode", &["client", "server"]);
    assert_enum_options(grid, "filterMode", &["client", "server"]);
    assert_enum_options(grid, "editMode", &["cell", "row"]);
    assert_enum_options(grid, "rowSpacingType", &["margin", "border"]);
    for prop in [
        "columns",
        "rows",
        "loading",
        "checkboxSelection",
        "disableColumnMenu",
        "disableRowSelectionOnClick",
        "autoHeight",
        "showToolbar",
        "hideFooter",
        "hideFooterPagination",
        "hideFooterSelectedRowCount",
        "showCellVerticalBorder",
        "showColumnVerticalBorder",
        "rowSpacingType",
        "scrollbarSize",
        "label",
        "page",
        "pageSize",
        "rowSelectionModel",
        "sortModel",
        "filterModel",
        "paginationModel",
        "quickFilterValues",
        "cellModesModel",
        "rowModesModel",
        "columnVisibilityModel",
        "pinnedColumns",
    ] {
        assert_has_prop(grid, prop);
    }
    for slot in [
        "header",
        "columnHeader",
        "row",
        "cell",
        "toolbar",
        "footer",
        "loadingOverlay",
        "noRowsOverlay",
    ] {
        assert_has_slot(grid, slot);
    }
    assert_has_event(grid, UiComponentEventKind::SetVisibleRange);
}

fn assert_date_time_pickers(registry: &UiComponentDescriptorRegistry) {
    let pickers = descriptor(registry, "DateTimePickers");
    assert_enum_options(
        pickers,
        "picker_mode",
        &["date", "time", "date_time", "date_range", "date_time_range"],
    );
    assert_enum_options(pickers, "variant", &["desktop", "mobile", "static"]);
    for prop in [
        "date_value",
        "time_value",
        "view",
        "views",
        "format",
        "ampm",
        "readOnly",
        "minDate",
        "maxDate",
    ] {
        assert_has_prop(pickers, prop);
    }
    for slot in ["field", "layout", "toolbar", "popper"] {
        assert_has_slot(pickers, slot);
    }
    assert_default_value(
        pickers,
        "date_value",
        UiValue::String("2026-05-17".to_string()),
    );
}

fn assert_charts(registry: &UiComponentDescriptorRegistry) {
    for id in [
        "Charts",
        "LineChart",
        "BarChart",
        "PieChart",
        "SparkLineChart",
        "Gauge",
    ] {
        let chart = descriptor(registry, id);
        for prop in [
            "series",
            "x_axis",
            "y_axis",
            "interaction",
            "width",
            "height",
            "colors",
            "margin",
            "loading",
        ] {
            assert_has_prop(chart, prop);
        }
        for slot in ["legend", "tooltip"] {
            assert_has_slot(chart, slot);
        }
    }
}

fn assert_chat(registry: &UiComponentDescriptorRegistry) {
    let agent = descriptor(registry, "AgentChat");
    for prop in ["messages", "composer_text", "streaming", "error"] {
        assert_has_prop(agent, prop);
    }
    for slot in ["messages", "composer"] {
        assert_has_slot(agent, slot);
    }
    assert_has_event(agent, UiComponentEventKind::Commit);

    assert_has_prop(
        descriptor(registry, "ChatConversationList"),
        "conversations",
    );
    assert_has_prop(descriptor(registry, "ChatMessageList"), "messages");
    assert_has_prop(descriptor(registry, "ChatComposer"), "composer_text");
}

fn descriptor<'a>(
    registry: &'a UiComponentDescriptorRegistry,
    id: &str,
) -> &'a zircon_runtime_interface::ui::component::UiComponentDescriptor {
    registry
        .descriptor(id)
        .unwrap_or_else(|| panic!("{id} descriptor"))
}

fn assert_has_slot(
    descriptor: &zircon_runtime_interface::ui::component::UiComponentDescriptor,
    slot_name: &str,
) {
    assert!(
        descriptor
            .slot_schema
            .iter()
            .any(|slot| slot.name == slot_name),
        "{} missing MUI X slot `{slot_name}`",
        descriptor.id
    );
}

fn assert_default_value(
    descriptor: &zircon_runtime_interface::ui::component::UiComponentDescriptor,
    prop_name: &str,
    expected: UiValue,
) {
    assert_eq!(
        descriptor
            .prop(prop_name)
            .unwrap_or_else(|| panic!("{} missing prop `{prop_name}`", descriptor.id))
            .default_value
            .clone(),
        Some(expected),
        "{} should expose local MUI X default for `{prop_name}`",
        descriptor.id
    );
}
