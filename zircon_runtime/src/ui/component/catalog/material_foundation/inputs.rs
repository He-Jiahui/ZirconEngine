use super::shared::*;
use zircon_runtime_interface::ui::style::ButtonVariant;

pub(super) fn descriptors() -> Vec<UiComponentDescriptor> {
    vec![
        primitive(
            "NumberField",
            "Number Field",
            UiComponentCategory::Numeric,
            "number-field",
        )
        .with_prop(field_number_value_prop())
        .with_prop(float_prop("min", 0.0))
        .with_prop(float_prop("max", 100.0))
        .with_prop(float_prop("step", 1.0))
        .with_prop(float_prop("large_step", 10.0))
        .default_prop("value", UiValue::Float(0.0))
        .default_prop("min", UiValue::Float(0.0))
        .default_prop("max", UiValue::Float(100.0))
        .default_prop("step", UiValue::Float(1.0))
        .default_prop("large_step", UiValue::Float(10.0))
        .events([
            UiComponentEventKind::Focus,
            UiComponentEventKind::BeginDrag,
            UiComponentEventKind::DragDelta,
            UiComponentEventKind::LargeDragDelta,
            UiComponentEventKind::EndDrag,
            UiComponentEventKind::ValueChanged,
            UiComponentEventKind::Commit,
        ]),
        primitive(
            "Checkbox",
            "Checkbox",
            UiComponentCategory::Input,
            "checkbox",
        )
        .with_prop(text_prop())
        .with_prop(checked_prop())
        .with_prop(bool_prop("indeterminate", false))
        .with_prop(bool_prop("label_click_toggles", true))
        .with_prop(bool_prop("indeterminate_resolves_to_checked", true))
        .default_prop("text", UiValue::String("Checkbox".to_string()))
        .state(bool_prop("indeterminate", false))
        .events([
            UiComponentEventKind::Focus,
            UiComponentEventKind::ValueChanged,
        ]),
        primitive("Radio", "Radio", UiComponentCategory::Input, "radio")
            .with_prop(text_prop())
            .with_prop(checked_prop())
            .with_prop(default_string_prop("group_value", "editor"))
            .with_prop(default_string_prop("option_id", "editor"))
            .with_prop(radio_options_prop())
            .with_prop(array_prop("disabled_options"))
            .with_prop(bool_prop("label_click_selects", true))
            .with_prop(bool_prop("exclusive_group", true))
            .with_prop(bool_prop("keyboard_navigation", true))
            .default_prop("text", UiValue::String("Radio".to_string()))
            .events([
                UiComponentEventKind::Focus,
                UiComponentEventKind::SelectOption,
                UiComponentEventKind::ValueChanged,
            ]),
        primitive("Switch", "Switch", UiComponentCategory::Input, "switch")
            .with_prop(text_prop())
            .with_prop(checked_prop())
            .with_prop(enum_prop_with_options(
                "switch_size",
                "medium",
                ["small", "medium"].into_iter().map(enum_option_descriptor),
            ))
            .with_prop(enum_prop_with_options(
                "switch_color",
                "primary",
                ["primary", "default", "error"]
                    .into_iter()
                    .map(enum_option_descriptor),
            ))
            .with_prop(bool_prop("label_click_toggles", true))
            .with_prop(bool_prop("track_click_toggles", true))
            .with_prop(bool_prop("thumb_draggable", false))
            .default_prop("text", UiValue::String("Switch".to_string()))
            .events([
                UiComponentEventKind::Focus,
                UiComponentEventKind::ValueChanged,
            ]),
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
        with_button_variant_default(
            primitive(
                "ToggleButton",
                "Toggle Button",
                UiComponentCategory::Input,
                "toggle-button",
            ),
            "outlined",
        )
        .with_prop(text_prop())
        .with_prop(checked_prop())
        .with_prop(button_color_prop())
        .with_prop(button_size_prop())
        .with_prop(button_icon_placement_prop("none"))
        .with_prop(enum_prop_with_options(
            "selection_state",
            "exclusive",
            ["exclusive", "multiple"]
                .into_iter()
                .map(enum_option_descriptor),
        ))
        .default_prop("button_variant", UiValue::Enum("outlined".to_string()))
        .default_prop("button_color", UiValue::Enum("primary".to_string()))
        .default_prop("button_size", UiValue::Enum("medium".to_string()))
        .default_prop("icon_placement", UiValue::Enum("none".to_string()))
        .default_prop("selection_state", UiValue::Enum("exclusive".to_string()))
        .event(UiComponentEventKind::ValueChanged),
        primitive("Rating", "Rating", UiComponentCategory::Numeric, "rating")
            .with_prop(number_value_prop())
            .with_prop(int_prop("max", 5))
            .event(UiComponentEventKind::ValueChanged),
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
    ]
}

fn radio_options_prop() -> UiPropSchema {
    UiPropSchema::new("options", UiValueKind::Array)
        .default_value(UiValue::Array(vec![
            UiValue::String("editor".to_string()),
            UiValue::String("runtime".to_string()),
            UiValue::String("disabled".to_string()),
            UiValue::String("qa".to_string()),
        ]))
        .with_options([
            UiOptionDescriptor::new("editor", "Editor", UiValue::String("editor".to_string())),
            UiOptionDescriptor::new("runtime", "Runtime", UiValue::String("runtime".to_string())),
            UiOptionDescriptor::new(
                "disabled",
                "Disabled",
                UiValue::String("disabled".to_string()),
            )
            .disabled(true),
            UiOptionDescriptor::new("qa", "QA", UiValue::String("qa".to_string())),
        ])
}

fn with_button_variant_default(
    mut descriptor: UiComponentDescriptor,
    default: &str,
) -> UiComponentDescriptor {
    let replacement = enum_prop_with_options(
        "button_variant",
        default,
        ButtonVariant::OPTIONS
            .into_iter()
            .map(enum_option_descriptor),
    );
    if let Some(schema) = descriptor
        .prop_schema
        .iter_mut()
        .find(|schema| schema.name == "button_variant")
    {
        *schema = replacement;
    } else {
        descriptor.prop_schema.push(replacement);
    }
    descriptor
}
