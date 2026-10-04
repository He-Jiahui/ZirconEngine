use super::super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};
use super::super::super::resolved_style_color;

// 根表面只用显式样式或声明边框；宿主调色板提供边框和文字默认值。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn badge_root_background_color(
    node: &TemplatePaneNodeData,
) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.background_color.as_ref())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn badge_root_border_color(
    node: &TemplatePaneNodeData,
    border_width: f32,
) -> Option<[u8; 4]> {
    badge_root_border_color_from_host(node, border_width, current_host_palette())
}

fn badge_root_border_color_from_host(
    node: &TemplatePaneNodeData,
    border_width: f32,
    palette: HostMaterialPalette,
) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.border_color.as_ref())
        .or_else(|| (border_width > 0.0).then_some(palette.border))
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn badge_root_border_width(
    node: &TemplatePaneNodeData,
) -> f32 {
    node.button_style
        .element
        .border_width
        .max(node.border_width)
        .max(0.0)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn badge_root_text_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    badge_root_text_color_from_host(node, current_host_palette())
}

fn badge_root_text_color_from_host(
    node: &TemplatePaneNodeData,
    palette: HostMaterialPalette,
) -> [u8; 4] {
    if node.disabled {
        return palette.text_disabled;
    }
    resolved_style_color(node.button_style.element.foreground_color.as_ref())
        .unwrap_or(palette.text)
}

#[cfg(test)]
#[path = "tests/root.rs"]
mod tests;
