use super::shared::*;
use zircon_runtime_interface::ui::component::UiPropSchema;

const MUI_COLORS: [&str; 3] = ["primary", "secondary", "standard"];
const MUI_SIZES: [&str; 3] = ["small", "medium", "large"];
const ORIENTATIONS: [&str; 2] = ["horizontal", "vertical"];

pub(super) fn descriptors() -> Vec<UiComponentDescriptor> {
    vec![
        bottom_navigation_action(),
        menu_item(),
        pagination_item(),
        step(),
        step_button(),
        step_connector(),
        step_content(),
        step_icon(),
        step_label(),
        tab(),
    ]
}

fn bottom_navigation_action() -> UiComponentDescriptor {
    add_slots(
        add_props(
            primitive(
                "BottomNavigationAction",
                "Bottom Navigation Action",
                UiComponentCategory::Input,
                "bottom-navigation-action",
            ),
            [
                default_string_prop("component", "button"),
                default_string_prop("label", ""),
                icon_prop(),
                value_text_prop(),
                bool_prop("showLabel", false),
            ],
        ),
        ["icon", "label"],
    )
    .event(UiComponentEventKind::SelectOption)
}

fn menu_item() -> UiComponentDescriptor {
    add_slots(
        add_props(
            primitive(
                "MenuItem",
                "Menu Item",
                UiComponentCategory::Input,
                "menu-item",
            ),
            [
                text_prop(),
                value_text_prop(),
                bool_prop("autoFocus", false),
                default_string_prop("component", "li"),
                bool_prop("dense", false),
                bool_prop("divider", false),
                bool_prop("disableGutters", false),
                default_string_prop("role", "menuitem"),
            ],
        ),
        ["icon", "text"],
    )
    .events([
        UiComponentEventKind::Commit,
        UiComponentEventKind::SelectOption,
    ])
}

fn pagination_item() -> UiComponentDescriptor {
    add_props(
        primitive(
            "PaginationItem",
            "Pagination Item",
            UiComponentCategory::Input,
            "pagination-item",
        ),
        [
            int_prop("page", 1),
            mui_enum_prop("color", "standard", MUI_COLORS),
            mui_enum_prop("shape", "circular", ["circular", "rounded"]),
            mui_enum_prop("size", "medium", MUI_SIZES),
            mui_enum_prop(
                "type",
                "page",
                [
                    "end-ellipsis",
                    "first",
                    "last",
                    "next",
                    "page",
                    "previous",
                    "start-ellipsis",
                ],
            ),
            mui_enum_prop("variant", "text", ["outlined", "text"]),
        ],
    )
    .slot(UiSlotSchema::new("icon"))
    .event(UiComponentEventKind::SetPage)
}

fn step() -> UiComponentDescriptor {
    add_slots(
        add_props(
            composite("Step", "Step", UiComponentCategory::Container, "step"),
            [
                bool_prop("active", false),
                bool_prop("completed", false),
                default_string_prop("component", "li"),
                bool_prop("disabled", false),
                bool_prop("expanded", false),
                int_prop("index", 0),
                bool_prop("last", false),
                mui_enum_prop("orientation", "horizontal", ORIENTATIONS),
                bool_prop("alternativeLabel", false),
            ],
        ),
        ["connector", "label", "content"],
    )
}

fn step_button() -> UiComponentDescriptor {
    add_slots(
        add_props(
            primitive(
                "StepButton",
                "Step Button",
                UiComponentCategory::Input,
                "step-button",
            ),
            [
                icon_prop(),
                default_string_prop("optional", ""),
                mui_enum_prop("orientation", "horizontal", ORIENTATIONS),
            ],
        ),
        ["label", "touchRipple"],
    )
    .event(UiComponentEventKind::SelectOption)
}

fn step_connector() -> UiComponentDescriptor {
    add_props(
        primitive(
            "StepConnector",
            "Step Connector",
            UiComponentCategory::Visual,
            "step-connector",
        ),
        [
            mui_enum_prop("orientation", "horizontal", ORIENTATIONS),
            bool_prop("alternativeLabel", false),
            bool_prop("active", false),
            bool_prop("completed", false),
        ],
    )
    .slot(UiSlotSchema::new("line"))
}

fn step_content() -> UiComponentDescriptor {
    composite(
        "StepContent",
        "Step Content",
        UiComponentCategory::Container,
        "step-content",
    )
    .with_prop(bool_prop("last", false))
    .with_prop(default_string_prop("transitionDuration", "auto"))
    .slot(UiSlotSchema::new("transition"))
    .slot(UiSlotSchema::new("content").multiple(true))
}

fn step_icon() -> UiComponentDescriptor {
    add_props(
        primitive(
            "StepIcon",
            "Step Icon",
            UiComponentCategory::Visual,
            "step-icon",
        ),
        [
            bool_prop("active", false),
            bool_prop("completed", false),
            bool_prop("error", false),
            icon_prop(),
        ],
    )
    .slot(UiSlotSchema::new("text"))
    .requires_render_capability(UiRenderCapability::Vector)
}

fn step_label() -> UiComponentDescriptor {
    add_slots(
        add_props(
            composite(
                "StepLabel",
                "Step Label",
                UiComponentCategory::Container,
                "step-label",
            ),
            [
                text_prop(),
                bool_prop("active", false),
                bool_prop("alternativeLabel", false),
                bool_prop("completed", false),
                bool_prop("disabled", false),
                bool_prop("error", false),
                icon_prop(),
                default_string_prop("optional", ""),
                mui_enum_prop("orientation", "horizontal", ORIENTATIONS),
            ],
        ),
        ["label", "stepIcon", "iconContainer", "labelContainer"],
    )
}

fn tab() -> UiComponentDescriptor {
    add_props(
        primitive("Tab", "Tab", UiComponentCategory::Input, "tab"),
        [
            default_string_prop("label", ""),
            icon_prop(),
            mui_enum_prop("iconPosition", "top", ["bottom", "end", "start", "top"]),
            bool_prop("disableFocusRipple", false),
            value_text_prop(),
            mui_enum_prop("textColor", "inherit", ["inherit", "primary", "secondary"]),
            bool_prop("wrapped", false),
            bool_prop("fullWidth", false),
        ],
    )
    .slot(UiSlotSchema::new("icon"))
    .slot(UiSlotSchema::new("indicator"))
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
