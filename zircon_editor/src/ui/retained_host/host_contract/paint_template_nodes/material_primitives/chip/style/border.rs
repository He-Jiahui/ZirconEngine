use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_palette, HostMaterialPalette,
};

use super::super::super::resolved_style_color;
use super::super::identity::chip_is_outlined;
use super::palette::{chip_color_token, chip_palette_main_from_host};

/// outlined 从颜色词元取得轮廓，显式边框色始终可覆盖；普通 Chip 只在声明边框时使用宿主边色。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_border_color(
    node: &TemplatePaneNodeData,
) -> Option<[u8; 4]> {
    chip_border_color_from_host(node, current_host_palette())
}

fn chip_border_color_from_host(
    node: &TemplatePaneNodeData,
    palette: HostMaterialPalette,
) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.border_color.as_ref()).or_else(|| {
        if chip_is_outlined(node) {
            Some(
                chip_palette_main_from_host(chip_color_token(node), palette)
                    .unwrap_or(palette.border),
            )
        } else if node.border_width > 0.0 || node.button_style.element.border_width > 0.0 {
            Some(palette.border)
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

#[cfg(test)]
#[path = "tests/border.rs"]
mod tests;
