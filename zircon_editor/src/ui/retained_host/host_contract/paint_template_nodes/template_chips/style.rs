use super::super::super::data::TemplatePaneNodeData;
use super::super::super::paint_theme::PALETTE;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const CHIP_FONT_SIZE: f32 =
    12.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const CHIP_LINE_HEIGHT: f32 =
    CHIP_FONT_SIZE * 1.2;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const CHIP_RADIUS: f32 = 5.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const CHIP_SURFACE: [u8; 4] =
    [31, 38, 44, 255];
const CHIP_HOVER_SURFACE: [u8; 4] = [38, 49, 56, 255];
const CHIP_PRESSED_SURFACE: [u8; 4] = [18, 52, 61, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const CHIP_BORDER: [u8; 4] =
    [48, 59, 66, 255];
const CHIP_TEXT: [u8; 4] = [211, 223, 228, 255];

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_surface(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    if node.disabled {
        PALETTE.surface_disabled
    } else if node.pressed || node.popup_open {
        CHIP_PRESSED_SURFACE
    } else if node.hovered || node.focused {
        CHIP_HOVER_SURFACE
    } else {
        CHIP_SURFACE
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_border(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    if node.disabled {
        PALETTE.border_disabled
    } else if node.focused || node.pressed || node.popup_open {
        PALETTE.focus_ring
    } else {
        CHIP_BORDER
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_text_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    if node.disabled {
        PALETTE.text_disabled
    } else if matches!(node.text_tone.as_str(), "muted" | "subtle") {
        PALETTE.text_muted
    } else {
        CHIP_TEXT
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_glyph_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    if node.disabled {
        PALETTE.text_disabled
    } else if node.focused || node.pressed || node.popup_open {
        PALETTE.focus_ring
    } else {
        PALETTE.text_muted
    }
}
