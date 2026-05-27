use super::shared::*;

pub(super) fn descriptors() -> Vec<UiComponentDescriptor> {
    vec![
        text_field_descriptor(),
        with_text_input_events(
            primitive("Input", "Input", UiComponentCategory::Input, "input")
                .with_prop(default_string_prop("value", ""))
                .with_prop(value_text_prop())
                .with_prop(string_prop("placeholder"))
                .with_prop(default_string_prop("type", "text"))
                .with_prop(default_string_prop("component", "div"))
                .with_prop(default_string_prop("inputComponent", "input"))
                .with_prop(bool_prop("autoFocus", false))
                .with_prop(bool_prop("readOnly", false))
                .with_prop(bool_prop("inputReadOnly", false))
                .with_prop(bool_prop("disableUnderline", false))
                .with_prop(bool_prop("fullWidth", false))
                .with_prop(bool_prop("formControl", false))
                .with_prop(bool_prop("multiline", false))
                .with_prop(default_string_prop("startAdornment", ""))
                .with_prop(default_string_prop("endAdornment", ""))
                .slot(UiSlotSchema::new("input"))
                .slot(UiSlotSchema::new("startAdornment"))
                .slot(UiSlotSchema::new("endAdornment")),
        ),
        textarea_autosize_descriptor(),
        editor_panel_component(
            "SearchField",
            "Search Field",
            UiComponentCategory::Input,
            "search-field",
        )
        .with_prop(string_prop("query"))
        .events([
            UiComponentEventKind::ValueChanged,
            UiComponentEventKind::Commit,
        ])
        .requires_host_capability(UiHostCapability::TextInput),
        editor_panel_component(
            "FieldEditor",
            "Field Editor",
            UiComponentCategory::Container,
            "field-editor",
        )
        .with_prop(text_prop())
        .with_prop(value_text_prop())
        .slot(UiSlotSchema::new("field").required(true))
        .events([
            UiComponentEventKind::ValueChanged,
            UiComponentEventKind::Commit,
        ]),
        editor_panel_component(
            "SourceEditor",
            "Source Editor",
            UiComponentCategory::Input,
            "source-editor",
        )
        .with_prop(text_prop())
        .events([
            UiComponentEventKind::ValueChanged,
            UiComponentEventKind::Commit,
        ])
        .requires_host_capability(UiHostCapability::TextInput),
    ]
}

fn text_field_descriptor() -> UiComponentDescriptor {
    with_text_input_events(
        primitive(
            "TextField",
            "Text Field",
            UiComponentCategory::Input,
            "text-field",
        )
        .with_prop(text_prop())
        .with_prop(value_text_prop())
        .with_prop(string_prop("label"))
        .with_prop(string_prop("placeholder"))
        .with_prop(string_prop("helper_text"))
        .with_prop(enum_prop_with_options(
            "variant",
            "outlined",
            ["outlined", "filled", "standard"]
                .into_iter()
                .map(enum_option_descriptor),
        ))
        .with_prop(bool_prop("multiline", false))
        .with_prop(bool_prop("select_mode", false))
        .default_prop("variant", UiValue::Enum("outlined".to_string()))
        .default_prop("value_text", UiValue::String(String::new()))
        .default_prop("label", UiValue::String(String::new()))
        .default_prop("placeholder", UiValue::String("Text".to_string()))
        .default_prop("helper_text", UiValue::String(String::new())),
    )
}

fn textarea_autosize_descriptor() -> UiComponentDescriptor {
    with_text_input_events(
        primitive(
            "TextareaAutosize",
            "Textarea Autosize",
            UiComponentCategory::Input,
            "textarea-autosize",
        )
        .with_prop(value_text_prop())
        .with_prop(string_prop("placeholder"))
        .with_prop(string_prop("helper_text"))
        .with_prop(enum_prop_with_options(
            "variant",
            "outlined",
            ["outlined", "filled", "standard"]
                .into_iter()
                .map(enum_option_descriptor),
        ))
        .with_prop(bool_prop("multiline", true))
        .with_prop(bool_prop("autosize", true))
        .with_prop(int_prop("min_rows", 2))
        .with_prop(int_prop("max_rows", 8))
        .default_prop("variant", UiValue::Enum("outlined".to_string()))
        .default_prop("value_text", UiValue::String(String::new()))
        .default_prop("placeholder", UiValue::String("Text".to_string()))
        .default_prop("helper_text", UiValue::String(String::new()))
        .default_prop("multiline", UiValue::Bool(true))
        .default_prop("autosize", UiValue::Bool(true))
        .default_prop("min_rows", UiValue::Int(2))
        .default_prop("max_rows", UiValue::Int(8)),
    )
}
