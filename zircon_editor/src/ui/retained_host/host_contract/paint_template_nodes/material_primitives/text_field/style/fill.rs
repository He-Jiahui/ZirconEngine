use super::super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};
use super::super::super::resolved_style_color;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const MUI_FIELD_FILLED_RADIUS: f32 = 4.0;

/// 禁用态使用宿主禁用色；其余状态才采用显式背景或 hover/默认色板。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn field_fill_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    field_fill_color_from_host(node, current_host_palette())
}

fn field_fill_color_from_host(
    node: &TemplatePaneNodeData,
    palette: HostMaterialPalette,
) -> [u8; 4] {
    if node.disabled {
        return palette.surface_disabled;
    }
    resolved_style_color(node.button_style.element.background_color.as_ref()).unwrap_or_else(|| {
        if node.hovered {
            palette.surface
        } else {
            palette.surface_inset
        }
    })
}

#[cfg(test)]
#[path = "tests/fill.rs"]
mod tests;
