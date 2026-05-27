use crate::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime_interface::ui::component::{UiComponentEventKind, UiValue};

use super::super::{assert_has_event, assert_has_prop};
use super::assert_enum_options;

pub(super) fn assert_descriptors(registry: &UiComponentDescriptorRegistry) {
    assert_menu_list(registry);
    assert_mobile_stepper(registry);
    assert_tab_scroll_button(registry);
}

fn assert_menu_list(registry: &UiComponentDescriptorRegistry) {
    let menu_list = descriptor(registry, "MenuList");
    assert_enum_options(menu_list, "variant", &["menu", "selectedMenu"]);
    for prop in [
        "autoFocus",
        "autoFocusItem",
        "disabledItemsFocusable",
        "disableListWrap",
    ] {
        assert_has_prop(menu_list, prop);
    }
    assert_has_slot(menu_list, "items");
    assert_default_value(
        menu_list,
        "variant",
        UiValue::Enum("selectedMenu".to_string()),
    );
    assert_has_event(menu_list, UiComponentEventKind::SelectOption);
}

fn assert_mobile_stepper(registry: &UiComponentDescriptorRegistry) {
    let stepper = descriptor(registry, "MobileStepper");
    assert_enum_options(stepper, "position", &["bottom", "static", "top"]);
    assert_enum_options(stepper, "variant", &["dots", "progress", "text"]);
    for prop in ["activeStep", "steps"] {
        assert_has_prop(stepper, prop);
    }
    for slot in [
        "backButton",
        "nextButton",
        "dots",
        "dot",
        "dotActive",
        "progress",
    ] {
        assert_has_slot(stepper, slot);
    }
    assert_default_value(stepper, "activeStep", UiValue::Int(0));
    assert_default_value(stepper, "steps", UiValue::Int(1));
    assert_has_event(stepper, UiComponentEventKind::SetPage);
}

fn assert_tab_scroll_button(registry: &UiComponentDescriptorRegistry) {
    let button = descriptor(registry, "TabScrollButton");
    assert_enum_options(button, "direction", &["left", "right"]);
    assert_enum_options(button, "orientation", &["horizontal", "vertical"]);
    assert_has_prop(button, "disabled");
    for slot in ["startScrollButtonIcon", "endScrollButtonIcon"] {
        assert_has_slot(button, slot);
    }
    assert_default_value(button, "disabled", UiValue::Bool(false));
    assert_has_event(button, UiComponentEventKind::SelectOption);
}

fn descriptor<'a>(
    registry: &'a UiComponentDescriptorRegistry,
    id: &str,
) -> &'a zircon_runtime_interface::ui::component::UiComponentDescriptor {
    registry
        .descriptor(id)
        .unwrap_or_else(|| panic!("{id} descriptor"))
}

fn assert_has_slot(
    descriptor: &zircon_runtime_interface::ui::component::UiComponentDescriptor,
    slot_name: &str,
) {
    assert!(
        descriptor
            .slot_schema
            .iter()
            .any(|slot| slot.name == slot_name),
        "{} missing MUI slot `{slot_name}`",
        descriptor.id
    );
}

fn assert_default_value(
    descriptor: &zircon_runtime_interface::ui::component::UiComponentDescriptor,
    prop_name: &str,
    expected: UiValue,
) {
    assert_eq!(
        descriptor
            .prop(prop_name)
            .unwrap_or_else(|| panic!("{} missing prop `{prop_name}`", descriptor.id))
            .default_value
            .clone(),
        Some(expected),
        "{} should expose local MUI default for `{prop_name}`",
        descriptor.id
    );
}
