use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::paint_theme::{HostMaterialPalette, current_host_palette};
use super::super::super::render_commands::HostPaintCommand;
use super::super::{component_variant_contains, push_quad};
use super::geometry::{picker_popup_cell_frame, picker_popup_frame, picker_popup_header_frame};
use super::metrics::PICKER_FIELD_RADIUS;

type PickerPopupColors = [[u8; 4]; 3];

pub(super) fn push_picker_popup_preview(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    field: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if !picker_popup_is_visible(node) {
        return;
    }
    let [popup_surface, header_surface, cell_surface] =
        picker_popup_colors_from_host(current_host_palette());
    let layout = picker_popup_frame(rect, field);
    push_quad(
        commands,
        layout.clone(),
        clip,
        order,
        popup_surface,
        0.0,
        PICKER_FIELD_RADIUS,
        opacity,
    );
    push_quad(
        commands,
        picker_popup_header_frame(&layout),
        clip,
        order + 1,
        header_surface,
        0.0,
        PICKER_FIELD_RADIUS,
        opacity,
    );
    let cell = picker_popup_cell_frame(&layout);
    push_quad(
        commands,
        cell.clone(),
        clip,
        order + 2,
        cell_surface,
        0.0,
        cell.width * 0.5,
        opacity,
    );
}

fn picker_popup_is_visible(node: &TemplatePaneNodeData) -> bool {
    node.popup_open || component_variant_contains(node, "desktop") || node.selected
}

fn picker_popup_colors_from_host(palette: HostMaterialPalette) -> PickerPopupColors {
    [palette.surface, palette.accent_soft, palette.accent_soft]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

    #[test]
    fn mui_x_picker_popup_colors_project_from_host_palette() {
        let mut palette = PALETTE;
        palette.surface = [10, 11, 12, 255];
        palette.accent_soft = [20, 21, 22, 255];

        assert_eq!(
            picker_popup_colors_from_host(palette),
            [[10, 11, 12, 255], [20, 21, 22, 255], [20, 21, 22, 255],]
        );
    }
}
