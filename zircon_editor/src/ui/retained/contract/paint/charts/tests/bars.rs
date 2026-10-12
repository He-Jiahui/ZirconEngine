use super::*;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

#[test]
fn mui_x_chart_bar_colors_project_from_host_palette() {
    let mut palette = PALETTE;
    palette.accent = [10, 11, 12, 255];
    palette.success = [20, 21, 22, 255];
    palette.warning = [30, 31, 32, 255];

    assert_eq!(
        chart_bar_colors_from_host(palette),
        [[10, 11, 12, 255], [20, 21, 22, 255], [30, 31, 32, 255]]
    );
}
