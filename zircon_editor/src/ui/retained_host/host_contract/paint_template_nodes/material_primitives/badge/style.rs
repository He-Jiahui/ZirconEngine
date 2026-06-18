use super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::paint_theme::PALETTE;
use super::super::{component_variant_contains, first_non_empty, resolved_style_color};

const MUI_PRIMARY_MAIN: [u8; 4] = [25, 118, 210, 255];
const MUI_SECONDARY_MAIN: [u8; 4] = [156, 39, 176, 255];
const MUI_ERROR_MAIN: [u8; 4] = [211, 47, 47, 255];
const MUI_INFO_MAIN: [u8; 4] = [2, 136, 209, 255];
const MUI_SUCCESS_MAIN: [u8; 4] = [46, 125, 50, 255];
const MUI_WARNING_MAIN: [u8; 4] = [237, 108, 2, 255];
const MUI_BADGE_DEFAULT_BG: [u8; 4] = [117, 117, 117, 255];
const MUI_ON_DARK: [u8; 4] = [255, 255, 255, 255];
const MUI_ON_WARNING: [u8; 4] = [0, 0, 0, 222];

pub(super) fn badge_root_background_color(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.background_color.as_ref())
}

pub(super) fn badge_root_border_color(
    node: &TemplatePaneNodeData,
    border_width: f32,
) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.border_color.as_ref())
        .or_else(|| (border_width > 0.0).then_some(PALETTE.border))
}

pub(super) fn badge_root_border_width(node: &TemplatePaneNodeData) -> f32 {
    node.button_style
        .element
        .border_width
        .max(node.border_width)
        .max(0.0)
}

pub(super) fn badge_root_text_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.disabled {
        return PALETTE.text_disabled;
    }
    resolved_style_color(node.button_style.element.foreground_color.as_ref())
        .unwrap_or(PALETTE.text)
}

pub(super) fn badge_overlay_background_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    match badge_color_token(node) {
        "primary" => MUI_PRIMARY_MAIN,
        "secondary" => MUI_SECONDARY_MAIN,
        "info" => MUI_INFO_MAIN,
        "success" => MUI_SUCCESS_MAIN,
        "warning" => MUI_WARNING_MAIN,
        "default" => MUI_BADGE_DEFAULT_BG,
        "error" | "danger" => MUI_ERROR_MAIN,
        _ => {
            if matches!(
                first_non_empty(&[node.validation_level.as_str(), node.text_tone.as_str()]),
                "error" | "danger"
            ) {
                MUI_ERROR_MAIN
            } else {
                MUI_BADGE_DEFAULT_BG
            }
        }
    }
}

pub(super) fn badge_overlay_text_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    match badge_color_token(node) {
        "warning" => MUI_ON_WARNING,
        "default" => MUI_ON_DARK,
        _ => MUI_ON_DARK,
    }
}

pub(super) fn badge_overlay_border_color(
    node: &TemplatePaneNodeData,
    background: [u8; 4],
) -> [u8; 4] {
    if component_variant_contains(node, "overlapCircular")
        || component_variant_contains(node, "circular")
    {
        background
    } else {
        resolved_style_color(node.button_style.element.border_color.as_ref()).unwrap_or(background)
    }
}

pub(super) fn badge_overlay_border_width(node: &TemplatePaneNodeData) -> f32 {
    node.border_width
        .max(node.button_style.element.border_width)
        .max(0.0)
        .min(2.0)
}

fn badge_color_token(node: &TemplatePaneNodeData) -> &str {
    for token in [
        "primary",
        "secondary",
        "error",
        "danger",
        "info",
        "success",
        "warning",
        "default",
    ] {
        if component_variant_contains(node, token) {
            return token;
        }
    }
    first_non_empty(&[node.validation_level.as_str(), node.text_tone.as_str()])
}
