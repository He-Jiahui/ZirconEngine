use crate::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime_interface::ui::component::{
    UiComponentDescriptor, UiComponentEventKind, UiValue,
};

use super::super::{assert_has_event, assert_has_prop};
use super::assert_enum_options;

pub(super) fn assert_descriptors(registry: &UiComponentDescriptorRegistry) {
    assert_breadcrumbs(registry);
    assert_bottom_navigation(registry);
    assert_link(registry);
    assert_menu(registry);
    assert_pagination(registry);
    assert_stepper_family(registry);
    assert_tabs_family(registry);
}

fn assert_breadcrumbs(registry: &UiComponentDescriptorRegistry) {
    let breadcrumbs = registry
        .descriptor("Breadcrumbs")
        .expect("Breadcrumbs descriptor");
    for prop in [
        "component",
        "separator",
        "expandText",
        "itemsBeforeCollapse",
        "itemsAfterCollapse",
        "maxItems",
    ] {
        assert_has_prop(breadcrumbs, prop);
    }
    for slot in ["ol", "li", "separator", "collapsedIcon", "items"] {
        assert_has_slot(breadcrumbs, slot);
    }
    assert_default_value(breadcrumbs, "component", UiValue::String("nav".to_string()));
    assert_default_value(breadcrumbs, "maxItems", UiValue::Int(8));
}

fn assert_bottom_navigation(registry: &UiComponentDescriptorRegistry) {
    let bottom_navigation = registry
        .descriptor("BottomNavigation")
        .expect("BottomNavigation descriptor");
    for prop in ["value_text", "component", "showLabels"] {
        assert_has_prop(bottom_navigation, prop);
    }
    assert_has_slot(bottom_navigation, "actions");
    assert_has_event(bottom_navigation, UiComponentEventKind::SelectOption);

    let action = registry
        .descriptor("BottomNavigationAction")
        .expect("BottomNavigationAction descriptor");
    for prop in ["component", "label", "icon", "value_text", "showLabel"] {
        assert_has_prop(action, prop);
    }
    for slot in ["icon", "label"] {
        assert_has_slot(action, slot);
    }
    assert_default_value(action, "showLabel", UiValue::Bool(false));
}

fn assert_link(registry: &UiComponentDescriptorRegistry) {
    let link = registry.descriptor("Link").expect("Link descriptor");
    assert_enum_options(link, "underline", &["always", "hover", "none"]);
    for prop in [
        "text",
        "href",
        "component",
        "color",
        "variant",
        "TypographyClasses",
    ] {
        assert_has_prop(link, prop);
    }
    assert_default_value(link, "component", UiValue::String("a".to_string()));
    assert_default_value(link, "underline", UiValue::Enum("always".to_string()));
}

fn assert_menu(registry: &UiComponentDescriptorRegistry) {
    let menu = registry.descriptor("Menu").expect("Menu descriptor");
    assert_enum_options(menu, "variant", &["menu", "selectedMenu"]);
    for prop in [
        "open",
        "autoFocus",
        "disableAutoFocusItem",
        "transitionDuration",
    ] {
        assert_has_prop(menu, prop);
    }
    for slot in ["paper", "list", "transition", "items"] {
        assert_has_slot(menu, slot);
    }
    assert_default_value(menu, "variant", UiValue::Enum("selectedMenu".to_string()));

    let item = registry
        .descriptor("MenuItem")
        .expect("MenuItem descriptor");
    for prop in [
        "text",
        "value_text",
        "autoFocus",
        "component",
        "dense",
        "divider",
        "disableGutters",
        "role",
    ] {
        assert_has_prop(item, prop);
    }
    for slot in ["icon", "text"] {
        assert_has_slot(item, slot);
    }
    assert_has_event(item, UiComponentEventKind::SelectOption);
}

fn assert_pagination(registry: &UiComponentDescriptorRegistry) {
    let pagination = registry
        .descriptor("Pagination")
        .expect("Pagination descriptor");
    assert_enum_options(pagination, "color", &["primary", "secondary", "standard"]);
    assert_enum_options(pagination, "shape", &["circular", "rounded"]);
    assert_enum_options(pagination, "size", &["small", "medium", "large"]);
    assert_enum_options(pagination, "variant", &["outlined", "text"]);
    for prop in [
        "page",
        "page_count",
        "count",
        "defaultPage",
        "boundaryCount",
        "siblingCount",
        "disabled",
        "hideNextButton",
        "hidePrevButton",
        "showFirstButton",
        "showLastButton",
    ] {
        assert_has_prop(pagination, prop);
    }
    for slot in ["ul", "items"] {
        assert_has_slot(pagination, slot);
    }
    assert_default_value(pagination, "variant", UiValue::Enum("text".to_string()));

    let item = registry
        .descriptor("PaginationItem")
        .expect("PaginationItem descriptor");
    assert_enum_options(
        item,
        "type",
        &[
            "end-ellipsis",
            "first",
            "last",
            "next",
            "page",
            "previous",
            "start-ellipsis",
        ],
    );
    for prop in ["page", "color", "shape", "size", "variant"] {
        assert_has_prop(item, prop);
    }
    assert_has_slot(item, "icon");
}

fn assert_stepper_family(registry: &UiComponentDescriptorRegistry) {
    let stepper = registry.descriptor("Stepper").expect("Stepper descriptor");
    assert_enum_options(stepper, "orientation", &["horizontal", "vertical"]);
    for prop in [
        "active_step",
        "activeStep",
        "alternativeLabel",
        "component",
        "nonLinear",
    ] {
        assert_has_prop(stepper, prop);
    }
    for slot in ["connector", "steps"] {
        assert_has_slot(stepper, slot);
    }

    for id in [
        "Step",
        "StepButton",
        "StepConnector",
        "StepContent",
        "StepIcon",
        "StepLabel",
    ] {
        registry
            .descriptor(id)
            .unwrap_or_else(|| panic!("missing MUI navigation descriptor `{id}`"));
    }
    assert_enum_options(
        registry.descriptor("Step").expect("Step descriptor"),
        "orientation",
        &["horizontal", "vertical"],
    );
    assert_has_slot(registry.descriptor("StepLabel").unwrap(), "labelContainer");
    assert_has_slot(registry.descriptor("StepConnector").unwrap(), "line");
}

fn assert_tabs_family(registry: &UiComponentDescriptorRegistry) {
    let tabs = registry.descriptor("Tabs").expect("Tabs descriptor");
    assert_enum_options(tabs, "orientation", &["horizontal", "vertical"]);
    assert_enum_options(tabs, "scrollButtons", &["auto", "false", "true"]);
    assert_enum_options(tabs, "textColor", &["inherit", "primary", "secondary"]);
    assert_enum_options(tabs, "variant", &["fullWidth", "scrollable", "standard"]);
    for prop in [
        "component",
        "centered",
        "allowScrollButtonsMobile",
        "indicatorColor",
        "selectionFollowsFocus",
        "visibleScrollbar",
    ] {
        assert_has_prop(tabs, prop);
    }
    for slot in [
        "list",
        "scroller",
        "indicator",
        "scrollButtons",
        "startScrollButtonIcon",
        "endScrollButtonIcon",
        "tabs",
        "panels",
    ] {
        assert_has_slot(tabs, slot);
    }

    let tab = registry.descriptor("Tab").expect("Tab descriptor");
    assert_enum_options(tab, "iconPosition", &["bottom", "end", "start", "top"]);
    assert_enum_options(tab, "textColor", &["inherit", "primary", "secondary"]);
    for prop in [
        "label",
        "icon",
        "disableFocusRipple",
        "value_text",
        "wrapped",
        "fullWidth",
    ] {
        assert_has_prop(tab, prop);
    }
    for slot in ["icon", "indicator"] {
        assert_has_slot(tab, slot);
    }
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
