use crate::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime_interface::ui::component::{UiComponentEventKind, UiRenderCapability, UiValue};

use super::super::{assert_has_event, assert_has_prop};
use super::assert_enum_options;

pub(super) fn assert_descriptors(registry: &UiComponentDescriptorRegistry) {
    assert_tab_context_family(registry);
    assert_timeline_family(registry);
    assert_tree_item(registry);
}

fn assert_tab_context_family(registry: &UiComponentDescriptorRegistry) {
    let context = descriptor(registry, "TabContext");
    for prop in ["value", "value_text", "idPrefix"] {
        assert_has_prop(context, prop);
    }
    assert_has_slot(context, "content");
    assert_has_event(context, UiComponentEventKind::ValueChanged);

    let list = descriptor(registry, "TabList");
    assert_enum_options(list, "orientation", &["horizontal", "vertical"]);
    assert_enum_options(list, "scrollButtons", &["auto", "false", "true"]);
    assert_enum_options(list, "textColor", &["inherit", "primary", "secondary"]);
    assert_enum_options(list, "variant", &["fullWidth", "scrollable", "standard"]);
    for prop in [
        "value_text",
        "value",
        "component",
        "centered",
        "allowScrollButtonsMobile",
        "indicatorColor",
        "selectionFollowsFocus",
        "visibleScrollbar",
    ] {
        assert_has_prop(list, prop);
    }
    assert_has_slot(list, "tabs");
    assert_has_event(list, UiComponentEventKind::ValueChanged);

    let panel = descriptor(registry, "TabPanel");
    for prop in [
        "value",
        "context_value",
        "selectedValue",
        "component",
        "keepMounted",
        "hidden",
    ] {
        assert_has_prop(panel, prop);
    }
    assert_has_slot(panel, "content");
    assert_default_value(panel, "component", UiValue::String("div".to_string()));
    assert_default_value(panel, "keepMounted", UiValue::Bool(false));
}

fn assert_timeline_family(registry: &UiComponentDescriptorRegistry) {
    let timeline = descriptor(registry, "Timeline");
    assert_enum_options(
        timeline,
        "position",
        &["alternate-reverse", "alternate", "left", "right"],
    );
    for prop in ["component", "time", "duration"] {
        assert_has_prop(timeline, prop);
    }
    for slot in ["items", "content", "separator", "connector", "dot"] {
        assert_has_slot(timeline, slot);
    }

    for id in [
        "TimelineConnector",
        "TimelineContent",
        "TimelineDot",
        "TimelineItem",
        "TimelineOppositeContent",
        "TimelineSeparator",
    ] {
        descriptor(registry, id);
    }

    let dot = descriptor(registry, "TimelineDot");
    assert_enum_options(dot, "variant", &["filled", "outlined"]);
    assert_enum_options(
        dot,
        "color",
        &[
            "error",
            "grey",
            "info",
            "inherit",
            "primary",
            "secondary",
            "success",
            "warning",
        ],
    );
    assert_has_slot(dot, "icon");
    assert!(dot
        .required_render_capabilities
        .contains(&UiRenderCapability::Vector));

    let item = descriptor(registry, "TimelineItem");
    assert_has_prop(item, "hasOppositeContent");
    for slot in ["oppositeContent", "separator", "content"] {
        assert_has_slot(item, slot);
    }

    for id in ["TimelineContent", "TimelineItem", "TimelineOppositeContent"] {
        assert_enum_options(
            descriptor(registry, id),
            "position",
            &["alternate-reverse", "alternate", "left", "right"],
        );
    }
    for slot in ["dot", "connector", "content"] {
        assert_has_slot(descriptor(registry, "TimelineSeparator"), slot);
    }
}

fn assert_tree_item(registry: &UiComponentDescriptorRegistry) {
    let tree_item = descriptor(registry, "TreeItem");
    for prop in [
        "itemId", "nodeId", "label", "disabled", "expanded", "selected",
    ] {
        assert_has_prop(tree_item, prop);
    }
    for slot in ["icon", "label", "checkbox", "content", "children"] {
        assert_has_slot(tree_item, slot);
    }
    assert_has_event(tree_item, UiComponentEventKind::SelectOption);
    assert_has_event(tree_item, UiComponentEventKind::ToggleExpanded);
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
