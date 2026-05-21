use crate::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime_interface::ui::component::{UiComponentEventKind, UiRenderCapability, UiValue};

use super::super::{assert_has_event, assert_has_prop};
use super::{assert_button_style_schema_with_variant_default, assert_enum_options};

pub(super) fn assert_descriptors(registry: &UiComponentDescriptorRegistry) {
    assert_button(registry);
    assert_button_group(registry);
    assert_icon_button(registry);
    assert_floating_action_button(registry);
}

fn assert_button(registry: &UiComponentDescriptorRegistry) {
    let button = registry.descriptor("Button").expect("Button descriptor");
    assert_button_style_schema_with_variant_default(button, "none", "default");
    assert_has_event(button, UiComponentEventKind::Commit);
    assert_enum_options(
        button,
        "slint_material_button_variant",
        &["filled", "outlined", "text", "tonal", "elevated"],
    );
    for (prop, expected) in [
        ("button_horizontal_padding", 24.0),
        ("button_vertical_padding", 10.0),
        ("button_spacing", 8.0),
        ("min_layout_width", 40.0),
        ("min_layout_height", 40.0),
        ("icon_size", 18.0),
        ("hover_elevation", 1.0),
    ] {
        assert_float_prop_default(button, prop, expected);
    }
    assert_enum_prop_default(button, "slint_material_button_variant", "filled");
    assert_bool_prop_default(button, "state_layer_enabled", true);
    assert_bool_prop_default(button, "ripple_enabled", true);
    assert_bool_prop_default(button, "clip_ripple", true);
}

fn assert_button_group(registry: &UiComponentDescriptorRegistry) {
    let button_group = registry
        .descriptor("ButtonGroup")
        .expect("ButtonGroup descriptor");
    assert_enum_options(
        button_group,
        "button_group_orientation",
        &["horizontal", "vertical"],
    );
    assert_enum_options(
        button_group,
        "button_group_attached_radius",
        &["first", "middle", "last", "only"],
    );
    assert_has_prop(button_group, "button_group_segment_count");
    assert_has_prop(button_group, "button_group_disabled_propagates");
    for (prop, expected) in [
        ("button_variant", "contained"),
        ("button_color", "primary"),
        ("button_size", "medium"),
        ("icon_placement", "none"),
    ] {
        assert_eq!(
            button_group
                .default_props
                .iter()
                .find(|(name, _)| name == prop)
                .map(|(_, value)| value),
            Some(&UiValue::Enum(expected.to_string())),
            "ButtonGroup should declare child button default `{prop}`"
        );
    }
    let button_group_slot = button_group
        .slot_schema("buttons")
        .expect("ButtonGroup buttons slot");
    assert!(button_group_slot.multiple);
    assert!(
        !button_group.supports_event(UiComponentEventKind::SelectOption),
        "ButtonGroup stays structural; child buttons own selection/click routes"
    );
    assert!(
        !button_group.supports_event(UiComponentEventKind::Commit),
        "ButtonGroup stays structural; child buttons own Commit/click routes"
    );
}

fn assert_icon_button(registry: &UiComponentDescriptorRegistry) {
    let icon_button = registry
        .descriptor("IconButton")
        .expect("IconButton descriptor");
    assert_button_style_schema_with_variant_default(icon_button, "icon_only", "default");
    assert_has_event(icon_button, UiComponentEventKind::Commit);
    for prop in [
        "checked_icon",
        "checkable",
        "checked",
        "inverse",
        "inline",
        "has_error",
    ] {
        assert_has_prop(icon_button, prop);
    }
    for (prop, expected) in [
        ("checkable", false),
        ("checked", false),
        ("inverse", false),
        ("inline", false),
        ("has_error", false),
        ("display_background", false),
        ("clip_ripple", true),
        ("state_layer_enabled", true),
        ("ripple_enabled", true),
    ] {
        assert_bool_prop_default(icon_button, prop, expected);
    }
    assert_float_prop_default(icon_button, "icon_size", 24.0);
    assert_float_prop_default(icon_button, "inline_icon_size", 18.0);
    assert_float_prop_default(icon_button, "min_layout_width", 40.0);
    assert_float_prop_default(icon_button, "min_layout_height", 40.0);
    assert!(icon_button
        .required_render_capabilities
        .contains(&UiRenderCapability::Vector));
}

fn assert_floating_action_button(registry: &UiComponentDescriptorRegistry) {
    let fab = registry
        .descriptor("FloatingActionButton")
        .expect("FloatingActionButton descriptor");
    assert_button_style_schema_with_variant_default(fab, "icon_only", "default");
    assert_enum_options(fab, "button_shape", &["circular", "extended", "pill"]);
    assert_enum_options(fab, "fab_style", &["small", "standard", "large"]);
    assert_enum_prop_default(fab, "fab_style", "standard");
    for (prop, expected) in [
        ("button_variant", "contained"),
        ("button_color", "primary"),
        ("button_size", "medium"),
        ("icon_placement", "icon_only"),
        ("button_shape", "circular"),
        ("surface_variant", "elevated"),
    ] {
        assert_eq!(
            fab.default_props
                .iter()
                .find(|(name, _)| name == prop)
                .map(|(_, value)| value),
            Some(&UiValue::Enum(expected.to_string())),
            "FloatingActionButton should declare default `{prop}`"
        );
    }
    for (prop, expected) in [
        ("min_layout_width", 56.0),
        ("min_layout_height", 56.0),
        ("button_horizontal_padding", 14.0),
        ("button_vertical_padding", 14.0),
        ("icon_size", 24.0),
        ("corner_radius", 16.0),
        ("elevation", 3.0),
        ("hover_elevation", 4.0),
    ] {
        assert_float_prop_default(fab, prop, expected);
    }
    assert_bool_prop_default(fab, "state_layer_enabled", true);
    assert_bool_prop_default(fab, "ripple_enabled", true);
    assert_bool_prop_default(fab, "clip_ripple", true);
    assert_has_event(fab, UiComponentEventKind::Commit);
    assert!(fab
        .required_render_capabilities
        .contains(&UiRenderCapability::Vector));
}

fn assert_bool_prop_default(
    descriptor: &zircon_runtime_interface::ui::component::UiComponentDescriptor,
    prop: &str,
    expected: bool,
) {
    assert_eq!(
        descriptor
            .prop(prop)
            .and_then(|schema| schema.default_value.as_ref()),
        Some(&UiValue::Bool(expected)),
        "{} should default `{prop}` to `{expected}`",
        descriptor.id
    );
}

fn assert_enum_prop_default(
    descriptor: &zircon_runtime_interface::ui::component::UiComponentDescriptor,
    prop: &str,
    expected: &str,
) {
    assert_eq!(
        descriptor
            .prop(prop)
            .and_then(|schema| schema.default_value.as_ref()),
        Some(&UiValue::Enum(expected.to_string())),
        "{} should default `{prop}` to `{expected}`",
        descriptor.id
    );
}

fn assert_float_prop_default(
    descriptor: &zircon_runtime_interface::ui::component::UiComponentDescriptor,
    prop: &str,
    expected: f64,
) {
    assert_eq!(
        descriptor
            .prop(prop)
            .and_then(|schema| schema.default_value.as_ref()),
        Some(&UiValue::Float(expected)),
        "{} should default `{prop}` to `{expected}`",
        descriptor.id
    );
}
