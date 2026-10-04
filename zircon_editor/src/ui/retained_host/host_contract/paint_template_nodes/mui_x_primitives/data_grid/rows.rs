use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};
use super::super::super::render_commands::HostPaintCommand;
use super::metrics::MUI_X_DATA_GRID_ROW_COUNT;

type DataGridRowColors = [[u8; 4]; 2];

/// 固定预览行共享网格根框；只有选中或勾选才高亮首行，根焦点不会把首行误标为已选。
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
    let [row_surface, selected_surface] = data_grid_row_colors_from_host(current_host_palette());
    commands.reserve(MUI_X_DATA_GRID_ROW_COUNT as usize);
    for row in 0..MUI_X_DATA_GRID_ROW_COUNT {
        let selected = row == 0 && (node.selected || node.checked);
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
                selected_surface
            } else {
                row_surface
            },
            0.0,
            2.0,
            opacity,
        );
    }
}

fn data_grid_row_colors_from_host(palette: HostMaterialPalette) -> DataGridRowColors {
    [palette.surface, palette.surface_selected]
}

#[cfg(test)]
#[path = "tests/rows.rs"]
mod tests;
