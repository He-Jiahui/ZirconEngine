use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

use super::super::super::component_variant_contains;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes::material_primitives::chip::style) const MUI_PRIMARY_MAIN: [u8; 4] =
    [25, 118, 210, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes::material_primitives::chip::style) const MUI_PRIMARY_DARK: [u8; 4] =
    [21, 101, 192, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes::material_primitives::chip::style) const MUI_SECONDARY_MAIN: [u8; 4] =
    [156, 39, 176, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes::material_primitives::chip::style) const MUI_SECONDARY_DARK: [u8; 4] =
    [123, 31, 162, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes::material_primitives::chip::style) const MUI_ERROR_MAIN: [u8; 4] =
    [211, 47, 47, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes::material_primitives::chip::style) const MUI_ERROR_DARK: [u8; 4] =
    [198, 40, 40, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes::material_primitives::chip::style) const MUI_INFO_MAIN: [u8; 4] =
    [2, 136, 209, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes::material_primitives::chip::style) const MUI_INFO_DARK: [u8; 4] =
    [1, 87, 155, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes::material_primitives::chip::style) const MUI_SUCCESS_MAIN: [u8; 4] =
    [46, 125, 50, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes::material_primitives::chip::style) const MUI_SUCCESS_DARK: [u8; 4] =
    [27, 94, 32, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes::material_primitives::chip::style) const MUI_WARNING_MAIN: [u8; 4] =
    [237, 108, 2, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes::material_primitives::chip::style) const MUI_WARNING_DARK: [u8; 4] =
    [230, 81, 0, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes::material_primitives::chip::style) const MUI_CHIP_DEFAULT_FILLED: [u8; 4] =
    [66, 66, 66, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes::material_primitives::chip::style) const MUI_CHIP_DEFAULT_AVATAR: [u8; 4] =
    [117, 117, 117, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes::material_primitives::chip::style) const MUI_ON_DARK: [u8; 4] =
    [255, 255, 255, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes::material_primitives::chip::style) const MUI_ON_WARNING: [u8; 4] =
    [0, 0, 0, 222];

pub(in crate::ui::retained_host::host_contract::paint_template_nodes::material_primitives::chip::style) fn chip_palette_main(
    color: &str,
) -> Option<[u8; 4]> {
    match color {
        "primary" => Some(MUI_PRIMARY_MAIN),
        "secondary" => Some(MUI_SECONDARY_MAIN),
        "error" => Some(MUI_ERROR_MAIN),
        "info" => Some(MUI_INFO_MAIN),
        "success" => Some(MUI_SUCCESS_MAIN),
        "warning" => Some(MUI_WARNING_MAIN),
        _ => None,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes::material_primitives::chip::style) fn chip_color_token(
    node: &TemplatePaneNodeData,
) -> &str {
    if component_variant_contains(node, "primary")
        || component_variant_contains(node, "colorPrimary")
    {
        "primary"
    } else if component_variant_contains(node, "secondary")
        || component_variant_contains(node, "colorSecondary")
    {
        "secondary"
    } else if component_variant_contains(node, "error")
        || component_variant_contains(node, "colorError")
    {
        "error"
    } else if component_variant_contains(node, "info")
        || component_variant_contains(node, "colorInfo")
    {
        "info"
    } else if component_variant_contains(node, "success")
        || component_variant_contains(node, "colorSuccess")
    {
        "success"
    } else if component_variant_contains(node, "warning")
        || component_variant_contains(node, "colorWarning")
    {
        "warning"
    } else {
        "default"
    }
}
