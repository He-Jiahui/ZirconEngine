use super::*;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

#[test]
fn mui_x_sparkline_color_projects_from_host_palette() {
    let mut palette = PALETTE;
    palette.accent = [10, 11, 12, 255];

    assert_eq!(sparkline_color_from_host(palette), [10, 11, 12, 255]);
}
