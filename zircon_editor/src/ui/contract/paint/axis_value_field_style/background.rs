//! 轴值字段表面按 disabled、pressed、hovered/selected 顺序选择宿主主题色；focus 只改变边框。

use super::super::super::data::TemplatePaneNodeData;
use super::super::template_style_color::resolved_style_color;
use super::colors::{
    axis_field_disabled_background, axis_field_hover_background, axis_field_normal_background,
    axis_field_palette, axis_field_pressed_background,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn axis_field_background(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    axis_field_background_from_host(node, axis_field_palette())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn axis_field_background_from_host(
    node: &TemplatePaneNodeData,
    palette: super::super::super::paint_theme::HostMaterialPalette,
) -> [u8; 4] {
    if node.disabled {
        resolved_style_color(node.authored_field_colors.get("disabled_background_color"))
            .unwrap_or_else(|| axis_field_disabled_background(palette))
    } else if node.pressed {
        axis_field_pressed_background(palette)
    } else if node.hovered || node.selected {
        axis_field_hover_background(palette)
    } else {
        resolved_style_color(node.button_style.element.background_color.as_ref())
            .unwrap_or_else(|| axis_field_normal_background(palette))
    }
}
