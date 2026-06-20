use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

use super::super::super::resolved_style_color;
use super::super::identity::chip_is_outlined;
use super::palette::{chip_color_token, chip_palette_main, MUI_ON_DARK, MUI_ON_WARNING};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_foreground_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    if node.disabled {
        return PALETTE.text_disabled;
    }
    resolved_style_color(node.button_style.element.foreground_color.as_ref()).unwrap_or_else(|| {
        let color = chip_color_token(node);
        if chip_is_outlined(node) {
            chip_palette_main(color).unwrap_or(PALETTE.text)
        } else if color == "warning" {
            MUI_ON_WARNING
        } else if color == "default" {
            PALETTE.text
        } else {
            MUI_ON_DARK
        }
    })
}
