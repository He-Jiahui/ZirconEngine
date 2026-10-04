use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_palette, HostMaterialPalette,
};

use super::super::super::resolved_style_color;
use super::super::identity::chip_is_outlined;
use super::palette::{chip_color_token, chip_palette_main_from_host};

/// 显式背景优先；outlined 未声明背景时保持透明，其余颜色才从宿主色板补齐。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_background_color(
    node: &TemplatePaneNodeData,
) -> Option<[u8; 4]> {
    chip_background_color_from_host(node, current_host_palette())
}

fn chip_background_color_from_host(
    node: &TemplatePaneNodeData,
    palette: HostMaterialPalette,
) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.background_color.as_ref()).or_else(|| {
        if chip_is_outlined(node) {
            None
        } else {
            Some(
                chip_palette_main_from_host(chip_color_token(node), palette)
                    .unwrap_or(palette.surface_hover),
            )
        }
    })
}

#[cfg(test)]
#[path = "tests/background.rs"]
mod tests;
