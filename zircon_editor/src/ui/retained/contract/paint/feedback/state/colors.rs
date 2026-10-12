//! 消费节点转换层将 track/fill 别名投影到样式通道后的颜色；显式样式先于主题回退。

use super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::template_style_color::resolved_style_color;
use super::super::palette::material_feedback_palette;
use super::tone::material_tone_color;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn progress_track_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    let palette = material_feedback_palette();
    if node.disabled {
        return resolved_style_color(node.button_style.element.background_color.as_ref())
            .unwrap_or(palette.disabled_track);
    }
    resolved_style_color(node.button_style.element.background_color.as_ref())
        .unwrap_or(palette.track)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn progress_fill_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    let palette = material_feedback_palette();
    if node.disabled {
        return resolved_style_color(node.button_style.element.foreground_color.as_ref())
            .unwrap_or(palette.disabled_fill);
    }
    resolved_style_color(node.button_style.element.foreground_color.as_ref())
        .or_else(|| material_tone_color(node, &palette))
        .unwrap_or(palette.accent)
}

#[cfg(test)]
#[path = "tests/colors.rs"]
mod tests;
