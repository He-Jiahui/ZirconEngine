use crate::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime_interface::ui::component::{UiComponentDescriptor, UiValue};

use super::super::assert_has_prop;
use super::assert_enum_options;

pub(super) fn assert_descriptors(registry: &UiComponentDescriptorRegistry) {
    assert_typography(registry);
    assert_divider(registry);
    assert_icon(registry);
    assert_svg_icon(registry);
    assert_avatar(registry);
    assert_badge(registry);
    assert_chip(registry);
    assert_list(registry);
    assert_image_list(registry);
    assert_table(registry);
}

fn assert_typography(registry: &UiComponentDescriptorRegistry) {
    let typography = registry
        .descriptor("Typography")
        .expect("Typography descriptor");
    assert_enum_options(
        typography,
        "variant",
        &[
            "body1",
            "body2",
            "button",
            "caption",
            "h1",
            "h2",
            "h3",
            "h4",
            "h5",
            "h6",
            "inherit",
            "overline",
            "subtitle1",
            "subtitle2",
        ],
    );
    assert_enum_options(
        typography,
        "align",
        &["center", "inherit", "justify", "left", "right"],
    );
    for prop in [
        "color",
        "component",
        "gutterBottom",
        "noWrap",
        "variantMapping",
    ] {
        assert_has_prop(typography, prop);
    }
    assert_default_value(typography, "variant", UiValue::Enum("body1".to_string()));
    assert_default_value(typography, "align", UiValue::Enum("inherit".to_string()));
}

fn assert_divider(registry: &UiComponentDescriptorRegistry) {
    let divider = registry.descriptor("Divider").expect("Divider descriptor");
    assert_enum_options(divider, "orientation", &["horizontal", "vertical"]);
    assert_enum_options(divider, "variant", &["fullWidth", "inset", "middle"]);
    assert_enum_options(divider, "textAlign", &["center", "left", "right"]);
    for prop in ["component", "role", "absolute", "flexItem", "text"] {
        assert_has_prop(divider, prop);
    }
    assert_has_slot(divider, "wrapper");
    assert_default_value(divider, "variant", UiValue::Enum("fullWidth".to_string()));
}

fn assert_avatar(registry: &UiComponentDescriptorRegistry) {
    let avatar = registry.descriptor("Avatar").expect("Avatar descriptor");
    assert_enum_options(avatar, "variant", &["circular", "rounded", "square"]);
    for prop in ["alt", "component", "src", "srcSet", "sizes"] {
        assert_has_prop(avatar, prop);
    }
    for slot in ["img", "fallback"] {
        assert_has_slot(avatar, slot);
    }
    assert_default_value(avatar, "variant", UiValue::Enum("circular".to_string()));
}

fn assert_badge(registry: &UiComponentDescriptorRegistry) {
    let badge = registry.descriptor("Badge").expect("Badge descriptor");
    assert_enum_options(badge, "variant", &["dot", "standard"]);
    assert_enum_options(
        badge,
        "color",
        &[
            "default",
            "primary",
            "secondary",
            "error",
            "info",
            "success",
            "warning",
        ],
    );
    assert_enum_options(badge, "overlap", &["circular", "rectangular"]);
    assert_enum_options(badge, "anchor_origin_vertical", &["top", "bottom"]);
    assert_enum_options(badge, "anchor_origin_horizontal", &["left", "right"]);
    for prop in [
        "badgeContent",
        "max",
        "showZero",
        "invisible",
        "anchorOrigin",
    ] {
        assert_has_prop(badge, prop);
    }
    assert_has_slot(badge, "badge");
    assert_default_value(badge, "max", UiValue::Int(99));
    assert_default_value(badge, "color", UiValue::Enum("default".to_string()));
}

fn assert_chip(registry: &UiComponentDescriptorRegistry) {
    let chip = registry.descriptor("Chip").expect("Chip descriptor");
    assert_enum_options(chip, "variant", &["filled", "outlined"]);
    assert_enum_options(chip, "size", &["small", "medium"]);
    assert_enum_options(
        chip,
        "color",
        &[
            "default",
            "primary",
            "secondary",
            "error",
            "info",
            "success",
            "warning",
        ],
    );
    for prop in [
        "label",
        "component",
        "clickable",
        "deletable",
        "onDelete",
        "deleteIcon",
        "skipFocusWhenDisabled",
    ] {
        assert_has_prop(chip, prop);
    }
    for slot in ["avatar", "icon", "label", "deleteIcon"] {
        assert_has_slot(chip, slot);
    }
    assert_default_value(chip, "variant", UiValue::Enum("filled".to_string()));
}

fn assert_icon(registry: &UiComponentDescriptorRegistry) {
    let icon = registry.descriptor("Icon").expect("Icon descriptor");
    assert_enum_options(
        icon,
        "color",
        &[
            "inherit",
            "action",
            "disabled",
            "primary",
            "secondary",
            "error",
            "info",
            "success",
            "warning",
        ],
    );
    assert_enum_options(icon, "fontSize", &["inherit", "small", "medium", "large"]);
    for prop in ["text", "icon", "baseClassName", "component"] {
        assert_has_prop(icon, prop);
    }
    assert_default_value(icon, "color", UiValue::Enum("inherit".to_string()));
    assert_default_value(icon, "fontSize", UiValue::Enum("medium".to_string()));
}

fn assert_svg_icon(registry: &UiComponentDescriptorRegistry) {
    let icon = registry.descriptor("SvgIcon").expect("SvgIcon descriptor");
    assert_enum_options(
        icon,
        "color",
        &[
            "inherit",
            "action",
            "disabled",
            "primary",
            "secondary",
            "error",
            "info",
            "success",
            "warning",
        ],
    );
    assert_enum_options(icon, "fontSize", &["inherit", "small", "medium", "large"]);
    for prop in [
        "text",
        "icon",
        "component",
        "htmlColor",
        "viewBox",
        "titleAccess",
        "inheritViewBox",
    ] {
        assert_has_prop(icon, prop);
    }
    assert_default_value(icon, "color", UiValue::Enum("inherit".to_string()));
    assert_default_value(icon, "fontSize", UiValue::Enum("medium".to_string()));
    assert_default_value(icon, "viewBox", UiValue::String("0 0 24 24".to_string()));
}

fn assert_list(registry: &UiComponentDescriptorRegistry) {
    let list = registry.descriptor("List").expect("List descriptor");
    for prop in ["component", "dense", "disablePadding", "subheader"] {
        assert_has_prop(list, prop);
    }
    assert_has_slot(list, "subheader");
}

fn assert_image_list(registry: &UiComponentDescriptorRegistry) {
    let image_list = registry
        .descriptor("ImageList")
        .expect("ImageList descriptor");
    assert_enum_options(
        image_list,
        "variant",
        &["masonry", "quilted", "standard", "woven"],
    );
    for prop in ["cols", "component", "gap", "rowHeight"] {
        assert_has_prop(image_list, prop);
    }
    assert_default_value(image_list, "cols", UiValue::Int(2));
    assert_default_value(image_list, "gap", UiValue::Float(4.0));
}

fn assert_table(registry: &UiComponentDescriptorRegistry) {
    let table = registry.descriptor("Table").expect("Table descriptor");
    assert_enum_options(table, "padding", &["checkbox", "none", "normal"]);
    assert_enum_options(table, "size", &["medium", "small"]);
    for prop in ["component", "stickyHeader"] {
        assert_has_prop(table, prop);
    }
    assert_default_value(table, "size", UiValue::Enum("medium".to_string()));
}

fn assert_has_slot(descriptor: &UiComponentDescriptor, slot_name: &str) {
    assert!(
        descriptor
            .slot_schema
            .iter()
            .any(|slot| slot.name == slot_name),
        "{} missing MUI slot `{slot_name}`",
        descriptor.id
    );
}

fn assert_default_value(descriptor: &UiComponentDescriptor, prop_name: &str, expected: UiValue) {
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
