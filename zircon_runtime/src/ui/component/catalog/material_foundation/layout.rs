use super::shared::*;

pub(super) fn descriptors() -> Vec<UiComponentDescriptor> {
    vec![
        primitive(
            "Scrollbar",
            "Scrollbar",
            UiComponentCategory::Numeric,
            "scrollbar",
        )
        .with_prop(number_value_prop())
        .event(UiComponentEventKind::ValueChanged)
        .requires_render_capability(UiRenderCapability::Scroll),
        layout(
            "Splitter",
            "Splitter",
            UiComponentLayoutRole::Size,
            "splitter",
        )
        .with_prop(number_value_prop())
        .event(UiComponentEventKind::ValueChanged),
        composite("Slot", "Slot", UiComponentCategory::Container, "slot")
            .with_prop(
                UiPropSchema::new("name", UiValueKind::String)
                    .default_value(UiValue::String("content".to_string())),
            )
            .slot(UiSlotSchema::new("content").multiple(true)),
        composite(
            "Composite",
            "Composite",
            UiComponentCategory::Container,
            "composite",
        )
        .slot(UiSlotSchema::new("content").multiple(true)),
        layout(
            "FlexGroup",
            "Flex Group",
            UiComponentLayoutRole::Flex,
            "flex-group",
        ),
        layout(
            "HorizontalGroup",
            "Horizontal Group",
            UiComponentLayoutRole::Flex,
            "horizontal-group",
        ),
        layout(
            "VerticalGroup",
            "Vertical Group",
            UiComponentLayoutRole::Flex,
            "vertical-group",
        ),
        layout(
            "GridGroup",
            "Grid Group",
            UiComponentLayoutRole::Grid,
            "grid-group",
        ),
        layout(
            "Overlay",
            "Overlay",
            UiComponentLayoutRole::Overlay,
            "overlay",
        ),
        layout(
            "ScrollBox",
            "Scroll Box",
            UiComponentLayoutRole::Flex,
            "scroll-box",
        )
        .with_prop(enum_prop_with_options(
            "scroll_axis",
            "vertical",
            ["vertical", "horizontal", "both"]
                .into_iter()
                .map(enum_option_descriptor),
        ))
        .with_prop(bool_prop("scroll_x", false))
        .with_prop(bool_prop("scroll_y", true))
        .with_prop(float_prop("scroll_offset_x", 0.0))
        .with_prop(float_prop("scroll_offset_y", 0.0))
        .with_prop(float_prop("viewport_width", 0.0))
        .with_prop(float_prop("viewport_height", 0.0))
        .with_prop(float_prop("content_width", 0.0))
        .with_prop(float_prop("content_height", 0.0))
        .with_prop(bool_prop("show_scrollbars", true))
        .with_prop(bool_prop("clip_content", true))
        .event(UiComponentEventKind::ValueChanged)
        .requires_render_capability(UiRenderCapability::Scroll),
        layout(
            "ScrollView",
            "Scroll View",
            UiComponentLayoutRole::Flex,
            "scroll-view",
        )
        .requires_render_capability(UiRenderCapability::Scroll),
        layout(
            "SplitView",
            "Split View",
            UiComponentLayoutRole::Size,
            "split-view",
        )
        .with_prop(enum_prop_with_options(
            "orientation",
            "horizontal",
            ["horizontal", "vertical"]
                .into_iter()
                .map(enum_option_descriptor),
        ))
        .with_prop(float_prop("split_ratio", 0.5))
        .with_prop(float_prop("splitter_size", 4.0))
        .with_prop(float_prop("min_first", 0.0))
        .with_prop(float_prop("min_second", 0.0))
        .with_prop(bool_prop("resizable", true))
        .with_prop(string_prop("collapsed_pane"))
        .slot(UiSlotSchema::new("first").required(true))
        .slot(UiSlotSchema::new("second").required(true))
        .slot(UiSlotSchema::new("splitter"))
        .events([
            UiComponentEventKind::BeginDrag,
            UiComponentEventKind::DragDelta,
            UiComponentEventKind::EndDrag,
            UiComponentEventKind::ValueChanged,
        ]),
        layout(
            "PanelGroup",
            "Panel Group",
            UiComponentLayoutRole::EditorDock,
            "panel-group",
        )
        .with_prop(required_string_prop("group_id"))
        .with_prop(enum_prop_with_options(
            "orientation",
            "vertical",
            ["vertical", "horizontal"]
                .into_iter()
                .map(enum_option_descriptor),
        ))
        .with_prop(string_prop("active_panel"))
        .with_prop(array_prop("panel_order"))
        .with_prop(bool_prop("resizable", true))
        .with_prop(bool_prop("collapsible", true))
        .slot(UiSlotSchema::new("header"))
        .slot(UiSlotSchema::new("panels").multiple(true))
        .slot(UiSlotSchema::new("toolbar"))
        .events([
            UiComponentEventKind::Focus,
            UiComponentEventKind::SelectOption,
            UiComponentEventKind::BeginDrag,
            UiComponentEventKind::EndDrag,
        ])
        .requires_host_capability(UiHostCapability::Editor),
    ]
}
