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
            "ScrollView",
            "Scroll View",
            UiComponentLayoutRole::Flex,
            "scroll-view",
        )
        .requires_render_capability(UiRenderCapability::Scroll),
    ]
}
