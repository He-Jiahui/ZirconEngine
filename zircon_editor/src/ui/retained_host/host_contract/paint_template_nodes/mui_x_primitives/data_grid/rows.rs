use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};
use super::super::super::render_commands::HostPaintCommand;
use super::metrics::MUI_X_DATA_GRID_ROW_COUNT;

type DataGridRowColors = [[u8; 4]; 2];

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
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

    fn row_backgrounds_for_node(node: TemplatePaneNodeData) -> Vec<[u8; 4]> {
        let mut commands = Vec::new();
        let rect = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 120.0,
            height: 80.0,
        };
        push_data_grid_rows(&mut commands, &node, &rect, &rect, 10, 16.0, 18.0, 1.0);
        commands
            .iter()
            .filter_map(|command| command.background_color)
            .collect()
    }

    #[test]
    fn focused_data_grid_does_not_mark_first_row_selected() {
        let backgrounds = row_backgrounds_for_node(TemplatePaneNodeData {
            focused: true,
            ..TemplatePaneNodeData::default()
        });

        assert_eq!(backgrounds, vec![PALETTE.surface, PALETTE.surface]);
    }

    #[test]
    fn selected_data_grid_marks_first_row_selected() {
        let backgrounds = row_backgrounds_for_node(TemplatePaneNodeData {
            selected: true,
            ..TemplatePaneNodeData::default()
        });

        assert_eq!(backgrounds, vec![PALETTE.surface_selected, PALETTE.surface]);
    }

    #[test]
    fn mui_x_data_grid_row_colors_project_from_host_palette() {
        let mut palette = PALETTE;
        palette.surface = [10, 11, 12, 255];
        palette.surface_selected = [20, 21, 22, 255];

        assert_eq!(
            data_grid_row_colors_from_host(palette),
            [[10, 11, 12, 255], [20, 21, 22, 255]]
        );
    }
}
