use super::shared::*;
use zircon_runtime_interface::ui::component::UiPropSchema;

const ORIENTATIONS: [&str; 2] = ["horizontal", "vertical"];

pub(super) fn descriptors() -> Vec<UiComponentDescriptor> {
    vec![menu_list(), mobile_stepper(), tab_scroll_button()]
}

fn menu_list() -> UiComponentDescriptor {
    add_props(
        composite(
            "MenuList",
            "Menu List",
            UiComponentCategory::Input,
            "menu-list",
        ),
        [
            bool_prop("autoFocus", false),
            bool_prop("autoFocusItem", false),
            bool_prop("disabledItemsFocusable", false),
            bool_prop("disableListWrap", false),
            mui_enum_prop("variant", "selectedMenu", ["menu", "selectedMenu"]),
        ],
    )
    .slot(UiSlotSchema::new("items").multiple(true))
    .events([
        UiComponentEventKind::Focus,
        UiComponentEventKind::SelectOption,
        UiComponentEventKind::Commit,
    ])
}

fn mobile_stepper() -> UiComponentDescriptor {
    add_slots(
        add_props(
            composite(
                "MobileStepper",
                "Mobile Stepper",
                UiComponentCategory::Input,
                "mobile-stepper",
            ),
            [
                int_prop("activeStep", 0),
                int_prop("steps", 1),
                mui_enum_prop("position", "bottom", ["bottom", "static", "top"]),
                mui_enum_prop("variant", "dots", ["dots", "progress", "text"]),
            ],
        ),
        [
            "backButton",
            "nextButton",
            "dots",
            "dot",
            "dotActive",
            "progress",
        ],
    )
    .event(UiComponentEventKind::SetPage)
}

fn tab_scroll_button() -> UiComponentDescriptor {
    add_slots(
        add_props(
            primitive(
                "TabScrollButton",
                "Tab Scroll Button",
                UiComponentCategory::Input,
                "tab-scroll-button",
            ),
            [
                mui_enum_prop("direction", "left", ["left", "right"]),
                mui_enum_prop("orientation", "horizontal", ORIENTATIONS),
                bool_prop("disabled", false),
            ],
        ),
        ["startScrollButtonIcon", "endScrollButtonIcon"],
    )
    .event(UiComponentEventKind::SelectOption)
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
