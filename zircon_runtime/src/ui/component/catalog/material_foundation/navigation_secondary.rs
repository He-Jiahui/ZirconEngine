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
            options_prop(),
            int_prop("focused_index", 0),
            array_prop("disabled_options"),
            bool_prop("keyboard_navigation", true),
            default_string_prop("typeahead_buffer", ""),
            bool_prop("typeahead_buffer_expired", false),
            int_prop("typeahead_timeout_ms", 500),
            bool_prop("allow_search", true),
            int_prop("search_bar_enabled_on_item_count", 0),
            default_string_prop("search_query", ""),
            array_prop("filtered_option_ids"),
            bool_prop("filter_no_results", false),
            default_string_prop("hovered_option_id", ""),
            default_string_prop("submenu_pending_option_id", ""),
            default_string_prop("submenu_open_option_id", ""),
            bool_prop("submenu_hover_ready", false),
            int_prop("submenu_hover_delay_ms", 300),
            default_string_prop("submenu_focus_scope", "root"),
            int_prop("submenu_active_parent_index", -1),
            bool_prop("submenu_focus_loop", true),
            bool_prop("autoFocus", false),
            bool_prop("autoFocusItem", false),
            bool_prop("disabledItemsFocusable", false),
            bool_prop("disableListWrap", false),
            mui_enum_prop("variant", "selectedMenu", ["menu", "selectedMenu"]),
        ],
    )
    .slot(UiSlotSchema::new("items").multiple(true))
    .events([
        UiComponentEventKind::KeyboardAction,
        UiComponentEventKind::KeyboardText,
        UiComponentEventKind::TypeaheadExpired,
        UiComponentEventKind::ValueChanged,
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
