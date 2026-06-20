use super::super::super::data::TemplatePaneNodeData;
use super::super::super::paint_theme::PALETTE;
use super::colors::{AXIS_FIELD_BORDER, AXIS_FIELD_DISABLED_BORDER, AXIS_FIELD_HOVER_BORDER};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn axis_field_border(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    if node.disabled {
        AXIS_FIELD_DISABLED_BORDER
    } else if matches!(node.validation_level.as_str(), "error" | "danger") {
        PALETTE.error
    } else if matches!(node.validation_level.as_str(), "warning") {
        PALETTE.warning
    } else if node.focused || node.selected || node.pressed {
        PALETTE.focus_ring
    } else if node.hovered {
        AXIS_FIELD_HOVER_BORDER
    } else {
        AXIS_FIELD_BORDER
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn axis_field_border_width(
    node: &TemplatePaneNodeData,
) -> f32 {
    if node.focused
        || node.selected
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
