use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_palette, HostMaterialPalette,
};

use super::super::identity::chip_is_outlined;
use super::foreground::chip_foreground_color_from_host;
use super::palette::{chip_color_token, chip_palette_main_from_host};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_delete_icon_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    chip_delete_icon_color_from_host(node, current_host_palette())
}

fn chip_delete_icon_color_from_host(
    node: &TemplatePaneNodeData,
    palette: HostMaterialPalette,
) -> [u8; 4] {
    if chip_is_outlined(node) {
        chip_palette_main_from_host(chip_color_token(node), palette).unwrap_or(palette.text_muted)
    } else {
        chip_foreground_color_from_host(node, palette)
    }
}

#[cfg(test)]
#[path = "tests/delete.rs"]
mod tests;
