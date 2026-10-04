//! 拖拽许可的呈现映射：正常与阻止状态共用宿主语义色，drop_allowed 来自上游落放判定。
//! 此层显示判定结果，不应自行推断权限或改变目标有效性。

use super::super::super::data::TemplatePaneNodeData;
use super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct DragOverlayPalette {
    pub preview_surface: [u8; 4],
    pub preview_surface_blocked: [u8; 4],
    pub preview_border: [u8; 4],
    pub preview_border_blocked: [u8; 4],
    pub preview_text: [u8; 4],
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn drag_overlay_palette(
) -> DragOverlayPalette {
    drag_overlay_palette_from_host(current_host_palette())
}

fn drag_overlay_palette_from_host(palette: HostMaterialPalette) -> DragOverlayPalette {
    DragOverlayPalette {
        preview_surface: palette.accent_soft,
        preview_surface_blocked: palette.error_container,
        preview_border: palette.accent,
        preview_border_blocked: palette.error,
        preview_text: palette.text,
    }
}

pub(super) fn preview_surface_color(
    node: &TemplatePaneNodeData,
    palette: DragOverlayPalette,
) -> [u8; 4] {
    if node.drop_allowed {
        palette.preview_surface
    } else {
        palette.preview_surface_blocked
    }
}

pub(super) fn preview_accent_color(
    node: &TemplatePaneNodeData,
    palette: DragOverlayPalette,
) -> [u8; 4] {
    if node.drop_allowed {
        palette.preview_border
    } else {
        palette.preview_border_blocked
    }
}

#[cfg(test)]
#[path = "tests/style.rs"]
mod tests;
