use super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::paint_theme::PALETTE;
use super::super::{first_non_empty, resolved_style_color};

pub(super) fn divider_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.disabled || node.validation_level.as_str() == "disabled" {
        return PALETTE.border_disabled;
    }
    resolved_style_color(node.button_style.element.border_color.as_ref())
        .or_else(|| resolved_style_color(node.button_style.element.foreground_color.as_ref()))
        .unwrap_or(PALETTE.border)
}

pub(super) fn divider_text_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.disabled || node.validation_level.as_str() == "disabled" {
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
