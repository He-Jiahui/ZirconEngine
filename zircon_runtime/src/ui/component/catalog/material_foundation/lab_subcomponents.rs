use super::shared::*;

const TAB_ORIENTATIONS: [&str; 2] = ["horizontal", "vertical"];
const TAB_VARIANTS: [&str; 3] = ["fullWidth", "scrollable", "standard"];
const TIMELINE_POSITIONS: [&str; 4] = ["alternate-reverse", "alternate", "left", "right"];

pub(super) fn descriptors() -> Vec<UiComponentDescriptor> {
    vec![
        tab_context(),
        tab_list(),
        tab_panel(),
        timeline_connector(),
        timeline_content(),
        timeline_dot(),
        timeline_item(),
        timeline_opposite_content(),
        timeline_separator(),
        tree_item(),
    ]
}

fn tab_context() -> UiComponentDescriptor {
    composite(
        "TabContext",
        "Tab Context",
        UiComponentCategory::Container,
        "tab-context",
    )
    .with_prop(required_string_prop("value"))
    .with_prop(value_text_prop())
    .with_prop(default_string_prop("idPrefix", ""))
    .slot(multi_slot("content"))
    .event(UiComponentEventKind::ValueChanged)
}

fn tab_list() -> UiComponentDescriptor {
    add_props(
        composite(
            "TabList",
            "Tab List",
            UiComponentCategory::Container,
            "tab-list",
        ),
        [
            value_text_prop(),
            default_string_prop("value", ""),
            default_string_prop("component", "div"),
            bool_prop("centered", false),
            bool_prop("allowScrollButtonsMobile", false),
            mui_enum_prop("indicatorColor", "primary", ["primary", "secondary"]),
            mui_enum_prop("orientation", "horizontal", TAB_ORIENTATIONS),
            mui_enum_prop("scrollButtons", "auto", ["auto", "false", "true"]),
            bool_prop("selectionFollowsFocus", false),
            mui_enum_prop("textColor", "primary", ["inherit", "primary", "secondary"]),
            mui_enum_prop("variant", "standard", TAB_VARIANTS),
            bool_prop("visibleScrollbar", false),
        ],
    )
    .slot(multi_slot("tabs"))
    .event(UiComponentEventKind::ValueChanged)
}

fn tab_panel() -> UiComponentDescriptor {
    add_props(
        composite(
            "TabPanel",
            "Tab Panel",
            UiComponentCategory::Container,
            "tab-panel",
        ),
        [
            required_string_prop("value"),
            default_string_prop("context_value", ""),
            default_string_prop("selectedValue", ""),
            default_string_prop("component", "div"),
            bool_prop("keepMounted", false),
            bool_prop("hidden", false),
        ],
    )
    .slot(multi_slot("content"))
}

fn timeline_connector() -> UiComponentDescriptor {
    primitive(
        "TimelineConnector",
        "Timeline Connector",
        UiComponentCategory::Visual,
        "timeline-connector",
    )
    .with_prop(default_string_prop("component", "span"))
}

fn timeline_content() -> UiComponentDescriptor {
    timeline_content_component(
        "TimelineContent",
        "Timeline Content",
        "timeline-content",
        "right",
    )
}

fn timeline_opposite_content() -> UiComponentDescriptor {
    timeline_content_component(
        "TimelineOppositeContent",
        "Timeline Opposite Content",
        "timeline-opposite-content",
        "left",
    )
}

fn timeline_content_component(
    id: &str,
    display_name: &str,
    role: &str,
    default_position: &str,
) -> UiComponentDescriptor {
    composite(id, display_name, UiComponentCategory::Container, role)
        .with_prop(default_string_prop("component", "div"))
        .with_prop(timeline_position_prop(default_position))
        .slot(multi_slot("content"))
}

fn timeline_dot() -> UiComponentDescriptor {
    primitive(
        "TimelineDot",
        "Timeline Dot",
        UiComponentCategory::Visual,
        "timeline-dot",
    )
    .with_prop(default_string_prop("component", "span"))
    .with_prop(mui_enum_prop("variant", "filled", ["filled", "outlined"]))
    .with_prop(mui_enum_prop(
        "color",
        "grey",
        [
            "error",
            "grey",
            "info",
            "inherit",
            "primary",
            "secondary",
            "success",
            "warning",
        ],
    ))
    .with_prop(icon_prop())
    .slot(UiSlotSchema::new("icon"))
    .slot(multi_slot("content"))
    .requires_render_capability(UiRenderCapability::Vector)
}

fn timeline_item() -> UiComponentDescriptor {
    composite(
        "TimelineItem",
        "Timeline Item",
        UiComponentCategory::Container,
        "timeline-item",
    )
    .with_prop(default_string_prop("component", "li"))
    .with_prop(timeline_position_prop("right"))
    .with_prop(bool_prop("hasOppositeContent", false))
    .slot(UiSlotSchema::new("oppositeContent"))
    .slot(UiSlotSchema::new("separator"))
    .slot(UiSlotSchema::new("content"))
}

fn timeline_separator() -> UiComponentDescriptor {
    composite(
        "TimelineSeparator",
        "Timeline Separator",
        UiComponentCategory::Container,
        "timeline-separator",
    )
    .with_prop(default_string_prop("component", "div"))
    .slot(UiSlotSchema::new("dot"))
    .slot(UiSlotSchema::new("connector"))
    .slot(multi_slot("content"))
}

fn tree_item() -> UiComponentDescriptor {
    composite(
        "TreeItem",
        "Tree Item",
        UiComponentCategory::Input,
        "tree-item",
    )
    .with_prop(default_string_prop("itemId", ""))
    .with_prop(default_string_prop("nodeId", ""))
    .with_prop(default_string_prop("label", ""))
    .with_prop(bool_prop("disabled", false))
    .with_prop(bool_prop("expanded", false))
    .with_prop(bool_prop("selected", false))
    .slot(UiSlotSchema::new("icon"))
    .slot(UiSlotSchema::new("label"))
    .slot(UiSlotSchema::new("checkbox"))
    .slot(UiSlotSchema::new("content"))
    .slot(multi_slot("children"))
    .events([
        UiComponentEventKind::Focus,
        UiComponentEventKind::SelectOption,
        UiComponentEventKind::ToggleExpanded,
    ])
}

fn timeline_position_prop(default: &str) -> UiPropSchema {
    mui_enum_prop("position", default, TIMELINE_POSITIONS)
}

fn add_props<const N: usize>(
    mut descriptor: UiComponentDescriptor,
    props: [UiPropSchema; N],
) -> UiComponentDescriptor {
    for prop in props {
        descriptor = descriptor.with_prop(prop);
    }
    descriptor
}

fn multi_slot(name: &str) -> UiSlotSchema {
    UiSlotSchema::new(name).multiple(true)
}

fn mui_enum_prop<const N: usize>(
    name: &str,
    default: &str,
    options: [&'static str; N],
) -> UiPropSchema {
    enum_prop_with_options(
        name,
        default,
        options.into_iter().map(enum_option_descriptor),
    )
}
