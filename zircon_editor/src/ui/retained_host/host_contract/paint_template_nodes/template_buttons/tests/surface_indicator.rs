use super::super::super::super::paint_theme::{METRICS, PALETTE};
use super::*;

#[test]
fn button_surface_indicator_metrics_project_from_host_control_metrics() {
    let mut host = METRICS;
    host.tab_underline_height = 3.0;
    host.button_pad_x = 10.0;

    let metrics = button_surface_indicator_metrics_from_host(host);

    assert_eq!(metrics.underline_height, 3.0);
    assert_eq!(metrics.asset_browser_tab_inset_x, 10.0);
}

#[test]
fn button_surface_indicator_palette_projects_from_host_palette() {
    let mut host = PALETTE;
    host.accent = [1, 2, 3, 4];

    let palette = button_surface_indicator_palette_from_host(host);

    assert_eq!(palette.underline, [1, 2, 3, 4]);
}
