use super::*;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

#[test]
fn mui_x_line_chart_colors_project_from_host_palette() {
    let mut palette = PALETTE;
    palette.accent = [10, 11, 12, 255];
    palette.success = [20, 21, 22, 255];

    assert_eq!(
        line_chart_colors_from_host(palette),
        [[10, 11, 12, 255], [20, 21, 22, 255]]
    );
}

#[test]
fn mui_x_line_chart_points_project_from_percent_units() {
    assert_eq!(line_chart_point(PRIMARY_LINE_POINTS[0]), (0.08, 0.78));
    assert_eq!(line_chart_point(SECONDARY_LINE_POINTS[3]), (0.80, 0.50));
}
