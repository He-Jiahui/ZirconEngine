use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

use super::super::super::resolved_style_color;
use super::super::identity::chip_is_outlined;
use super::palette::{chip_color_token, chip_palette_main};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_border_color(
    node: &TemplatePaneNodeData,
) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.border_color.as_ref()).or_else(|| {
        if chip_is_outlined(node) {
            Some(chip_palette_main(chip_color_token(node)).unwrap_or(PALETTE.border))
        } else if node.border_width > 0.0 || node.button_style.element.border_width > 0.0 {
            Some(PALETTE.border)
        } else {
            None
        }
    })
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_border_width(
    node: &TemplatePaneNodeData,
) -> f32 {
    node.button_style
        .element
        .border_width
        .max(node.border_width)
        .max(if chip_is_outlined(node) { 1.0 } else { 0.0 })
}
