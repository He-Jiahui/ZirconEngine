use super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};
use super::super::resolved_style_color;

const SKELETON_DISABLED_OPACITY: f32 = 0.56;

/// 显式背景色优先于宿主色板；宿主色板只补齐模板未声明的骨架填充。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn skeleton_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    skeleton_color_from_host(node, current_host_palette())
}

fn skeleton_color_from_host(node: &TemplatePaneNodeData, palette: HostMaterialPalette) -> [u8; 4] {
    resolved_style_color(node.button_style.element.background_color.as_ref())
        .unwrap_or(palette.surface_hover)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn skeleton_wave_color(
) -> [u8; 4] {
    skeleton_wave_color_from_host(current_host_palette())
}

fn skeleton_wave_color_from_host(palette: HostMaterialPalette) -> [u8; 4] {
    palette.separator_soft
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn skeleton_border_color(
    node: &TemplatePaneNodeData,
) -> Option<[u8; 4]> {
    skeleton_border_color_from_host(node, current_host_palette())
}

fn skeleton_border_color_from_host(
    node: &TemplatePaneNodeData,
    palette: HostMaterialPalette,
) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.border_color.as_ref())
        .or_else(|| (skeleton_border_width(node) > 0.0).then_some(palette.surface_hover))
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn skeleton_border_width(
    node: &TemplatePaneNodeData,
) -> f32 {
    let width = node
        .button_style
        .element
        .border_width
        .max(node.border_width);
    if width.is_finite() {
        width.max(0.0)
    } else {
        0.0
    }
}

#[cfg(test)]
#[path = "tests/style.rs"]
mod tests;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn skeleton_opacity(
    node: &TemplatePaneNodeData,
) -> f32 {
    if node.disabled {
        SKELETON_DISABLED_OPACITY
    } else {
        1.0
    }
}
