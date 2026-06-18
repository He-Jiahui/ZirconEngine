use super::super::data::TemplatePaneNodeData;
use super::super::paint_theme::PALETTE;

pub(super) const AXIS_FIELD_BACKGROUND: [u8; 4] = [17, 22, 26, 255];
pub(super) const AXIS_FIELD_HOVER_BACKGROUND: [u8; 4] = [23, 30, 35, 255];
pub(super) const AXIS_FIELD_PRESSED_BACKGROUND: [u8; 4] = [18, 39, 47, 255];
pub(super) const AXIS_FIELD_DISABLED_BACKGROUND: [u8; 4] = [21, 25, 29, 255];
pub(super) const AXIS_FIELD_BORDER: [u8; 4] = [38, 48, 55, 255];
pub(super) const AXIS_FIELD_HOVER_BORDER: [u8; 4] = [56, 70, 79, 255];
pub(super) const AXIS_FIELD_DISABLED_BORDER: [u8; 4] = [42, 49, 55, 255];

pub(super) fn axis_field_background(node: &TemplatePaneNodeData) -> [u8; 4] {
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

pub(super) fn axis_field_border(node: &TemplatePaneNodeData) -> [u8; 4] {
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

pub(super) fn axis_field_border_width(node: &TemplatePaneNodeData) -> f32 {
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

pub(super) fn axis_field_text_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.disabled {
        PALETTE.text_disabled
    } else if matches!(node.validation_level.as_str(), "error" | "danger") {
        PALETTE.error
    } else if node.value_color.a > 0 {
        [
            node.value_color.r,
            node.value_color.g,
            node.value_color.b,
            node.value_color.a,
        ]
    } else {
        PALETTE.text
    }
}
