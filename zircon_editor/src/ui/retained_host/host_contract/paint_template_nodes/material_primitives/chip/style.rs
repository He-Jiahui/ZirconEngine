use super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::paint_theme::PALETTE;
use super::super::{component_variant_contains, resolved_style_color};
use super::chip_is_outlined;

const MUI_PRIMARY_MAIN: [u8; 4] = [25, 118, 210, 255];
const MUI_PRIMARY_DARK: [u8; 4] = [21, 101, 192, 255];
const MUI_SECONDARY_MAIN: [u8; 4] = [156, 39, 176, 255];
const MUI_SECONDARY_DARK: [u8; 4] = [123, 31, 162, 255];
const MUI_ERROR_MAIN: [u8; 4] = [211, 47, 47, 255];
const MUI_ERROR_DARK: [u8; 4] = [198, 40, 40, 255];
const MUI_INFO_MAIN: [u8; 4] = [2, 136, 209, 255];
const MUI_INFO_DARK: [u8; 4] = [1, 87, 155, 255];
const MUI_SUCCESS_MAIN: [u8; 4] = [46, 125, 50, 255];
const MUI_SUCCESS_DARK: [u8; 4] = [27, 94, 32, 255];
const MUI_WARNING_MAIN: [u8; 4] = [237, 108, 2, 255];
const MUI_WARNING_DARK: [u8; 4] = [230, 81, 0, 255];
const MUI_CHIP_DEFAULT_FILLED: [u8; 4] = [66, 66, 66, 255];
const MUI_CHIP_DEFAULT_AVATAR: [u8; 4] = [117, 117, 117, 255];
const MUI_ON_DARK: [u8; 4] = [255, 255, 255, 255];
const MUI_ON_WARNING: [u8; 4] = [0, 0, 0, 222];

pub(super) fn chip_background_color(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.background_color.as_ref()).or_else(|| {
        if chip_is_outlined(node) {
            None
        } else {
            Some(match chip_color_token(node) {
                "primary" => MUI_PRIMARY_MAIN,
                "secondary" => MUI_SECONDARY_MAIN,
                "error" => MUI_ERROR_MAIN,
                "info" => MUI_INFO_MAIN,
                "success" => MUI_SUCCESS_MAIN,
                "warning" => MUI_WARNING_MAIN,
                _ => MUI_CHIP_DEFAULT_FILLED,
            })
        }
    })
}

pub(super) fn chip_foreground_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.disabled {
        return PALETTE.text_disabled;
    }
    resolved_style_color(node.button_style.element.foreground_color.as_ref()).unwrap_or_else(|| {
        let color = chip_color_token(node);
        if chip_is_outlined(node) {
            chip_palette_main(color).unwrap_or(PALETTE.text)
        } else if color == "warning" {
            MUI_ON_WARNING
        } else if color == "default" {
            PALETTE.text
        } else {
            MUI_ON_DARK
        }
    })
}

pub(super) fn chip_border_color(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.border_color.as_ref()).or_else(|| {
        if chip_is_outlined(node) {
            Some(chip_palette_main(chip_color_token(node)).unwrap_or(PALETTE.border))
        } else if node.border_width > 0.0 || node.button_style.element.border_width > 0.0 {
            Some(PALETTE.border)
        } else {
            None
        }
    })
}

pub(super) fn chip_border_width(node: &TemplatePaneNodeData) -> f32 {
    node.button_style
        .element
        .border_width
        .max(node.border_width)
        .max(if chip_is_outlined(node) { 1.0 } else { 0.0 })
}

pub(super) fn chip_avatar_background_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    match chip_color_token(node) {
        "primary" => MUI_PRIMARY_DARK,
        "secondary" => MUI_SECONDARY_DARK,
        "error" => MUI_ERROR_DARK,
        "info" => MUI_INFO_DARK,
        "success" => MUI_SUCCESS_DARK,
        "warning" => MUI_WARNING_DARK,
        _ => MUI_CHIP_DEFAULT_AVATAR,
    }
}

pub(super) fn chip_delete_icon_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if chip_is_outlined(node) {
        chip_palette_main(chip_color_token(node)).unwrap_or(PALETTE.text_muted)
    } else {
        chip_foreground_color(node)
    }
}

fn chip_palette_main(color: &str) -> Option<[u8; 4]> {
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

fn chip_color_token(node: &TemplatePaneNodeData) -> &str {
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
