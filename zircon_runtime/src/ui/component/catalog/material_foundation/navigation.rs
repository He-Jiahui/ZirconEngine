use super::shared::*;
use zircon_runtime_interface::ui::component::UiPropSchema;

const ORIENTATIONS: [&str; 2] = ["horizontal", "vertical"];

pub(super) fn descriptors() -> Vec<UiComponentDescriptor> {
    vec![
        breadcrumbs(),
        bottom_navigation(),
        drawer(),
        link(),
        menu(),
        menubar(),
        pagination(),
        stepper(),
        tabs(),
    ]
}

fn breadcrumbs() -> UiComponentDescriptor {
    add_slots(
        add_props(
            composite(
                "Breadcrumbs",
                "Breadcrumbs",
                UiComponentCategory::Container,
                "breadcrumbs",
            ),
            [
                default_string_prop("component", "nav"),
                default_string_prop("separator", "/"),
                default_string_prop("expandText", "Show path"),
                int_prop("itemsBeforeCollapse", 1),
                int_prop("itemsAfterCollapse", 1),
                int_prop("maxItems", 8),
            ],
        ),
        ["ol", "collapsedIcon"],
    )
    .slot(multi_slot("li"))
    .slot(multi_slot("separator"))
    .slot(multi_slot("items"))
    .event(UiComponentEventKind::SelectOption)
}

fn bottom_navigation() -> UiComponentDescriptor {
    add_props(
        composite(
            "BottomNavigation",
            "Bottom Navigation",
            UiComponentCategory::Container,
            "bottom-navigation",
        ),
        [
            value_text_prop(),
            default_string_prop("component", "div"),
            bool_prop("showLabels", false),
        ],
    )
    .slot(multi_slot("actions"))
    .event(UiComponentEventKind::SelectOption)
}

fn drawer() -> UiComponentDescriptor {
    overlay_layer_props(shell("Drawer", "Drawer", "drawer"))
        .with_prop(enum_prop("slot", "left_top"))
        .with_prop(enum_prop("mode", "pinned"))
        .with_prop(string_prop("active_view"))
        .slot(multi_slot("tabs"))
        .slot(multi_slot("content"))
        .event(UiComponentEventKind::SelectOption)
}

fn link() -> UiComponentDescriptor {
    add_props(
        primitive("Link", "Link", UiComponentCategory::Input, "link"),
        [
            text_prop(),
            string_prop("href"),
            default_string_prop("component", "a"),
            default_string_prop("color", "primary"),
            mui_enum_prop("underline", "always", ["always", "hover", "none"]),
            default_string_prop("variant", "inherit"),
            map_prop("TypographyClasses"),
        ],
    )
    .event(UiComponentEventKind::Commit)
}

fn menu() -> UiComponentDescriptor {
    add_slots(
        overlay_layer_props(modal_interaction_props(popup_position_props(
            add_props(
                composite("Menu", "Menu", UiComponentCategory::Input, "menu")
                    .with_prop(bool_prop("open", false)),
                [
                    bool_prop("autoFocus", true),
                    bool_prop("disableAutoFocusItem", false),
                    default_string_prop("transitionDuration", "auto"),
                    mui_enum_prop("variant", "selectedMenu", ["menu", "selectedMenu"]),
                ],
            ),
            "bottom-start",
        ))),
        ["paper", "list", "transition"],
    )
    .slot(multi_slot("items"))
    .event(UiComponentEventKind::Commit)
}

fn menubar() -> UiComponentDescriptor {
    composite("Menubar", "Menubar", UiComponentCategory::Input, "menubar")
        .slot(multi_slot("items"))
        .events([
            UiComponentEventKind::OpenPopup,
            UiComponentEventKind::SelectOption,
            UiComponentEventKind::ClosePopup,
        ])
}

fn pagination() -> UiComponentDescriptor {
    add_slots(
        add_props(
            composite(
                "Pagination",
                "Pagination",
                UiComponentCategory::Input,
                "pagination",
            ),
            [
                int_prop("page", 1),
                int_prop("page_count", 10),
                int_prop("count", 1),
                int_prop("defaultPage", 1),
                int_prop("boundaryCount", 1),
                int_prop("siblingCount", 1),
                bool_prop("disabled", false),
                bool_prop("hideNextButton", false),
                bool_prop("hidePrevButton", false),
                bool_prop("showFirstButton", false),
                bool_prop("showLastButton", false),
                mui_enum_prop("color", "standard", ["primary", "secondary", "standard"]),
                mui_enum_prop("shape", "circular", ["circular", "rounded"]),
                mui_enum_prop("size", "medium", ["small", "medium", "large"]),
                mui_enum_prop("variant", "text", ["outlined", "text"]),
            ],
        ),
        ["ul"],
    )
    .slot(multi_slot("items"))
    .event(UiComponentEventKind::SetPage)
}

fn stepper() -> UiComponentDescriptor {
    add_slots(
        add_props(
            composite(
                "Stepper",
                "Stepper",
                UiComponentCategory::Container,
                "stepper",
            ),
            [
                int_prop("active_step", 0),
                int_prop("activeStep", 0),
                bool_prop("alternativeLabel", false),
                default_string_prop("component", "ol"),
                bool_prop("nonLinear", false),
                mui_enum_prop("orientation", "horizontal", ORIENTATIONS),
            ],
        ),
        ["connector"],
    )
    .slot(multi_slot("steps"))
    .event(UiComponentEventKind::SelectOption)
}

fn tabs() -> UiComponentDescriptor {
    add_slots(
        add_props(
            composite("Tabs", "Tabs", UiComponentCategory::Container, "tabs"),
            [
                value_text_prop(),
                default_string_prop("component", "div"),
                bool_prop("centered", false),
                bool_prop("allowScrollButtonsMobile", false),
                mui_enum_prop("indicatorColor", "primary", ["primary", "secondary"]),
                mui_enum_prop("orientation", "horizontal", ORIENTATIONS),
                mui_enum_prop("scrollButtons", "auto", ["auto", "false", "true"]),
                bool_prop("selectionFollowsFocus", false),
                mui_enum_prop("textColor", "primary", ["inherit", "primary", "secondary"]),
                mui_enum_prop(
                    "variant",
                    "standard",
                    ["fullWidth", "scrollable", "standard"],
                ),
                bool_prop("visibleScrollbar", false),
            ],
        ),
        [
            "list",
            "scroller",
            "indicator",
            "scrollButtons",
            "startScrollButtonIcon",
            "endScrollButtonIcon",
        ],
    )
    .slot(multi_slot("tabs"))
    .slot(multi_slot("panels"))
    .event(UiComponentEventKind::ValueChanged)
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

fn add_slots<const N: usize>(
    mut descriptor: UiComponentDescriptor,
    names: [&str; N],
) -> UiComponentDescriptor {
    for name in names {
        descriptor = descriptor.slot(UiSlotSchema::new(name));
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
