//! 比例链接仅使用主题链接色与禁用色；ZUI 控件身份负责显示，图标资源入口负责实际像素。

use super::super::super::super::data::TemplatePaneNodeData;
use super::super::palette::axis_label_palette;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn scale_link_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    let palette = axis_label_palette();
    if node.disabled {
        palette.disabled_scale_link
    } else {
        palette.scale_link
    }
}

#[cfg(test)]
#[path = "tests/scale_link.rs"]
mod tests;
