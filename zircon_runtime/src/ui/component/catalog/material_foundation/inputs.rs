use super::shared::*;

pub(super) fn descriptors() -> Vec<UiComponentDescriptor> {
    vec![
        primitive("Button", "Button", UiComponentCategory::Input, "button")
            .with_prop(text_prop())
            .with_prop(button_color_prop())
            .with_prop(button_size_prop())
            .with_prop(button_icon_placement_prop("none"))
            .default_prop("text", UiValue::String("Button".to_string()))
            .default_prop("button_variant", UiValue::Enum("default".to_string()))
            .default_prop("button_color", UiValue::Enum("primary".to_string()))
            .default_prop("button_size", UiValue::Enum("medium".to_string()))
            .default_prop("icon_placement", UiValue::Enum("none".to_string()))
            .event(UiComponentEventKind::Commit),
        composite(
            "ButtonGroup",
            "Button Group",
            UiComponentCategory::Input,
            "button-group",
        )
        .with_prop(enum_prop("orientation", "horizontal"))
        .with_prop(enum_prop_with_options(
            "button_group_orientation",
            "horizontal",
            ["horizontal", "vertical"]
                .into_iter()
                .map(enum_option_descriptor),
        ))
        .with_prop(enum_prop_with_options(
            "button_group_attached_radius",
            "only",
            ["first", "middle", "last", "only"]
                .into_iter()
                .map(enum_option_descriptor),
        ))
        .with_prop(int_prop("button_group_segment_count", 1))
        .with_prop(bool_prop("button_group_disabled_propagates", true))
        .default_prop(
            "button_group_orientation",
            UiValue::Enum("horizontal".to_string()),
        )
        .default_prop(
            "button_group_attached_radius",
            UiValue::Enum("only".to_string()),
        )
        .default_prop("button_group_segment_count", UiValue::Int(1))
        .default_prop("button_group_disabled_propagates", UiValue::Bool(true))
        .default_prop("button_variant", UiValue::Enum("contained".to_string()))
        .default_prop("button_color", UiValue::Enum("primary".to_string()))
        .default_prop("button_size", UiValue::Enum("medium".to_string()))
        .default_prop("icon_placement", UiValue::Enum("none".to_string()))
        .slot(UiSlotSchema::new("buttons").multiple(true)),
        primitive(
            "IconButton",
            "Icon Button",
            UiComponentCategory::Input,
            "icon-button",
        )
        .with_prop(text_prop())
        .with_prop(icon_prop())
        .with_prop(button_color_prop())
        .with_prop(button_size_prop())
        .with_prop(button_icon_placement_prop("icon_only"))
        .default_prop("button_variant", UiValue::Enum("default".to_string()))
        .default_prop("button_color", UiValue::Enum("primary".to_string()))
        .default_prop("button_size", UiValue::Enum("medium".to_string()))
        .default_prop("icon_placement", UiValue::Enum("icon_only".to_string()))
        .event(UiComponentEventKind::Commit)
        .requires_render_capability(UiRenderCapability::Vector),
        primitive(
            "FloatingActionButton",
            "Floating Action Button",
            UiComponentCategory::Input,
            "fab",
        )
        .with_prop(icon_prop())
        .with_prop(text_prop())
        .with_prop(button_color_prop())
        .with_prop(button_size_prop())
        .with_prop(button_icon_placement_prop("icon_only"))
        .with_prop(enum_prop_with_options(
            "button_shape",
            "circular",
            ["circular", "extended", "pill"]
                .into_iter()
                .map(enum_option_descriptor),
        ))
        .default_prop("button_variant", UiValue::Enum("contained".to_string()))
        .default_prop("button_color", UiValue::Enum("primary".to_string()))
        .default_prop("button_size", UiValue::Enum("medium".to_string()))
        .default_prop("icon_placement", UiValue::Enum("icon_only".to_string()))
        .default_prop("button_shape", UiValue::Enum("circular".to_string()))
        .default_prop("surface_variant", UiValue::Enum("elevated".to_string()))
        .default_prop("corner_radius", UiValue::Float(999.0))
        .default_prop("border_width", UiValue::Float(0.0))
        .default_prop("elevation", UiValue::Float(2.0))
        .event(UiComponentEventKind::Commit)
        .requires_render_capability(UiRenderCapability::Vector),
        primitive(
            "TextField",
            "Text Field",
            UiComponentCategory::Input,
            "text-field",
        )
        .with_prop(text_prop())
        .with_prop(string_prop("placeholder"))
        .with_prop(string_prop("helper_text"))
        .with_prop(enum_prop("variant", "outlined"))
        .events([
            UiComponentEventKind::Focus,
            UiComponentEventKind::ValueChanged,
            UiComponentEventKind::Commit,
        ])
        .requires_host_capability(UiHostCapability::TextInput),
        primitive("Input", "Input", UiComponentCategory::Input, "input")
            .with_prop(value_text_prop())
            .with_prop(string_prop("placeholder"))
            .events([
                UiComponentEventKind::Focus,
                UiComponentEventKind::ValueChanged,
                UiComponentEventKind::Commit,
            ])
            .requires_host_capability(UiHostCapability::TextInput),
        primitive(
            "TextareaAutosize",
            "Textarea Autosize",
            UiComponentCategory::Input,
            "textarea-autosize",
        )
        .with_prop(value_text_prop())
        .with_prop(int_prop("min_rows", 2))
        .with_prop(int_prop("max_rows", 8))
        .events([
            UiComponentEventKind::Focus,
            UiComponentEventKind::ValueChanged,
            UiComponentEventKind::Commit,
        ])
        .requires_host_capability(UiHostCapability::TextInput),
        primitive(
            "NumberField",
            "Number Field",
            UiComponentCategory::Numeric,
            "number-field",
        )
        .with_prop(number_value_prop())
        .with_prop(float_prop("min", 0.0))
        .with_prop(float_prop("max", 100.0))
        .with_prop(float_prop("step", 1.0))
        .events([
            UiComponentEventKind::Focus,
            UiComponentEventKind::BeginDrag,
            UiComponentEventKind::DragDelta,
            UiComponentEventKind::EndDrag,
            UiComponentEventKind::ValueChanged,
            UiComponentEventKind::Commit,
        ]),
        primitive("Select", "Select", UiComponentCategory::Selection, "select")
            .with_prop(options_prop())
            .with_prop(value_text_prop())
            .events([
                UiComponentEventKind::Focus,
                UiComponentEventKind::OpenPopup,
                UiComponentEventKind::SelectOption,
                UiComponentEventKind::ClosePopup,
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
        .with_prop(options_prop())
        .with_prop(value_text_prop())
        .slot(UiSlotSchema::new("listbox").multiple(true))
        .events([
            UiComponentEventKind::Focus,
            UiComponentEventKind::ValueChanged,
            UiComponentEventKind::OpenPopup,
            UiComponentEventKind::SelectOption,
            UiComponentEventKind::ClosePopup,
        ])
        .requires_host_capability(UiHostCapability::TextInput),
        primitive(
            "Checkbox",
            "Checkbox",
            UiComponentCategory::Input,
            "checkbox",
        )
        .with_prop(text_prop())
        .with_prop(checked_prop())
        .event(UiComponentEventKind::ValueChanged),
        primitive("Radio", "Radio", UiComponentCategory::Input, "radio")
            .with_prop(text_prop())
            .with_prop(checked_prop())
            .event(UiComponentEventKind::SelectOption),
        primitive("Switch", "Switch", UiComponentCategory::Input, "switch")
            .with_prop(checked_prop())
            .event(UiComponentEventKind::ValueChanged),
        primitive("Slider", "Slider", UiComponentCategory::Numeric, "slider")
            .with_prop(number_value_prop())
            .with_prop(bool_prop("show_value_label", true))
            .with_prop(array_prop("marks"))
            .events([
                UiComponentEventKind::Focus,
                UiComponentEventKind::BeginDrag,
                UiComponentEventKind::DragDelta,
                UiComponentEventKind::EndDrag,
                UiComponentEventKind::ValueChanged,
            ]),
        primitive(
            "ToggleButton",
            "Toggle Button",
            UiComponentCategory::Input,
            "toggle-button",
        )
        .with_prop(text_prop())
        .with_prop(checked_prop())
        .event(UiComponentEventKind::ValueChanged),
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
        primitive("Rating", "Rating", UiComponentCategory::Numeric, "rating")
            .with_prop(number_value_prop())
            .with_prop(int_prop("max", 5))
            .event(UiComponentEventKind::ValueChanged),
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
            "GizmoControls",
            "Gizmo Controls",
            UiComponentCategory::Input,
            "gizmo-controls",
        )
        .with_prop(enum_prop("mode", "translate"))
        .events([
            UiComponentEventKind::BeginDrag,
            UiComponentEventKind::DragDelta,
            UiComponentEventKind::EndDrag,
            UiComponentEventKind::SetWorldTransform,
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
