use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_palette, HostMaterialPalette,
};

use super::palette::{chip_color_token, chip_palette_main_from_host};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_avatar_background_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    chip_avatar_background_color_from_host(node, current_host_palette())
}

fn chip_avatar_background_color_from_host(
    node: &TemplatePaneNodeData,
    palette: HostMaterialPalette,
) -> [u8; 4] {
    chip_palette_main_from_host(chip_color_token(node), palette).unwrap_or(palette.surface_selected)
}

#[cfg(test)]
#[path = "tests/avatar.rs"]
mod tests;
