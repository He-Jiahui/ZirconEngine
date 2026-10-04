use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_palette, HostMaterialPalette,
};

use super::super::super::resolved_style_color;
use super::super::identity::chip_is_outlined;
use super::palette::{chip_color_token, chip_palette_main_from_host};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_foreground_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    chip_foreground_color_from_host(node, current_host_palette())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes::material_primitives::chip::style) fn chip_foreground_color_from_host(
    node: &TemplatePaneNodeData,
    palette: HostMaterialPalette,
) -> [u8; 4] {
    if node.disabled {
        return palette.text_disabled;
    }
    resolved_style_color(node.button_style.element.foreground_color.as_ref()).unwrap_or_else(|| {
        let color = chip_color_token(node);
        if chip_is_outlined(node) {
            chip_palette_main_from_host(color, palette).unwrap_or(palette.text)
        } else if color == "default" {
            palette.text
        } else {
            palette.shell_background
        }
    })
}

#[cfg(test)]
#[path = "tests/foreground.rs"]
mod tests;
