use super::super::super::data::TemplatePaneNodeData;
use super::colors::{
    AXIS_FIELD_BACKGROUND, AXIS_FIELD_DISABLED_BACKGROUND, AXIS_FIELD_HOVER_BACKGROUND,
    AXIS_FIELD_PRESSED_BACKGROUND,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn axis_field_background(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    if node.disabled {
        AXIS_FIELD_DISABLED_BACKGROUND
    } else if node.pressed {
        AXIS_FIELD_PRESSED_BACKGROUND
    } else if node.hovered || node.focused || node.selected {
        AXIS_FIELD_HOVER_BACKGROUND
    } else {
        AXIS_FIELD_BACKGROUND
    }
}
