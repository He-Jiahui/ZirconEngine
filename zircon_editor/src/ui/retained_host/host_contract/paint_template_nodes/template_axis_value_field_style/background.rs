use super::super::super::data::TemplatePaneNodeData;
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
        axis_field_disabled_background(palette)
    } else if node.pressed {
        axis_field_pressed_background(palette)
    } else if node.hovered || node.selected {
        axis_field_hover_background(palette)
    } else {
        axis_field_normal_background(palette)
    }
}
