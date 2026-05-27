use crate::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime_interface::ui::component::{
    UiComponentDescriptor, UiComponentDescriptorKind, UiComponentEventKind, UiValue,
};

use super::super::{assert_has_event, assert_has_prop};
use super::assert_enum_options;

pub(super) fn assert_descriptors(registry: &UiComponentDescriptorRegistry) {
    assert_mui_layout(registry);
    assert_foundation_layout(registry);
    assert_layout_utilities(registry);
    assert_transitions(registry);
    assert_editor_layout(registry);
}

fn assert_mui_layout(registry: &UiComponentDescriptorRegistry) {
    let box_descriptor = registry.descriptor("Box").expect("Box descriptor");
    assert_has_prop(box_descriptor, "component");
    assert_has_slot(box_descriptor, "content");
    assert_default_value(
        box_descriptor,
        "component",
        UiValue::String("div".to_string()),
    );

    let container = registry
        .descriptor("Container")
        .expect("Container descriptor");
    for prop in [
        "component",
        "disableGutters",
        "fixed",
        "maxWidth",
        "max_width",
    ] {
        assert_has_prop(container, prop);
    }
    assert_enum_options(
        container,
        "maxWidth",
        &["false", "xs", "sm", "md", "lg", "xl"],
    );
    assert_default_value(container, "maxWidth", UiValue::Enum("lg".to_string()));

    let grid = registry.descriptor("Grid").expect("Grid descriptor");
    for prop in [
        "component",
        "columns",
        "columnSpacing",
        "container",
        "direction",
        "offset",
        "rowSpacing",
        "size",
        "spacing",
        "wrap",
        "unstable_level",
    ] {
        assert_has_prop(grid, prop);
    }
    assert_enum_options(grid, "direction", &["row", "row-reverse"]);
    assert_enum_options(grid, "wrap", &["nowrap", "wrap", "wrap-reverse"]);

    let stack = registry.descriptor("Stack").expect("Stack descriptor");
    for prop in ["component", "direction", "divider", "spacing", "useFlexGap"] {
        assert_has_prop(stack, prop);
    }
    assert_enum_options(
        stack,
        "direction",
        &["column-reverse", "column", "row-reverse", "row"],
    );

    let masonry = registry.descriptor("Masonry").expect("Masonry descriptor");
    for prop in [
        "component",
        "columns",
        "spacing",
        "sequential",
        "defaultColumns",
        "defaultHeight",
        "defaultSpacing",
        "needs_support",
    ] {
        assert_has_prop(masonry, prop);
    }
    assert_default_value(masonry, "columns", UiValue::Int(4));
}

fn assert_foundation_layout(registry: &UiComponentDescriptorRegistry) {
    let scrollbar = registry
        .descriptor("Scrollbar")
        .expect("Scrollbar descriptor");
    assert_has_prop(scrollbar, "value");
    assert_has_event(scrollbar, UiComponentEventKind::ValueChanged);

    for component_id in [
        "Splitter",
        "Slot",
        "Composite",
        "FlexGroup",
        "HorizontalGroup",
        "VerticalGroup",
        "GridGroup",
        "Overlay",
        "ScrollView",
    ] {
        registry
            .descriptor(component_id)
            .unwrap_or_else(|| panic!("missing layout descriptor `{component_id}`"));
    }
}

fn assert_layout_utilities(registry: &UiComponentDescriptorRegistry) {
    let click_away = registry
        .descriptor("ClickAwayListener")
        .expect("ClickAwayListener descriptor");
    for prop in [
        "disableReactTree",
        "mouseEvent",
        "touchEvent",
        "behavior_utility",
    ] {
        assert_has_prop(click_away, prop);
    }
    assert_has_slot(click_away, "content");
    assert_has_event(click_away, UiComponentEventKind::ClosePopup);

    let portal = registry.descriptor("Portal").expect("Portal descriptor");
    for prop in [
        "container",
        "container_id",
        "disablePortal",
        "disable_portal",
    ] {
        assert_has_prop(portal, prop);
    }
    assert_has_slot(portal, "content");

    let no_ssr = registry.descriptor("NoSsr").expect("NoSsr descriptor");
    for prop in ["defer", "fallback"] {
        assert_has_prop(no_ssr, prop);
    }
    assert_has_slot(no_ssr, "fallback");

    let css_baseline = registry
        .descriptor("CssBaseline")
        .expect("CssBaseline descriptor");
    assert_has_prop(css_baseline, "enableColorScheme");

    let init_script = registry
        .descriptor("InitColorSchemeScript")
        .expect("InitColorSchemeScript descriptor");
    for prop in [
        "defaultMode",
        "defaultLightColorScheme",
        "defaultDarkColorScheme",
        "colorSchemeNode",
        "modeStorageKey",
        "colorSchemeStorageKey",
        "attribute",
        "nonce",
    ] {
        assert_has_prop(init_script, prop);
    }
    assert_enum_options(init_script, "defaultMode", &["dark", "light", "system"]);
    assert_default_value(
        init_script,
        "attribute",
        UiValue::String("data-mui-color-scheme".to_string()),
    );

    let media_query = registry
        .descriptor("UseMediaQuery")
        .expect("UseMediaQuery descriptor");
    for prop in [
        "query",
        "defaultMatches",
        "default_matches",
        "matchMedia",
        "noSsr",
        "no_ssr",
        "ssrMatchMedia",
        "matches",
        "up",
        "down",
        "between",
        "breakpoint",
    ] {
        assert_has_prop(media_query, prop);
    }
}

fn assert_transitions(registry: &UiComponentDescriptorRegistry) {
    for component_id in ["Collapse", "Fade", "Grow", "Slide", "Zoom"] {
        let descriptor = registry
            .descriptor(component_id)
            .unwrap_or_else(|| panic!("missing transition descriptor `{component_id}`"));
        for prop in [
            "transition_kind",
            "in",
            "transition_status",
            "transition_progress",
            "timeout_ms",
            "transition_duration_ms",
            "timeout",
            "easing",
            "transition_easing",
            "appear",
            "mount_on_enter",
            "mountOnEnter",
            "unmount_on_exit",
            "unmountOnExit",
            "addEndListener",
            "style",
        ] {
            assert_has_prop(descriptor, prop);
        }
    }

    let collapse = registry
        .descriptor("Collapse")
        .expect("Collapse descriptor");
    assert_enum_options(collapse, "orientation", &["horizontal", "vertical"]);
    for slot in ["content", "wrapper", "wrapperInner"] {
        assert_has_slot(collapse, slot);
    }
    assert_has_prop(collapse, "collapsedSize");

    let slide = registry.descriptor("Slide").expect("Slide descriptor");
    assert_enum_options(slide, "direction", &["down", "left", "right", "up"]);
    assert_has_prop(slide, "container");
}

fn assert_editor_layout(registry: &UiComponentDescriptorRegistry) {
    let viewport_host = registry
        .descriptor("ViewportHost")
        .expect("ViewportHost descriptor");
    assert_eq!(
        viewport_host.descriptor_kind,
        UiComponentDescriptorKind::Layout
    );
    assert_has_event(viewport_host, UiComponentEventKind::SetWorldSurface);

    for component_id in [
        "GraphCanvas",
        "Timeline",
        "VisualDesigner",
        "View",
        "Window",
        "WindowFrame",
        "DocumentNode",
        "FloatingWindow",
        "DockHost",
        "WorkbenchShell",
    ] {
        registry
            .descriptor(component_id)
            .unwrap_or_else(|| panic!("missing editor layout descriptor `{component_id}`"));
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
