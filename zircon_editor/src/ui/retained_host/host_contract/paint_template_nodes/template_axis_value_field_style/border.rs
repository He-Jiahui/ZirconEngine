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
