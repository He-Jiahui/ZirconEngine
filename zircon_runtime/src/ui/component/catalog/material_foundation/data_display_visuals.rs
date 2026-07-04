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

pub(super) fn descriptors() -> Vec<UiComponentDescriptor> {
    vec![
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
