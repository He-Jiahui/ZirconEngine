use super::super::super::paint_theme::{HostMaterialPalette, current_host_palette};
use super::super::data::TemplatePaneNodeData;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct DragOverlayPalette {
    pub preview_surface: [u8; 4],
    pub preview_surface_blocked: [u8; 4],
    pub preview_border: [u8; 4],
    pub preview_border_blocked: [u8; 4],
    pub preview_text: [u8; 4],
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn drag_overlay_palette()
-> DragOverlayPalette {
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
mod tests {
    use super::super::super::super::paint_theme::PALETTE;
    use super::*;

    #[test]
    fn drag_overlay_palette_projects_from_host_material_roles() {
        let mut host = PALETTE;
        host.accent_soft = [1, 2, 3, 4];
        host.error_container = [5, 6, 7, 8];
        host.accent = [9, 10, 11, 12];
        host.error = [13, 14, 15, 16];
        host.text = [17, 18, 19, 20];

        let overlay = drag_overlay_palette_from_host(host);

        assert_eq!(overlay.preview_surface, [1, 2, 3, 4]);
        assert_eq!(overlay.preview_surface_blocked, [5, 6, 7, 8]);
        assert_eq!(overlay.preview_border, [9, 10, 11, 12]);
        assert_eq!(overlay.preview_border_blocked, [13, 14, 15, 16]);
        assert_eq!(overlay.preview_text, [17, 18, 19, 20]);
    }
}
