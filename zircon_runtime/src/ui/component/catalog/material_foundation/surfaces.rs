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
        override_float_prop_defaults(
            composite(
                "AppBar",
                "App Bar",
                UiComponentCategory::Container,
                "app-bar",
            )
            .with_prop(enum_prop_with_options(
                "position",
                "fixed",
                ["absolute", "fixed", "relative", "static", "sticky"]
                    .into_iter()
                    .map(enum_option_descriptor),
            ))
            .with_prop(enum_prop_with_options(
                "color",
                "primary",
                [
                    "default",
                    "inherit",
                    "primary",
                    "secondary",
                    "transparent",
                    "error",
                    "info",
                    "success",
                    "warning",
                ]
                .into_iter()
                .map(enum_option_descriptor),
            )),
            [("elevation", 4.0)],
        )
        .default_prop("elevation", UiValue::Float(4.0))
        .with_prop(bool_prop("enableColorOnDark", false))
        .with_prop(bool_prop("square", true))
        .slot(UiSlotSchema::new("navigation"))
        .slot(UiSlotSchema::new("title"))
        .slot(UiSlotSchema::new("actions").multiple(true)),
        composite("Card", "Card", UiComponentCategory::Container, "card")
            .with_prop(enum_prop_with_options(
                "variant",
                "elevation",
                ["elevation", "outlined"]
                    .into_iter()
                    .map(enum_option_descriptor),
            ))
            .with_prop(bool_prop("raised", false))
            .slot(UiSlotSchema::new("media"))
            .slot(UiSlotSchema::new("content").multiple(true))
            .slot(UiSlotSchema::new("actions").multiple(true)),
        composite(
            "CardActionArea",
            "Card Action Area",
            UiComponentCategory::Container,
            "card-action-area",
        )
        .with_prop(default_string_prop("focusVisibleClassName", ""))
        .slot(UiSlotSchema::new("content").multiple(true))
        .slot(UiSlotSchema::new("focusHighlight")),
        composite(
            "CardActions",
            "Card Actions",
            UiComponentCategory::Container,
            "card-actions",
        )
        .with_prop(bool_prop("disableSpacing", false))
        .slot(UiSlotSchema::new("content").multiple(true)),
        composite(
            "CardContent",
            "Card Content",
            UiComponentCategory::Container,
            "card-content",
        )
        .with_prop(default_string_prop("component", "div"))
        .slot(UiSlotSchema::new("content").multiple(true)),
        composite(
            "CardHeader",
            "Card Header",
            UiComponentCategory::Container,
            "card-header",
        )
        .with_prop(default_string_prop("component", "div"))
        .with_prop(default_string_prop("title", ""))
        .with_prop(default_string_prop("subheader", ""))
        .with_prop(default_string_prop("avatar", ""))
        .with_prop(default_string_prop("action", ""))
        .with_prop(bool_prop("disableTypography", false))
        .slot(UiSlotSchema::new("avatar"))
        .slot(UiSlotSchema::new("content"))
        .slot(UiSlotSchema::new("title"))
        .slot(UiSlotSchema::new("subheader"))
        .slot(UiSlotSchema::new("action")),
        composite(
            "CardMedia",
            "Card Media",
            UiComponentCategory::Container,
            "card-media",
        )
        .with_prop(default_string_prop("component", "div"))
        .with_prop(default_string_prop("image", ""))
        .with_prop(default_string_prop("src", ""))
        .slot(UiSlotSchema::new("content").multiple(true)),
        override_float_prop_defaults(
            composite("Paper", "Paper", UiComponentCategory::Container, "paper")
                .with_prop(default_string_prop("component", "div"))
                .with_prop(enum_prop_with_options(
                    "variant",
                    "elevation",
                    ["elevation", "outlined"]
                        .into_iter()
                        .map(enum_option_descriptor),
                )),
            [("elevation", 1.0)],
        )
        .default_prop("elevation", UiValue::Float(1.0))
        .with_prop(bool_prop("square", false))
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
        composite(
            "Toolbar",
            "Toolbar",
            UiComponentCategory::Container,
            "toolbar",
        )
        .with_prop(default_string_prop("component", "div"))
        .with_prop(bool_prop("disableGutters", false))
        .with_prop(enum_prop_with_options(
            "variant",
            "regular",
            ["regular", "dense"].into_iter().map(enum_option_descriptor),
        ))
        .slot(UiSlotSchema::new("content").multiple(true)),
    ]
}

fn override_float_prop_defaults(
    mut descriptor: UiComponentDescriptor,
    defaults: impl IntoIterator<Item = (&'static str, f64)>,
) -> UiComponentDescriptor {
    for (name, default) in defaults {
        if let Some(schema) = descriptor
            .prop_schema
            .iter_mut()
            .find(|schema| schema.name == name)
        {
            schema.default_value = Some(UiValue::Float(default));
        } else {
            descriptor = descriptor.with_prop(float_prop(name, default));
        }
    }
    descriptor
}
