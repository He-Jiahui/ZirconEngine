use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

use super::super::identity::chip_is_outlined;
use super::foreground::chip_foreground_color;
use super::palette::{chip_color_token, chip_palette_main};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_delete_icon_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    if chip_is_outlined(node) {
        chip_palette_main(chip_color_token(node)).unwrap_or(PALETTE.text_muted)
    } else {
        chip_foreground_color(node)
    }
}
