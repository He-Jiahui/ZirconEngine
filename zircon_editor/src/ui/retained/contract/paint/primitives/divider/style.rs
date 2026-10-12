use super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};
use super::super::{first_non_empty, resolved_style_color};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn divider_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    divider_color_from_host(node, current_host_palette())
}

fn divider_color_from_host(node: &TemplatePaneNodeData, palette: HostMaterialPalette) -> [u8; 4] {
    if node.disabled || node.validation_level.as_str() == "disabled" {
        return palette.border_disabled;
    }
    resolved_style_color(node.button_style.element.border_color.as_ref())
        .or_else(|| resolved_style_color(node.button_style.element.foreground_color.as_ref()))
        .unwrap_or(palette.border)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn divider_text_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    divider_text_color_from_host(node, current_host_palette())
}

fn divider_text_color_from_host(
    node: &TemplatePaneNodeData,
    palette: HostMaterialPalette,
) -> [u8; 4] {
    if node.disabled || node.validation_level.as_str() == "disabled" {
        return palette.text_disabled;
    }
    resolved_style_color(node.button_style.element.foreground_color.as_ref()).unwrap_or_else(|| {
        match first_non_empty(&[node.text_tone.as_str(), node.validation_level.as_str()]) {
            "primary" | "accent" => palette.accent,
            "muted" | "secondary" => palette.text_muted,
            "warning" => palette.warning,
            "error" | "danger" => palette.error,
            "success" => palette.success,
            "info" => palette.info,
            _ => palette.text,
        }
    })
}

#[cfg(test)]
#[path = "tests/style.rs"]
mod tests;
