use super::shared::*;
use zircon_runtime_interface::ui::component::UiPropSchema;

const GRID_DIRECTIONS: [&str; 2] = ["row", "row-reverse"];
const GRID_WRAPS: [&str; 3] = ["nowrap", "wrap", "wrap-reverse"];
const STACK_DIRECTIONS: [&str; 4] = ["column-reverse", "column", "row-reverse", "row"];

pub(super) fn descriptors() -> Vec<UiComponentDescriptor> {
    vec![box_descriptor(), container(), grid(), stack(), masonry()]
}

fn box_descriptor() -> UiComponentDescriptor {
    layout("Box", "Box", UiComponentLayoutRole::Flex, "box")
        .with_prop(default_string_prop("component", "div"))
}

fn container() -> UiComponentDescriptor {
    add_props(
        layout(
            "Container",
            "Container",
            UiComponentLayoutRole::Flex,
            "container",
        ),
        [
            default_string_prop("component", "div"),
            bool_prop("disableGutters", false),
            bool_prop("fixed", false),
            mui_enum_prop("maxWidth", "lg", ["false", "xs", "sm", "md", "lg", "xl"]),
            float_prop("max_width", 1200.0),
        ],
    )
}

fn grid() -> UiComponentDescriptor {
    add_props(
        layout("Grid", "Grid", UiComponentLayoutRole::Grid, "grid"),
        [
            default_string_prop("component", "div"),
            int_prop("columns", 12),
            default_string_prop("columnSpacing", "0"),
            bool_prop("container", false),
            mui_enum_prop("direction", "row", GRID_DIRECTIONS),
            default_string_prop("offset", ""),
            default_string_prop("rowSpacing", "0"),
            default_string_prop("size", "false"),
            default_string_prop("spacing", "0"),
            mui_enum_prop("wrap", "wrap", GRID_WRAPS),
            int_prop("unstable_level", 0),
        ],
    )
}

fn stack() -> UiComponentDescriptor {
    add_props(
        layout("Stack", "Stack", UiComponentLayoutRole::Flex, "stack"),
        [
            default_string_prop("component", "div"),
            mui_enum_prop("direction", "column", STACK_DIRECTIONS),
            any_prop("divider"),
            default_string_prop("spacing", "0"),
            bool_prop("useFlexGap", false),
        ],
    )
}

fn masonry() -> UiComponentDescriptor {
    add_props(
        layout("Masonry", "Masonry", UiComponentLayoutRole::Grid, "masonry"),
        [
            default_string_prop("component", "div"),
            int_prop("columns", 4),
            default_string_prop("spacing", "1"),
            bool_prop("sequential", false),
            int_prop("defaultColumns", 0),
            default_string_prop("defaultHeight", ""),
            default_string_prop("defaultSpacing", ""),
            bool_prop("needs_support", true),
        ],
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
