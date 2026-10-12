use super::super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};
use super::super::super::resolved_style_color;
use super::tokens::timeline_neutral_color_from_host;

/// 连接线优先采用模板显式背景、前景或边框色；均缺省时读取宿主分隔线色。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn timeline_connector_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    timeline_connector_color_from_host(node, current_host_palette())
}

fn timeline_connector_color_from_host(
    node: &TemplatePaneNodeData,
    palette: HostMaterialPalette,
) -> [u8; 4] {
    resolved_style_color(node.button_style.element.background_color.as_ref())
        .or_else(|| resolved_style_color(node.button_style.element.foreground_color.as_ref()))
        .or_else(|| resolved_style_color(node.button_style.element.border_color.as_ref()))
        .unwrap_or_else(|| timeline_neutral_color_from_host(palette))
}

#[cfg(test)]
#[path = "tests/connector.rs"]
mod tests;
