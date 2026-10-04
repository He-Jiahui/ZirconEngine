use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_template_nodes::material_primitives::resolved_style_color;
use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_metrics, current_host_palette, HostControlMetrics, HostMaterialPalette,
};

/// 显式 Paper 背景优先，缺省采用宿主 popup 表面色；边框由 outlined 或显式宽度另行决定。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn paper_background_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    paper_background_color_from_host(node, current_host_palette())
}

fn paper_background_color_from_host(
    node: &TemplatePaneNodeData,
    palette: HostMaterialPalette,
) -> [u8; 4] {
    resolved_style_color(node.button_style.element.background_color.as_ref())
        .unwrap_or(palette.popup)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn paper_border_color(
    node: &TemplatePaneNodeData,
    outlined: bool,
) -> Option<[u8; 4]> {
    paper_border_color_from_host(node, outlined, current_host_palette())
}

fn paper_border_color_from_host(
    node: &TemplatePaneNodeData,
    outlined: bool,
    palette: HostMaterialPalette,
) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.border_color.as_ref())
        .or_else(|| (paper_border_width(node, outlined) > 0.0).then_some(palette.border))
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn paper_border_width(
    node: &TemplatePaneNodeData,
    outlined: bool,
) -> f32 {
    paper_border_width_from_host(node, outlined, current_host_metrics())
}

fn paper_border_width_from_host(
    node: &TemplatePaneNodeData,
    outlined: bool,
    metrics: HostControlMetrics,
) -> f32 {
    let configured = node
        .border_width
        .max(node.button_style.element.border_width)
        .max(0.0);
    if outlined {
        configured.max(metrics.border_width)
    } else {
        configured
    }
}

#[cfg(test)]
#[path = "tests/colors.rs"]
mod tests;
