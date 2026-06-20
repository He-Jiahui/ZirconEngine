use super::super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::{component_variant_contains, first_non_empty, resolved_style_color};
use super::palette::{
    MUI_BADGE_DEFAULT_BG, MUI_ERROR_MAIN, MUI_INFO_MAIN, MUI_ON_DARK, MUI_ON_WARNING,
    MUI_PRIMARY_MAIN, MUI_SECONDARY_MAIN, MUI_SUCCESS_MAIN, MUI_WARNING_MAIN,
};
use super::tokens::badge_color_token;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn badge_overlay_background_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
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

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn badge_overlay_text_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    match badge_color_token(node) {
        "warning" => MUI_ON_WARNING,
        "default" => MUI_ON_DARK,
        _ => MUI_ON_DARK,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn badge_overlay_border_color(
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

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn badge_overlay_border_width(
    node: &TemplatePaneNodeData,
) -> f32 {
    node.border_width
        .max(node.button_style.element.border_width)
        .max(0.0)
        .min(2.0)
}
