use super::shared::*;

pub(super) fn descriptors() -> Vec<UiComponentDescriptor> {
    vec![
        data_view("MaterialTreeView", "MUI X Tree View", "mui-x-tree-view")
            .with_prop(string_prop("query"))
            .with_prop(expanded_prop())
            .with_prop(bool_prop("editable", true))
            .with_prop(bool_prop("checkboxSelection", false))
            .with_prop(bool_prop("multiSelect", false))
            .with_prop(bool_prop("disabledItemsFocusable", false))
            .with_prop(array_prop("defaultExpandedItems"))
            .with_prop(array_prop("selectedItems"))
            .with_prop(float_prop("itemChildrenIndentation", 16.0))
            .slot(UiSlotSchema::new("items").multiple(true))
            .slot(UiSlotSchema::new("item").multiple(true))
            .slot(UiSlotSchema::new("content").multiple(true))
            .slot(UiSlotSchema::new("label"))
            .slot(UiSlotSchema::new("icon"))
            .slot(UiSlotSchema::new("checkbox"))
            .events([
                UiComponentEventKind::SelectOption,
                UiComponentEventKind::ToggleExpanded,
                UiComponentEventKind::Commit,
            ]),
        virtualized_range_props(
            data_view("DataGrid", "MUI X Data Grid", "mui-x-data-grid")
                .descriptor_kind(UiComponentDescriptorKind::Layout)
                .layout_role(UiComponentLayoutRole::VirtualList)
                .with_prop(array_prop("columns"))
                .with_prop(array_prop("rows")),
        )
        .with_prop(bool_prop("loading", false))
        .with_prop(bool_prop("checkboxSelection", false))
        .with_prop(bool_prop("disableColumnMenu", false))
        .with_prop(bool_prop("disableRowSelectionOnClick", false))
        .with_prop(bool_prop("autoHeight", false))
        .with_prop(bool_prop("showToolbar", false))
        .with_prop(bool_prop("hideFooter", false))
        .with_prop(bool_prop("hideFooterPagination", false))
        .with_prop(bool_prop("hideFooterSelectedRowCount", false))
        .with_prop(bool_prop("showCellVerticalBorder", false))
        .with_prop(bool_prop("showColumnVerticalBorder", false))
        .with_prop(mui_enum_prop(
            "rowSpacingType",
            "margin",
            ["margin", "border"],
        ))
        .with_prop(int_prop("scrollbarSize", 0))
        .with_prop(default_string_prop("label", ""))
        .with_prop(int_prop("page", 0))
        .with_prop(int_prop("pageSize", 25))
        .with_prop(array_prop("rowSelectionModel"))
        .with_prop(mui_enum_prop("sortingMode", "client", ["client", "server"]))
        .with_prop(mui_enum_prop("filterMode", "client", ["client", "server"]))
        .with_prop(array_prop("sortModel"))
        .with_prop(map_prop("filterModel"))
        .with_prop(map_prop("paginationModel"))
        .with_prop(array_prop("quickFilterValues"))
        .with_prop(mui_enum_prop("editMode", "cell", ["cell", "row"]))
        .with_prop(map_prop("cellModesModel"))
        .with_prop(map_prop("rowModesModel"))
        .with_prop(map_prop("columnVisibilityModel"))
        .with_prop(map_prop("pinnedColumns"))
        .slot(UiSlotSchema::new("header").multiple(true))
        .slot(UiSlotSchema::new("columnHeader").multiple(true))
        .slot(UiSlotSchema::new("row").multiple(true))
        .slot(UiSlotSchema::new("cell").multiple(true))
        .slot(UiSlotSchema::new("toolbar"))
        .slot(UiSlotSchema::new("footer"))
        .slot(UiSlotSchema::new("loadingOverlay"))
        .slot(UiSlotSchema::new("noRowsOverlay"))
        .events([
            UiComponentEventKind::SetVisibleRange,
            UiComponentEventKind::SelectOption,
            UiComponentEventKind::Commit,
        ])
        .requires_host_capability(UiHostCapability::VirtualizedLayout)
        .requires_render_capability(UiRenderCapability::VirtualizedLayout),
        primitive(
            "DateTimePickers",
            "Date and Time Pickers",
            UiComponentCategory::Input,
            "mui-x-date-time-pickers",
        )
        .with_prop(value_text_prop())
        .with_prop(default_string_prop("date_value", "2026-05-17"))
        .with_prop(default_string_prop("time_value", "09:30"))
        .with_prop(mui_enum_prop(
            "picker_mode",
            "date_time",
            ["date", "time", "date_time", "date_range", "date_time_range"],
        ))
        .with_prop(mui_enum_prop(
            "variant",
            "desktop",
            ["desktop", "mobile", "static"],
        ))
        .with_prop(default_string_prop("view", "day"))
        .with_prop(array_prop("views"))
        .with_prop(default_string_prop("format", ""))
        .with_prop(bool_prop("ampm", false))
        .with_prop(bool_prop("readOnly", false))
        .with_prop(default_string_prop("minDate", ""))
        .with_prop(default_string_prop("maxDate", ""))
        .slot(UiSlotSchema::new("field"))
        .slot(UiSlotSchema::new("layout"))
        .slot(UiSlotSchema::new("toolbar"))
        .slot(UiSlotSchema::new("popper"))
        .events([
            UiComponentEventKind::Focus,
            UiComponentEventKind::OpenPopup,
            UiComponentEventKind::ClosePopup,
            UiComponentEventKind::ValueChanged,
            UiComponentEventKind::Commit,
        ])
        .requires_host_capability(UiHostCapability::TextInput),
        chart("Charts", "MUI X Charts", "mui-x-charts"),
        chart("LineChart", "Line Chart", "mui-x-line-chart"),
        chart("BarChart", "Bar Chart", "mui-x-bar-chart"),
        chart("PieChart", "Pie Chart", "mui-x-pie-chart"),
        chart("SparkLineChart", "Spark Line Chart", "mui-x-sparkline"),
        chart("Gauge", "Gauge", "mui-x-gauge"),
        composite(
            "AgentChat",
            "MUI X Agent Chat",
            UiComponentCategory::Container,
            "mui-x-agent-chat",
        )
        .with_prop(array_prop("messages"))
        .with_prop(string_prop("composer_text"))
        .with_prop(bool_prop("streaming", false))
        .with_prop(bool_prop("error", false))
        .slot(UiSlotSchema::new("messages").multiple(true))
        .slot(UiSlotSchema::new("composer"))
        .events([
            UiComponentEventKind::ValueChanged,
            UiComponentEventKind::Commit,
            UiComponentEventKind::ClosePopup,
        ]),
        composite(
            "ChatConversationList",
            "Chat Conversation List",
            UiComponentCategory::Collection,
            "mui-x-chat-conversation-list",
        )
        .with_prop(array_prop("conversations"))
        .event(UiComponentEventKind::SelectOption),
        composite(
            "ChatMessageList",
            "Chat Message List",
            UiComponentCategory::Collection,
            "mui-x-chat-message-list",
        )
        .with_prop(array_prop("messages"))
        .slot(UiSlotSchema::new("messages").multiple(true)),
        primitive(
            "ChatComposer",
            "Chat Composer",
            UiComponentCategory::Input,
            "mui-x-chat-composer",
        )
        .with_prop(string_prop("composer_text"))
        .with_prop(bool_prop("streaming", false))
        .slot(UiSlotSchema::new("root"))
        .events([
            UiComponentEventKind::Focus,
            UiComponentEventKind::ValueChanged,
            UiComponentEventKind::Commit,
        ])
        .requires_host_capability(UiHostCapability::TextInput),
    ]
}

fn chart(id: &str, display_name: &str, role: &str) -> UiComponentDescriptor {
    composite(id, display_name, UiComponentCategory::Visual, role)
        .with_prop(array_prop("series"))
        .with_prop(array_prop("x_axis"))
        .with_prop(array_prop("y_axis"))
        .with_prop(enum_prop("interaction", "hover"))
        .with_prop(float_prop("width", 320.0))
        .with_prop(float_prop("height", 180.0))
        .with_prop(array_prop("colors"))
        .with_prop(map_prop("margin"))
        .with_prop(bool_prop("loading", false))
        .slot(UiSlotSchema::new("legend"))
        .slot(UiSlotSchema::new("tooltip"))
        .event(UiComponentEventKind::Hover)
        .requires_render_capability(UiRenderCapability::Canvas)
}

fn mui_enum_prop<const N: usize>(
    name: &str,
    default: &str,
    options: [&'static str; N],
) -> UiPropSchema {
    enum_prop_with_options(
        name,
        default,
        options.into_iter().map(enum_option_descriptor),
    )
}
