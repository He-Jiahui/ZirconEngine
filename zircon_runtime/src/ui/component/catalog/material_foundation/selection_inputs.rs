use super::shared::*;

pub(super) fn descriptors() -> Vec<UiComponentDescriptor> {
    vec![
        primitive("Select", "Select", UiComponentCategory::Selection, "select")
            .with_prop(select_options_prop())
            .with_prop(default_string_prop("value", "primary"))
            .with_prop(value_text_prop())
            .with_prop(array_prop("selected_options"))
            .with_prop(default_string_prop("label", "Select"))
            .with_prop(default_string_prop("placeholder", "Choose option"))
            .with_prop(string_prop("helper_text"))
            .with_prop(enum_prop_with_options(
                "variant",
                "outlined",
                ["outlined", "filled", "standard"]
                    .into_iter()
                    .map(enum_option_descriptor),
            ))
            .with_prop(bool_prop("multiple", false))
            .with_prop(bool_prop("display_empty", false))
            .with_prop(bool_prop("popup_open", false))
            .with_prop(array_prop("disabled_options"))
            .with_prop(array_prop("focused_options"))
            .with_prop(array_prop("hovered_options"))
            .with_prop(array_prop("pressed_options"))
            .events([
                UiComponentEventKind::Focus,
                UiComponentEventKind::OpenPopup,
                UiComponentEventKind::SelectOption,
                UiComponentEventKind::ClosePopup,
                UiComponentEventKind::ValueChanged,
            ]),
        primitive(
            "Dropdown",
            "Dropdown",
            UiComponentCategory::Selection,
            "dropdown",
        )
        .with_prop(options_prop())
        .with_prop(value_text_prop())
        .event(UiComponentEventKind::ValueChanged),
        composite(
            "Autocomplete",
            "Autocomplete",
            UiComponentCategory::Selection,
            "autocomplete",
        )
        .with_prop(string_prop("query"))
        .with_prop(autocomplete_options_prop())
        .with_prop(array_prop("filtered_options"))
        .with_prop(default_string_prop("value", "atlas"))
        .with_prop(value_text_prop())
        .with_prop(array_prop("selected_options"))
        .with_prop(array_prop("disabled_options"))
        .with_prop(array_prop("focused_options"))
        .with_prop(array_prop("hovered_options"))
        .with_prop(array_prop("pressed_options"))
        .with_prop(array_prop("matched_options"))
        .with_prop(bool_prop("multiple", false))
        .with_prop(bool_prop("free_solo", false))
        .with_prop(bool_prop("popup_open", false))
        .slot(UiSlotSchema::new("listbox").multiple(true))
        .events([
            UiComponentEventKind::Focus,
            UiComponentEventKind::ValueChanged,
            UiComponentEventKind::OpenPopup,
            UiComponentEventKind::SelectOption,
            UiComponentEventKind::ClosePopup,
            UiComponentEventKind::RemoveElement,
        ])
        .requires_host_capability(UiHostCapability::TextInput),
        composite(
            "ToggleButtonGroup",
            "Toggle Button Group",
            UiComponentCategory::Selection,
            "toggle-button-group",
        )
        .with_prop(enum_prop("selection_state", "exclusive"))
        .with_prop(value_text_prop())
        .slot(UiSlotSchema::new("buttons").multiple(true))
        .event(UiComponentEventKind::SelectOption),
    ]
}

fn select_options_prop() -> UiPropSchema {
    UiPropSchema::new("options", UiValueKind::Array)
        .default_value(UiValue::Array(vec![
            UiValue::String("primary".to_string()),
            UiValue::String("secondary".to_string()),
            UiValue::String("disabled".to_string()),
        ]))
        .with_options([
            UiOptionDescriptor::new("primary", "Primary", UiValue::String("primary".to_string())),
            UiOptionDescriptor::new(
                "secondary",
                "Secondary",
                UiValue::String("secondary".to_string()),
            ),
            UiOptionDescriptor::new(
                "disabled",
                "Disabled",
                UiValue::String("disabled".to_string()),
            )
            .disabled(true),
        ])
}

fn autocomplete_options_prop() -> UiPropSchema {
    UiPropSchema::new("options", UiValueKind::Array)
        .default_value(UiValue::Array(vec![
            UiValue::String("atlas".to_string()),
            UiValue::String("asset".to_string()),
            UiValue::String("disabled".to_string()),
        ]))
        .with_options([
            UiOptionDescriptor::new("atlas", "Atlas", UiValue::String("atlas".to_string())),
            UiOptionDescriptor::new("asset", "Asset", UiValue::String("asset".to_string())),
            UiOptionDescriptor::new(
                "disabled",
                "Disabled",
                UiValue::String("disabled".to_string()),
            )
            .disabled(true),
        ])
}
