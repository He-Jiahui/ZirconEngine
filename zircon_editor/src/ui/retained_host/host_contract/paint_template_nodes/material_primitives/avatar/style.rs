use super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::paint_theme::PALETTE;
use super::super::{component_variant_contains, first_non_empty, resolved_style_color};

const MUI_GREY_600: [u8; 4] = [117, 117, 117, 255];

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn avatar_background_color(
    node: &TemplatePaneNodeData,
    color_default: bool,
) -> [u8; 4] {
    if node.disabled {
        return PALETTE.surface_disabled;
    }
    resolved_style_color(node.button_style.element.background_color.as_ref()).unwrap_or_else(|| {
        if color_default || component_variant_contains(node, "colorDefault") {
            MUI_GREY_600
        } else {
            PALETTE.surface_selected
        }
    })
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn avatar_foreground_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    if node.disabled {
        return PALETTE.text_disabled;
    }
    resolved_style_color(node.button_style.element.foreground_color.as_ref()).unwrap_or_else(|| {
        match first_non_empty(&[node.text_tone.as_str(), node.validation_level.as_str()]) {
            "primary" | "accent" => PALETTE.accent,
            "muted" | "secondary" => PALETTE.text_muted,
            "warning" => PALETTE.warning,
            "error" | "danger" => PALETTE.error,
            "success" => PALETTE.success,
            "info" => PALETTE.info,
            _ => PALETTE.text,
        }
    })
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn avatar_border_color(
    node: &TemplatePaneNodeData,
) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.border_color.as_ref()).or_else(|| {
        (node.border_width > 0.0 || node.button_style.element.border_width > 0.0)
            .then_some(PALETTE.border)
    })
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn avatar_border_width(
    node: &TemplatePaneNodeData,
) -> f32 {
    node.button_style
        .element
        .border_width
        .max(node.border_width)
        .max(1.0)
}
