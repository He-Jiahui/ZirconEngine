use super::shared::*;
use zircon_runtime_interface::ui::component::UiPropSchema;

const MUI_COLORS: [&str; 7] = [
    "default",
    "primary",
    "secondary",
    "error",
    "info",
    "success",
    "warning",
];
const MUI_ICON_COLORS: [&str; 9] = [
    "inherit",
    "action",
    "disabled",
    "primary",
    "secondary",
    "error",
    "info",
    "success",
    "warning",
];
const MUI_ICON_FONT_SIZES: [&str; 4] = ["inherit", "small", "medium", "large"];
const TYPOGRAPHY_VARIANTS: [&str; 14] = [
    "body1",
    "body2",
    "button",
    "caption",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "inherit",
    "overline",
    "subtitle1",
    "subtitle2",
];

pub(super) fn descriptors() -> Vec<UiComponentDescriptor> {
    vec![
        primitive("Chip", "Chip", UiComponentCategory::Selection, "chip")
            .with_prop(text_prop())
            .with_prop(default_string_prop("label", ""))
            .with_prop(icon_prop())
            .with_prop(default_string_prop("component", "div"))
            .with_prop(mui_enum_prop("color", "default", MUI_COLORS))
            .with_prop(mui_enum_prop("size", "medium", ["small", "medium"]))
            .with_prop(mui_enum_prop("variant", "filled", ["filled", "outlined"]))
            .with_prop(bool_prop("clickable", false))
            .with_prop(bool_prop("deletable", false))
            .with_prop(bool_prop("onDelete", false))
            .with_prop(default_string_prop("deleteIcon", ""))
            .with_prop(bool_prop("skipFocusWhenDisabled", false))
            .slot(UiSlotSchema::new("avatar"))
            .slot(UiSlotSchema::new("icon"))
            .slot(UiSlotSchema::new("label"))
            .slot(UiSlotSchema::new("deleteIcon"))
            .events([
                UiComponentEventKind::Commit,
                UiComponentEventKind::SelectOption,
            ]),
        primitive("Divider", "Divider", UiComponentCategory::Visual, "divider")
            .with_prop(mui_enum_prop(
                "orientation",
                "horizontal",
                ["horizontal", "vertical"],
            ))
            .with_prop(mui_enum_prop(
                "variant",
                "fullWidth",
                ["fullWidth", "inset", "middle"],
            ))
            .with_prop(mui_enum_prop(
                "textAlign",
                "center",
                ["center", "left", "right"],
            ))
            .with_prop(default_string_prop("component", "hr"))
            .with_prop(default_string_prop("role", ""))
            .with_prop(bool_prop("absolute", false))
            .with_prop(bool_prop("flexItem", false))
            .with_prop(text_prop())
            .slot(UiSlotSchema::new("wrapper")),
        primitive("Icon", "Icon", UiComponentCategory::Visual, "icon")
            .with_prop(text_prop())
            .with_prop(icon_prop())
            .with_prop(default_string_prop("baseClassName", "material-icons"))
            .with_prop(default_string_prop("component", "span"))
            .with_prop(mui_enum_prop("color", "inherit", MUI_ICON_COLORS))
            .with_prop(mui_enum_prop("fontSize", "medium", MUI_ICON_FONT_SIZES))
            .requires_render_capability(UiRenderCapability::Vector),
        primitive(
            "SvgIcon",
            "Svg Icon",
            UiComponentCategory::Visual,
            "svg-icon",
        )
        .with_prop(text_prop())
        .with_prop(icon_prop())
        .with_prop(default_string_prop("component", "svg"))
        .with_prop(mui_enum_prop("color", "inherit", MUI_ICON_COLORS))
        .with_prop(mui_enum_prop("fontSize", "medium", MUI_ICON_FONT_SIZES))
        .with_prop(default_string_prop("htmlColor", ""))
        .with_prop(default_string_prop("viewBox", "0 0 24 24"))
        .with_prop(default_string_prop("titleAccess", ""))
        .with_prop(bool_prop("inheritViewBox", false))
        .requires_render_capability(UiRenderCapability::Vector),
        composite("List", "List", UiComponentCategory::Collection, "list")
            .with_prop(array_prop("items"))
            .with_prop(default_string_prop("component", "ul"))
            .with_prop(bool_prop("dense", false))
            .with_prop(bool_prop("disablePadding", false))
            .with_prop(default_string_prop("subheader", ""))
            .slot(UiSlotSchema::new("subheader"))
            .slot(UiSlotSchema::new("items").multiple(true))
            .events([
                UiComponentEventKind::Hover,
                UiComponentEventKind::Press,
                UiComponentEventKind::SelectOption,
            ]),
        data_view("ListView", "List View", "list-view")
            .with_prop(int_prop("item_count", 0))
            .slot(UiSlotSchema::new("items").multiple(true))
            .event(UiComponentEventKind::SelectOption),
        virtualized_range_props(
            data_view("VirtualList", "Virtual List", "virtual-list")
                .descriptor_kind(UiComponentDescriptorKind::Layout)
                .layout_role(UiComponentLayoutRole::VirtualList),
        )
        .event(UiComponentEventKind::SetVisibleRange)
        .requires_host_capability(UiHostCapability::VirtualizedLayout)
        .requires_render_capability(UiRenderCapability::VirtualizedLayout),
        data_view("TreeView", "Tree View", "tree-view")
            .with_prop(string_prop("query"))
            .with_prop(expanded_prop())
            .slot(UiSlotSchema::new("nodes").multiple(true))
            .events([
                UiComponentEventKind::SelectOption,
                UiComponentEventKind::ToggleExpanded,
                UiComponentEventKind::OpenPopupAt,
                UiComponentEventKind::Commit,
            ]),
        composite("Table", "Table", UiComponentCategory::Collection, "table")
            .with_prop(array_prop("rows"))
            .with_prop(array_prop("columns"))
            .with_prop(default_string_prop("component", "table"))
            .with_prop(mui_enum_prop(
                "padding",
                "normal",
                ["checkbox", "none", "normal"],
            ))
            .with_prop(mui_enum_prop("size", "medium", ["medium", "small"]))
            .with_prop(bool_prop("stickyHeader", false))
            .slot(UiSlotSchema::new("header").multiple(true))
            .slot(UiSlotSchema::new("row").multiple(true))
            .events([
                UiComponentEventKind::Hover,
                UiComponentEventKind::Press,
                UiComponentEventKind::SelectOption,
            ]),
        primitive(
            "Typography",
            "Typography",
            UiComponentCategory::Visual,
            "typography",
        )
        .with_prop(text_prop())
        .with_prop(mui_enum_prop("variant", "body1", TYPOGRAPHY_VARIANTS))
        .with_prop(mui_enum_prop(
            "align",
            "inherit",
            ["center", "inherit", "justify", "left", "right"],
        ))
        .with_prop(default_string_prop("color", ""))
        .with_prop(default_string_prop("component", ""))
        .with_prop(bool_prop("gutterBottom", false))
        .with_prop(bool_prop("noWrap", false))
        .with_prop(map_prop("variantMapping")),
        primitive("Avatar", "Avatar", UiComponentCategory::Visual, "avatar")
            .with_prop(text_prop())
            .with_prop(string_prop("image"))
            .with_prop(default_string_prop("alt", ""))
            .with_prop(default_string_prop("component", "div"))
            .with_prop(default_string_prop("src", ""))
            .with_prop(default_string_prop("srcSet", ""))
            .with_prop(default_string_prop("sizes", ""))
            .with_prop(mui_enum_prop(
                "variant",
                "circular",
                ["circular", "rounded", "square"],
            ))
            .slot(UiSlotSchema::new("img"))
            .slot(UiSlotSchema::new("fallback"))
            .requires_render_capability(UiRenderCapability::Image),
        composite(
            "AvatarGroup",
            "Avatar Group",
            UiComponentCategory::Visual,
            "avatar-group",
        )
        .with_prop(int_prop("max", 4))
        .slot(UiSlotSchema::new("avatars").multiple(true)),
        primitive("Badge", "Badge", UiComponentCategory::Feedback, "badge")
            .with_prop(text_prop())
            .with_prop(value_text_prop())
            .with_prop(default_string_prop("badgeContent", ""))
            .with_prop(int_prop("max", 99))
            .with_prop(bool_prop("showZero", false))
            .with_prop(bool_prop("invisible", false))
            .with_prop(mui_enum_prop(
                "overlap",
                "rectangular",
                ["circular", "rectangular"],
            ))
            .with_prop(mui_enum_prop("variant", "standard", ["dot", "standard"]))
            .with_prop(mui_enum_prop("color", "default", MUI_COLORS))
            .with_prop(map_prop("anchorOrigin"))
            .with_prop(mui_enum_prop(
                "anchor_origin_vertical",
                "top",
                ["top", "bottom"],
            ))
            .with_prop(mui_enum_prop(
                "anchor_origin_horizontal",
                "right",
                ["left", "right"],
            ))
            .slot(UiSlotSchema::new("badge")),
        composite(
            "ImageList",
            "Image List",
            UiComponentCategory::Collection,
            "image-list",
        )
        .with_prop(array_prop("items"))
        .with_prop(int_prop("cols", 2))
        .with_prop(default_string_prop("component", "ul"))
        .with_prop(float_prop("gap", 4.0))
        .with_prop(default_string_prop("rowHeight", "auto"))
        .with_prop(mui_enum_prop(
            "variant",
            "standard",
            ["masonry", "quilted", "standard", "woven"],
        ))
        .slot(UiSlotSchema::new("items").multiple(true))
        .requires_render_capability(UiRenderCapability::Image),
    ]
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
