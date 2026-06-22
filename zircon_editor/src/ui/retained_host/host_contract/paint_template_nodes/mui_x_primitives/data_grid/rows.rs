use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::paint_theme::PALETTE;
use super::super::super::render_commands::HostPaintCommand;
use super::metrics::MUI_X_DATA_GRID_ROW_COUNT;

pub(super) fn push_data_grid_rows(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    first_row_y: f32,
    row_height: f32,
    opacity: f32,
) {
    for row in 0..MUI_X_DATA_GRID_ROW_COUNT {
        let selected = row == 0 && (node.selected || node.checked || node.focused);
        super::super::push_quad(
            commands,
            FrameRect {
                x: rect.x + 2.0,
                y: first_row_y + row as f32 * row_height,
                width: (rect.width - 4.0).max(1.0),
                height: (row_height - 1.0).max(1.0),
            },
            clip,
            order + 2 + row,
            if selected {
                PALETTE.surface_selected
            } else {
                PALETTE.surface
            },
            0.0,
            2.0,
            opacity,
        );
    }
}
