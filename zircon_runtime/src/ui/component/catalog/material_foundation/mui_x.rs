use super::shared::*;

pub(super) fn descriptors() -> Vec<UiComponentDescriptor> {
    vec![
        data_view("MaterialTreeView", "MUI X Tree View", "mui-x-tree-view")
            .with_prop(string_prop("query"))
            .with_prop(expanded_prop())
            .with_prop(bool_prop("editable", true))
            .slot(UiSlotSchema::new("items").multiple(true))
            .events([
                UiComponentEventKind::SelectOption,
                UiComponentEventKind::ToggleExpanded,
                UiComponentEventKind::Commit,
            ]),
        data_view("DataGrid", "MUI X Data Grid", "mui-x-data-grid")
            .descriptor_kind(UiComponentDescriptorKind::Layout)
            .layout_role(UiComponentLayoutRole::VirtualList)
            .with_prop(array_prop("columns"))
            .with_prop(array_prop("rows"))
            .with_prop(int_prop("row_count", 0))
            .with_prop(int_prop("viewport_start", 0))
            .with_prop(int_prop("viewport_count", 20))
            .slot(UiSlotSchema::new("header").multiple(true))
            .slot(UiSlotSchema::new("row").multiple(true))
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
        .with_prop(enum_prop("picker_mode", "date_time"))
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
        .event(UiComponentEventKind::Hover)
        .requires_render_capability(UiRenderCapability::Canvas)
}
