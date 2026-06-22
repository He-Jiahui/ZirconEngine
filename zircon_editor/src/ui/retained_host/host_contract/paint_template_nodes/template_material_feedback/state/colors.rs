use super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::paint_theme::PALETTE;
use super::super::super::template_style_color::resolved_style_color;
use super::tone::material_tone_color;

const MATERIAL_PROGRESS_TRACK: [u8; 4] = [42, 52, 60, 255];

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn progress_track_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    if node.disabled {
        return PALETTE.surface_disabled;
    }
    resolved_style_color(node.button_style.element.background_color.as_ref())
        .unwrap_or(MATERIAL_PROGRESS_TRACK)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn progress_fill_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    if node.disabled {
        return PALETTE.text_disabled;
    }
    resolved_style_color(node.button_style.element.foreground_color.as_ref())
        .or_else(|| material_tone_color(node))
        .unwrap_or(PALETTE.accent)
}
