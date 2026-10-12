//! 禁用态优先于节点声明标签色；其余先用声明色，再区分 Scale 轴和普通轴主题色。
//! 声明 alpha=0 表示未提供该局部覆盖色。

use super::super::super::super::data::TemplatePaneNodeData;
use super::super::identity::is_transform_scale_axis_control_id;
use super::super::palette::axis_label_palette;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn axis_label_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    let palette = axis_label_palette();
    if node.disabled {
        palette.disabled_axis
    } else if let Some(color) = declared_label_color(node) {
        color
    } else if is_scale_axis(node) {
        palette.scale_axis
    } else {
        palette.axis
    }
}

fn declared_label_color(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    (node.label_color.a > 0).then_some([
        node.label_color.r,
        node.label_color.g,
        node.label_color.b,
        node.label_color.a,
    ])
}

fn is_scale_axis(node: &TemplatePaneNodeData) -> bool {
    is_transform_scale_axis_control_id(node.control_id.as_str())
}

#[cfg(test)]
#[path = "tests/axis.rs"]
mod tests;
