use super::shared::*;
use zircon_runtime_interface::ui::component::UiPropSchema;

const ALIGN_ITEMS: [&str; 2] = ["center", "flex-start"];
const IMAGE_LIST_VARIANTS: [&str; 4] = ["masonry", "quilted", "standard", "woven"];

pub(super) fn descriptors() -> Vec<UiComponentDescriptor> {
    vec![
        list_item(),
        list_item_avatar(),
        list_item_button(),
        list_item_icon(),
        list_item_secondary_action(),
        list_item_text(),
        list_subheader(),
        image_list_item(),
        image_list_item_bar(),
    ]
}

fn list_item() -> UiComponentDescriptor {
    add_slots(
        add_props(
            composite(
                "ListItem",
                "List Item",
                UiComponentCategory::Collection,
                "list-item",
            ),
            [
                default_string_prop("component", "li"),
                mui_enum_prop("alignItems", "center", ALIGN_ITEMS),
                bool_prop("dense", false),
                bool_prop("disableGutters", false),
                bool_prop("disablePadding", false),
                bool_prop("divider", false),
                default_string_prop("secondaryAction", ""),
            ],
        ),
        ["content", "secondaryAction"],
    )
    .events([UiComponentEventKind::Hover, UiComponentEventKind::Press])
}

fn list_item_avatar() -> UiComponentDescriptor {
    add_slots(
        add_props(
            composite(
                "ListItemAvatar",
                "List Item Avatar",
                UiComponentCategory::Visual,
                "list-item-avatar",
            ),
            [mui_enum_prop("alignItems", "center", ALIGN_ITEMS)],
        ),
        ["content"],
    )
}

fn list_item_button() -> UiComponentDescriptor {
    add_props(
        primitive(
            "ListItemButton",
            "List Item Button",
            UiComponentCategory::Input,
            "list-item-button",
        ),
        [
            default_string_prop("component", "div"),
            mui_enum_prop("alignItems", "center", ALIGN_ITEMS),
            bool_prop("autoFocus", false),
            bool_prop("dense", false),
            bool_prop("disableGutters", false),
            bool_prop("divider", false),
            default_string_prop("focusVisibleClassName", ""),
            default_string_prop("href", ""),
            bool_prop("selected", false),
            default_string_prop("to", ""),
        ],
    )
    .events([
        UiComponentEventKind::Focus,
        UiComponentEventKind::Press,
        UiComponentEventKind::SelectOption,
        UiComponentEventKind::Commit,
    ])
}

fn list_item_icon() -> UiComponentDescriptor {
    add_slots(
        add_props(
            composite(
                "ListItemIcon",
                "List Item Icon",
                UiComponentCategory::Visual,
                "list-item-icon",
            ),
            [
                icon_prop(),
                mui_enum_prop("alignItems", "center", ALIGN_ITEMS),
            ],
        ),
        ["content"],
    )
    .requires_render_capability(UiRenderCapability::Vector)
}

fn list_item_secondary_action() -> UiComponentDescriptor {
    add_slots(
        add_props(
            primitive(
                "ListItemSecondaryAction",
                "List Item Secondary Action",
                UiComponentCategory::Input,
                "list-item-secondary-action",
            ),
            [
                default_string_prop("component", "div"),
                bool_prop("disableGutters", false),
            ],
        ),
        ["content"],
    )
    .event(UiComponentEventKind::Press)
}

fn list_item_text() -> UiComponentDescriptor {
    add_slots(
        add_props(
            primitive(
                "ListItemText",
                "List Item Text",
                UiComponentCategory::Visual,
                "list-item-text",
            ),
            [
                text_prop(),
                default_string_prop("primary", ""),
                default_string_prop("secondary", ""),
                bool_prop("dense", false),
                bool_prop("disableTypography", false),
                bool_prop("inset", false),
            ],
        ),
        ["primary", "secondary"],
    )
}

fn list_subheader() -> UiComponentDescriptor {
    add_props(
        primitive(
            "ListSubheader",
            "List Subheader",
            UiComponentCategory::Visual,
            "list-subheader",
        ),
        [
            text_prop(),
            default_string_prop("component", "li"),
            mui_enum_prop("color", "default", ["default", "inherit", "primary"]),
            bool_prop("disableGutters", false),
            bool_prop("disableSticky", false),
            bool_prop("inset", false),
        ],
    )
}

fn image_list_item() -> UiComponentDescriptor {
    add_slots(
        add_props(
            composite(
                "ImageListItem",
                "Image List Item",
                UiComponentCategory::Collection,
                "image-list-item",
            ),
            [
                default_string_prop("alt", ""),
                int_prop("cols", 1),
                default_string_prop("component", "li"),
                int_prop("rows", 1),
                default_string_prop("src", ""),
                mui_enum_prop("variant", "standard", IMAGE_LIST_VARIANTS),
            ],
        ),
        ["img", "bar", "content"],
    )
    .requires_render_capability(UiRenderCapability::Image)
}

fn image_list_item_bar() -> UiComponentDescriptor {
    add_slots(
        add_props(
            composite(
                "ImageListItemBar",
                "Image List Item Bar",
                UiComponentCategory::Visual,
                "image-list-item-bar",
            ),
            [
                default_string_prop("actionIcon", ""),
                mui_enum_prop("actionPosition", "right", ["left", "right"]),
                mui_enum_prop("position", "bottom", ["below", "bottom", "top"]),
                default_string_prop("subtitle", ""),
                default_string_prop("title", ""),
            ],
        ),
        ["titleWrap", "title", "subtitle", "actionIcon"],
    )
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
