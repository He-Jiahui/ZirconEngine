use super::*;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

#[test]
fn mui_x_shared_quad_border_projects_from_host_border_palette() {
    let mut palette = PALETTE;
    palette.border = [10, 11, 12, 255];
    palette.focus_ring = [90, 91, 92, 255];

    assert_eq!(
        quad_border_color_from_host(1.0, palette),
        Some([10, 11, 12, 255])
    );
}

#[test]
fn mui_x_shared_quad_border_stays_absent_without_width() {
    let mut palette = PALETTE;
    palette.focus_ring = [10, 11, 12, 255];

    assert_eq!(quad_border_color_from_host(0.0, palette), None);
}
