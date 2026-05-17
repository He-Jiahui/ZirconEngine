use super::shared::*;

pub(super) fn descriptors() -> Vec<UiComponentDescriptor> {
    vec![
        composite(
            "Accordion",
            "Accordion",
            UiComponentCategory::Container,
            "accordion",
        )
        .with_prop(expanded_prop())
        .slot(UiSlotSchema::new("summary").required(true))
        .slot(UiSlotSchema::new("details").multiple(true))
        .event(UiComponentEventKind::ToggleExpanded),
        composite(
            "AppBar",
            "App Bar",
            UiComponentCategory::Container,
            "app-bar",
        )
        .with_prop(enum_prop("position", "top"))
        .slot(UiSlotSchema::new("navigation"))
        .slot(UiSlotSchema::new("title"))
        .slot(UiSlotSchema::new("actions").multiple(true)),
        composite("Card", "Card", UiComponentCategory::Container, "card")
            .with_prop(enum_prop("variant", "elevated"))
            .slot(UiSlotSchema::new("media"))
            .slot(UiSlotSchema::new("content").multiple(true))
            .slot(UiSlotSchema::new("actions").multiple(true)),
        composite("Paper", "Paper", UiComponentCategory::Container, "paper")
            .with_prop(int_prop("elevation", 1))
            .slot(UiSlotSchema::new("content").multiple(true)),
        composite("Panel", "Panel", UiComponentCategory::Container, "panel")
            .slot(UiSlotSchema::new("header"))
            .slot(UiSlotSchema::new("content").multiple(true)),
        composite(
            "PropertyGrid",
            "Property Grid",
            UiComponentCategory::Container,
            "property-grid",
        )
        .with_prop(string_prop("selection_summary"))
        .slot(UiSlotSchema::new("sections").multiple(true))
        .event(UiComponentEventKind::ValueChanged)
        .requires_host_capability(UiHostCapability::Editor),
        composite(
            "InspectorSection",
            "Inspector Section",
            UiComponentCategory::Container,
            "inspector-section",
        )
        .with_prop(text_prop())
        .with_prop(expanded_prop())
        .slot(UiSlotSchema::new("fields").multiple(true))
        .event(UiComponentEventKind::ToggleExpanded)
        .requires_host_capability(UiHostCapability::Editor),
        editor_panel_component(
            "PreviewPane",
            "Preview Pane",
            UiComponentCategory::Container,
            "preview-pane",
        )
        .with_prop(string_prop("asset_id"))
        .slot(UiSlotSchema::new("content").multiple(true))
        .requires_host_capability(UiHostCapability::ImageRender)
        .requires_render_capability(UiRenderCapability::Image),
        editor_panel_component(
            "MetadataPane",
            "Metadata Pane",
            UiComponentCategory::Container,
            "metadata-pane",
        )
        .with_prop(string_prop("asset_id"))
        .slot(UiSlotSchema::new("fields").multiple(true))
        .event(UiComponentEventKind::ValueChanged),
    ]
}
