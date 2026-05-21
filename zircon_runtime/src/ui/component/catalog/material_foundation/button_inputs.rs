use super::shared::*;

pub(super) fn descriptors() -> Vec<UiComponentDescriptor> {
    vec![
        button(),
        button_group(),
        icon_button(),
        floating_action_button(),
    ]
}

fn button() -> UiComponentDescriptor {
    primitive("Button", "Button", UiComponentCategory::Input, "button")
        .with_prop(text_prop())
        .with_prop(icon_prop())
        .with_prop(button_color_prop())
        .with_prop(button_size_prop())
        .with_prop(button_icon_placement_prop("none"))
        .with_prop(enum_prop_with_options(
            "slint_material_button_variant",
            "filled",
            ["filled", "outlined", "text", "tonal", "elevated"]
                .into_iter()
                .map(enum_option_descriptor),
        ))
        .with_prop(float_prop("button_horizontal_padding", 24.0))
        .with_prop(float_prop("button_vertical_padding", 10.0))
        .with_prop(float_prop("button_spacing", 8.0))
        .with_prop(float_prop("min_layout_width", 40.0))
        .with_prop(float_prop("min_layout_height", 40.0))
        .with_prop(float_prop("icon_size", 18.0))
        .with_prop(float_prop("hover_elevation", 1.0))
        .with_prop(bool_prop("state_layer_enabled", true))
        .with_prop(bool_prop("ripple_enabled", true))
        .with_prop(bool_prop("clip_ripple", true))
        .default_prop("text", UiValue::String("Button".to_string()))
        .default_prop("button_variant", UiValue::Enum("default".to_string()))
        .default_prop("button_color", UiValue::Enum("primary".to_string()))
        .default_prop("button_size", UiValue::Enum("medium".to_string()))
        .default_prop("icon_placement", UiValue::Enum("none".to_string()))
        .event(UiComponentEventKind::Commit)
}

fn button_group() -> UiComponentDescriptor {
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
    .with_prop(button_color_prop())
    .with_prop(button_size_prop())
    .with_prop(button_icon_placement_prop("none"))
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
    .slot(UiSlotSchema::new("buttons").multiple(true))
}

fn icon_button() -> UiComponentDescriptor {
    primitive(
        "IconButton",
        "Icon Button",
        UiComponentCategory::Input,
        "icon-button",
    )
    .with_prop(text_prop())
    .with_prop(icon_prop())
    .with_prop(default_string_prop("checked_icon", ""))
    .with_prop(checked_prop())
    .with_prop(bool_prop("checkable", false))
    .with_prop(bool_prop("inverse", false))
    .with_prop(bool_prop("inline", false))
    .with_prop(bool_prop("has_error", false))
    .with_prop(button_color_prop())
    .with_prop(button_size_prop())
    .with_prop(button_icon_placement_prop("icon_only"))
    .with_prop(bool_prop("display_background", false))
    .with_prop(float_prop("icon_size", 24.0))
    .with_prop(float_prop("inline_icon_size", 18.0))
    .with_prop(float_prop("min_layout_width", 40.0))
    .with_prop(float_prop("min_layout_height", 40.0))
    .with_prop(bool_prop("clip_ripple", true))
    .with_prop(bool_prop("state_layer_enabled", true))
    .with_prop(bool_prop("ripple_enabled", true))
    .default_prop("button_variant", UiValue::Enum("default".to_string()))
    .default_prop("button_color", UiValue::Enum("primary".to_string()))
    .default_prop("button_size", UiValue::Enum("medium".to_string()))
    .default_prop("icon_placement", UiValue::Enum("icon_only".to_string()))
    .event(UiComponentEventKind::Commit)
    .requires_render_capability(UiRenderCapability::Vector)
}

fn floating_action_button() -> UiComponentDescriptor {
    let descriptor = primitive(
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
        "fab_style",
        "standard",
        ["small", "standard", "large"]
            .into_iter()
            .map(enum_option_descriptor),
    ))
    .with_prop(enum_prop_with_options(
        "button_shape",
        "circular",
        ["circular", "extended", "pill"]
            .into_iter()
            .map(enum_option_descriptor),
    ))
    .with_prop(float_prop("min_layout_width", 56.0))
    .with_prop(float_prop("min_layout_height", 56.0))
    .with_prop(float_prop("button_horizontal_padding", 14.0))
    .with_prop(float_prop("button_vertical_padding", 14.0))
    .with_prop(float_prop("icon_size", 24.0))
    .with_prop(float_prop("hover_elevation", 4.0))
    .with_prop(bool_prop("state_layer_enabled", true))
    .with_prop(bool_prop("ripple_enabled", true))
    .with_prop(bool_prop("clip_ripple", true));

    override_float_prop_defaults(
        descriptor,
        [
            ("corner_radius", 16.0),
            ("border_width", 0.0),
            ("elevation", 3.0),
        ],
    )
    .default_prop("button_variant", UiValue::Enum("contained".to_string()))
    .default_prop("button_color", UiValue::Enum("primary".to_string()))
    .default_prop("button_size", UiValue::Enum("medium".to_string()))
    .default_prop("icon_placement", UiValue::Enum("icon_only".to_string()))
    .default_prop("fab_style", UiValue::Enum("standard".to_string()))
    .default_prop("button_shape", UiValue::Enum("circular".to_string()))
    .default_prop("surface_variant", UiValue::Enum("elevated".to_string()))
    .default_prop("corner_radius", UiValue::Float(16.0))
    .default_prop("border_width", UiValue::Float(0.0))
    .default_prop("elevation", UiValue::Float(3.0))
    .event(UiComponentEventKind::Commit)
    .requires_render_capability(UiRenderCapability::Vector)
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
