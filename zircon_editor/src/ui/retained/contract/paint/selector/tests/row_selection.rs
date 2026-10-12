use super::*;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

#[test]
fn row_selection_palette_projects_selected_outline_from_host_palette() {
    let mut palette = PALETTE;
    palette.accent = [10, 11, 12, 255];

    let selection_palette = workbench_row_selection_palette_from_host(palette);

    assert_eq!(
        selected_row_outline_color_from_palette(selection_palette),
        [10, 11, 12, 255]
    );
}
