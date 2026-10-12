use super::*;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

#[test]
fn mui_x_data_grid_surface_colors_project_from_host_palette() {
    let mut palette = PALETTE;
    palette.surface_inset = [10, 11, 12, 255];
    palette.surface_hover = [20, 21, 22, 255];

    assert_eq!(
        data_grid_surface_color_from_host(&TemplatePaneNodeData::default(), palette),
        [10, 11, 12, 255]
    );
    assert_eq!(data_grid_header_color_from_host(palette), [20, 21, 22, 255]);
}
