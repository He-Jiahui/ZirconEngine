use std::{collections::BTreeSet, fs, path::Path};

use crate::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime_interface::ui::component::{
    UiComponentDescriptorKind, UiComponentEventKind, UiComponentLayoutRole, UiHostCapability,
    UiHostCapabilitySet, UiRenderCapability, UiValue, UiValueKind,
};
use zircon_runtime_interface::ui::style::{
    ButtonColor, ButtonIconPlacement, ButtonSize, ButtonVariant,
};

use super::{assert_has_event, assert_has_prop};

mod button_inputs;
mod data_display;
mod data_display_subcomponents;
mod editor_components;
mod feedback;
mod folder_structure;
mod form_controls;
mod inputs;
mod lab_subcomponents;
mod layout;
mod mui_surface_overlay;
mod mui_web_inventory;
mod mui_x;
mod mui_x_runtime;
mod navigation;
mod navigation_editor;
mod navigation_secondary;
mod planned_layers;
mod selection_inputs;
mod surface_subcomponents;
mod surfaces;
mod virtualization;

fn assert_button_style_schema(
    descriptor: &zircon_runtime_interface::ui::component::UiComponentDescriptor,
    expected_icon_placement: &str,
) {
    assert_button_style_schema_with_variant_default(descriptor, expected_icon_placement, "default");
}

pub(super) fn assert_button_style_schema_with_variant_default(
    descriptor: &zircon_runtime_interface::ui::component::UiComponentDescriptor,
    expected_icon_placement: &str,
    expected_variant_default: &str,
) {
    assert_enum_options(descriptor, "button_variant", &ButtonVariant::OPTIONS);
    assert_enum_options(descriptor, "button_color", &ButtonColor::OPTIONS);
    assert_enum_options(descriptor, "button_size", &ButtonSize::OPTIONS);
    assert_enum_options(descriptor, "icon_placement", &ButtonIconPlacement::OPTIONS);
    assert_eq!(
        descriptor.prop("button_variant").unwrap().default_value,
        Some(UiValue::Enum(expected_variant_default.to_string()))
    );
    assert_eq!(
        descriptor.prop("button_variant").unwrap().value_kind,
        UiValueKind::Enum
    );
    assert_eq!(
        descriptor.prop("icon_placement").unwrap().default_value,
        Some(UiValue::Enum(expected_icon_placement.to_string()))
    );
}

pub(super) fn assert_enum_options(
    descriptor: &zircon_runtime_interface::ui::component::UiComponentDescriptor,
    name: &str,
    expected: &[&str],
) {
    let schema = descriptor
        .prop(name)
        .unwrap_or_else(|| panic!("{} missing prop `{name}`", descriptor.id));
    assert_eq!(schema.value_kind, UiValueKind::Enum);
    assert_eq!(
        schema
            .options
            .iter()
            .map(|option| option.id.as_str())
            .collect::<Vec<_>>(),
        expected
    );
}

fn assert_mui_web_customization_schema(
    descriptor: &zircon_runtime_interface::ui::component::UiComponentDescriptor,
) {
    for (name, expected_default) in [
        ("mui_variant", ""),
        ("mui_color", "primary"),
        ("mui_size", "medium"),
    ] {
        let schema = descriptor
            .prop(name)
            .unwrap_or_else(|| panic!("{} missing prop `{name}`", descriptor.id));
        assert_eq!(schema.value_kind, UiValueKind::String);
        assert_eq!(
            schema.default_value,
            Some(UiValue::String(expected_default.to_string())),
            "{} should default `{name}` to `{expected_default}`",
            descriptor.id
        );
    }

    for name in [
        "mui_slots",
        "mui_slot_props",
        "mui_sx",
        "slots",
        "slotProps",
        "sx",
        "classes",
    ] {
        let schema = descriptor
            .prop(name)
            .unwrap_or_else(|| panic!("{} missing prop `{name}`", descriptor.id));
        assert_eq!(schema.value_kind, UiValueKind::Map);
        assert_eq!(
            schema.default_value,
            Some(UiValue::Map(Default::default())),
            "{} should default `{name}` to an empty map",
            descriptor.id
        );
    }

    let classes = descriptor
        .prop("mui_classes")
        .unwrap_or_else(|| panic!("{} missing prop `mui_classes`", descriptor.id));
    assert_eq!(classes.value_kind, UiValueKind::Array);
    assert_eq!(
        classes.default_value,
        Some(UiValue::Array(Vec::new())),
        "{} should default `mui_classes` to an empty array",
        descriptor.id
    );

    let class_name = descriptor
        .prop("className")
        .unwrap_or_else(|| panic!("{} missing prop `className`", descriptor.id));
    assert_eq!(class_name.value_kind, UiValueKind::String);
    assert_eq!(
        class_name.default_value,
        Some(UiValue::String(String::new())),
        "{} should default `className` to an empty string",
        descriptor.id
    );
}
