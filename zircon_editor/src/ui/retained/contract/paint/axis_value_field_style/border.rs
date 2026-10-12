//! 轴值轮廓用禁用、验证错误/警告、焦点/按压、hover/selected 的优先级；校验色压过焦点提示。
//! 字段视觉和宽度由 surface 一起消费，不能独立改动而漏掉焦点态。

use super::super::super::data::TemplatePaneNodeData;
use super::super::super::paint_theme::HostMaterialPalette;
use super::super::style_selector::focus_visible_for_node;
use super::colors::{
    axis_field_disabled_border, axis_field_hover_border, axis_field_normal_border,
    axis_field_palette,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn axis_field_border(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    axis_field_border_from_host(node, axis_field_palette())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn axis_field_border_from_host(
    node: &TemplatePaneNodeData,
    palette: HostMaterialPalette,
) -> [u8; 4] {
    if node.disabled {
        axis_field_disabled_border(palette)
    } else if matches!(node.validation_level.as_str(), "error" | "danger") {
        palette.error
    } else if matches!(node.validation_level.as_str(), "warning") {
        palette.warning
    } else if focus_visible_for_node(node) || node.pressed {
        palette.focus_ring
    } else if node.hovered || node.selected {
        axis_field_hover_border(palette)
    } else {
        axis_field_normal_border(palette)
    }
}

// TODO: [CR-EDITOR-PAINT-ROWS-0006] 字段半径、行高使用动态 HostControlMetrics，
// 边框宽度仍固定 1.0/1.5；需确认主题密度调整时这些宽度是否故意恒定。
// 若应跟随 border_width，普通与强调态须同时约定比例及像素对齐。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn axis_field_border_width(
    node: &TemplatePaneNodeData,
) -> f32 {
    if focus_visible_for_node(node)
        || node.pressed
        || matches!(
            node.validation_level.as_str(),
            "error" | "danger" | "warning"
        )
    {
        1.5
    } else {
        1.0
    }
}
