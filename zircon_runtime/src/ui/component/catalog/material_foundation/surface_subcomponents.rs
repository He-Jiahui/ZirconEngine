use super::shared::*;
use zircon_runtime_interface::ui::component::UiPropSchema;

const DRAWER_ANCHORS: [&str; 4] = ["left", "right", "top", "bottom"];
const DRAWER_VARIANTS: [&str; 3] = ["temporary", "persistent", "permanent"];

pub(super) fn descriptors() -> Vec<UiComponentDescriptor> {
    vec![
        accordion_summary(),
        accordion_actions(),
        accordion_details(),
        dialog_actions(),
        dialog_content(),
        dialog_content_text(),
        dialog_title(),
        swipeable_drawer(),
    ]
}

fn accordion_summary() -> UiComponentDescriptor {
    add_slots(
        add_props(
            primitive(
                "AccordionSummary",
                "Accordion Summary",
                UiComponentCategory::Input,
                "accordion-summary",
            ),
            [
                text_prop(),
                default_string_prop("expandIcon", ""),
                bool_prop("expanded", false),
                bool_prop("disabled", false),
                bool_prop("disableGutters", false),
                default_string_prop("focusVisibleClassName", ""),
            ],
        ),
        ["content", "expandIconWrapper", "expandIcon"],
    )
    .events([
        UiComponentEventKind::ToggleExpanded,
        UiComponentEventKind::Focus,
        UiComponentEventKind::Press,
    ])
}

fn accordion_actions() -> UiComponentDescriptor {
    composite(
        "AccordionActions",
        "Accordion Actions",
        UiComponentCategory::Container,
        "accordion-actions",
    )
    .with_prop(bool_prop("disableSpacing", false))
    .slot(UiSlotSchema::new("content").multiple(true))
}

fn accordion_details() -> UiComponentDescriptor {
    composite(
        "AccordionDetails",
        "Accordion Details",
        UiComponentCategory::Container,
        "accordion-details",
    )
    .slot(UiSlotSchema::new("content").multiple(true))
}

fn dialog_actions() -> UiComponentDescriptor {
    composite(
        "DialogActions",
        "Dialog Actions",
        UiComponentCategory::Container,
        "dialog-actions",
    )
    .with_prop(bool_prop("disableSpacing", false))
    .slot(UiSlotSchema::new("content").multiple(true))
    .event(UiComponentEventKind::Commit)
}

fn dialog_content() -> UiComponentDescriptor {
    composite(
        "DialogContent",
        "Dialog Content",
        UiComponentCategory::Container,
        "dialog-content",
    )
    .with_prop(bool_prop("dividers", false))
    .slot(UiSlotSchema::new("content").multiple(true))
}

fn dialog_content_text() -> UiComponentDescriptor {
    add_props(
        primitive(
            "DialogContentText",
            "Dialog Content Text",
            UiComponentCategory::Visual,
            "dialog-content-text",
        ),
        [
            text_prop(),
            default_string_prop("component", "p"),
            default_string_prop("variant", "body1"),
            default_string_prop("color", "textSecondary"),
        ],
    )
}

fn dialog_title() -> UiComponentDescriptor {
    add_props(
        primitive(
            "DialogTitle",
            "Dialog Title",
            UiComponentCategory::Visual,
            "dialog-title",
        ),
        [
            text_prop(),
            default_string_prop("component", "h2"),
            default_string_prop("variant", "h6"),
        ],
    )
}

fn swipeable_drawer() -> UiComponentDescriptor {
    add_slots(
        add_props(
            overlay_layer_props(modal_interaction_props(composite(
                "SwipeableDrawer",
                "Swipeable Drawer",
                UiComponentCategory::Container,
                "swipeable-drawer",
            ))),
            [
                bool_prop("open", false),
                mui_enum_prop("anchor", "left", DRAWER_ANCHORS),
                mui_enum_prop("variant", "temporary", DRAWER_VARIANTS),
                bool_prop("disableBackdropTransition", false),
                bool_prop("disableDiscovery", false),
                bool_prop("disableSwipeToOpen", false),
                bool_prop("allowSwipeInChildren", false),
                bool_prop("hideBackdrop", false),
                float_prop("hysteresis", 0.52),
                float_prop("minFlingVelocity", 450.0),
                float_prop("swipeAreaWidth", 20.0),
            ],
        ),
        [
            "root",
            "backdrop",
            "docked",
            "paper",
            "transition",
            "swipeArea",
        ],
    )
    .slot(UiSlotSchema::new("content").multiple(true))
    .events([
        UiComponentEventKind::OpenPopup,
        UiComponentEventKind::ClosePopup,
        UiComponentEventKind::DragDelta,
    ])
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
