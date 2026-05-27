use crate::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime_interface::ui::component::{UiComponentEventKind, UiRenderCapability, UiValue};

use super::super::{assert_has_event, assert_has_prop};
use super::assert_enum_options;

pub(super) fn assert_descriptors(registry: &UiComponentDescriptorRegistry) {
    assert_accordion_subcomponents(registry);
    assert_dialog_subcomponents(registry);
    assert_speed_dial_subcomponents(registry);
    assert_swipeable_drawer(registry);
}

fn assert_accordion_subcomponents(registry: &UiComponentDescriptorRegistry) {
    let summary = descriptor(registry, "AccordionSummary");
    for prop in [
        "text",
        "expandIcon",
        "expanded",
        "disabled",
        "disableGutters",
        "focusVisibleClassName",
    ] {
        assert_has_prop(summary, prop);
    }
    for slot in ["content", "expandIconWrapper", "expandIcon"] {
        assert_has_slot(summary, slot);
    }
    assert_has_event(summary, UiComponentEventKind::ToggleExpanded);
    assert_default_value(summary, "expanded", UiValue::Bool(false));

    let actions = descriptor(registry, "AccordionActions");
    assert_has_prop(actions, "disableSpacing");
    assert_has_slot(actions, "content");
    assert_default_value(actions, "disableSpacing", UiValue::Bool(false));

    assert_has_slot(descriptor(registry, "AccordionDetails"), "content");
}

fn assert_dialog_subcomponents(registry: &UiComponentDescriptorRegistry) {
    let actions = descriptor(registry, "DialogActions");
    assert_has_prop(actions, "disableSpacing");
    assert_has_slot(actions, "content");
    assert_has_event(actions, UiComponentEventKind::Commit);

    let content = descriptor(registry, "DialogContent");
    assert_has_prop(content, "dividers");
    assert_has_slot(content, "content");
    assert_default_value(content, "dividers", UiValue::Bool(false));

    let content_text = descriptor(registry, "DialogContentText");
    for prop in ["text", "component", "variant", "color"] {
        assert_has_prop(content_text, prop);
    }
    assert_default_value(content_text, "component", UiValue::String("p".to_string()));
    assert_default_value(
        content_text,
        "variant",
        UiValue::String("body1".to_string()),
    );
    assert_default_value(
        content_text,
        "color",
        UiValue::String("textSecondary".to_string()),
    );

    let title = descriptor(registry, "DialogTitle");
    for prop in ["text", "component", "variant"] {
        assert_has_prop(title, prop);
    }
    assert_default_value(title, "component", UiValue::String("h2".to_string()));
    assert_default_value(title, "variant", UiValue::String("h6".to_string()));
}

fn assert_speed_dial_subcomponents(registry: &UiComponentDescriptorRegistry) {
    let action = descriptor(registry, "SpeedDialAction");
    assert_enum_options(action, "tooltipPlacement", &["left", "right"]);
    for prop in ["icon", "tooltipTitle", "tooltipOpen", "open", "delay"] {
        assert_has_prop(action, prop);
    }
    for slot in ["fab", "icon", "staticTooltipLabel"] {
        assert_has_slot(action, slot);
    }
    assert_has_event(action, UiComponentEventKind::SelectOption);

    let icon = descriptor(registry, "SpeedDialIcon");
    for prop in ["icon", "open", "openIcon"] {
        assert_has_prop(icon, prop);
    }
    for slot in ["icon", "openIcon"] {
        assert_has_slot(icon, slot);
    }
    assert!(icon
        .required_render_capabilities
        .contains(&UiRenderCapability::Vector));
}

fn assert_swipeable_drawer(registry: &UiComponentDescriptorRegistry) {
    let drawer = descriptor(registry, "SwipeableDrawer");
    assert_enum_options(drawer, "anchor", &["left", "right", "top", "bottom"]);
    assert_enum_options(drawer, "variant", &["temporary", "persistent", "permanent"]);
    for prop in [
        "open",
        "disableBackdropTransition",
        "disableDiscovery",
        "disableSwipeToOpen",
        "allowSwipeInChildren",
        "hideBackdrop",
        "hysteresis",
        "minFlingVelocity",
        "swipeAreaWidth",
        "z_index",
        "disable_portal",
        "portal_layer",
    ] {
        assert_has_prop(drawer, prop);
    }
    for slot in [
        "root",
        "backdrop",
        "docked",
        "paper",
        "transition",
        "swipeArea",
        "content",
    ] {
        assert_has_slot(drawer, slot);
    }
    assert_default_value(drawer, "hysteresis", UiValue::Float(0.52));
    assert_default_value(drawer, "minFlingVelocity", UiValue::Float(450.0));
    assert_default_value(drawer, "swipeAreaWidth", UiValue::Float(20.0));
    assert_has_event(drawer, UiComponentEventKind::OpenPopup);
    assert_has_event(drawer, UiComponentEventKind::ClosePopup);
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
