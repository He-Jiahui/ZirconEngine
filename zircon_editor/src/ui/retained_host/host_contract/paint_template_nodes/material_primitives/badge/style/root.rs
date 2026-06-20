use super::super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::super::paint_theme::PALETTE;
use super::super::super::resolved_style_color;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn badge_root_background_color(
    node: &TemplatePaneNodeData,
) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.background_color.as_ref())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn badge_root_border_color(
    node: &TemplatePaneNodeData,
    border_width: f32,
) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.border_color.as_ref())
        .or_else(|| (border_width > 0.0).then_some(PALETTE.border))
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn badge_root_border_width(
    node: &TemplatePaneNodeData,
) -> f32 {
    node.button_style
        .element
        .border_width
        .max(node.border_width)
        .max(0.0)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn badge_root_text_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    if node.disabled {
        return PALETTE.text_disabled;
    }
    resolved_style_color(node.button_style.element.foreground_color.as_ref())
        .unwrap_or(PALETTE.text)
}
