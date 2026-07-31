use super::{resolve_vertical_flex_bands, VerticalFlexBandRequest};
use crate::ui::workbench::autolayout::{
    AxisConstraint, ShellSizePx, StretchMode, WorkbenchChromeMetrics,
};

fn stretch_band(min: f32, preferred: f32, weight: f32) -> AxisConstraint {
    AxisConstraint {
        min,
        max: -1.0,
        preferred,
        priority: 50,
        weight,
        stretch_mode: StretchMode::Stretch,
    }
}

#[test]
fn workbench_shell_geometry_vertical_flex_bands_fill_the_shell_with_token_gaps() {
    let metrics = WorkbenchChromeMetrics::default();
    let shell = ShellSizePx::new(900.0, 620.0);
    let bands = resolve_vertical_flex_bands(
        shell,
        VerticalFlexBandRequest::new(
            stretch_band(280.0, 420.0, 3.0),
            Some(stretch_band(120.0, 148.0, 1.0)),
            metrics,
        ),
    );

    assert_eq!(
        bands.center_band_frame.y,
        metrics.top_bar_height + metrics.host_bar_height + metrics.separator_thickness * 2.0
    );
    assert_eq!(
        bands.bottom_frame.y,
        bands.center_band_frame.y + bands.center_band_frame.height + metrics.separator_thickness
    );
    assert_eq!(
        bands.status_bar_frame.y + bands.status_bar_frame.height,
        shell.height
    );
}

#[test]
fn workbench_shell_geometry_vertical_flex_without_bottom_keeps_status_at_shell_edge() {
    let metrics = WorkbenchChromeMetrics::default();
    let shell = ShellSizePx::new(640.0, 420.0);
    let bands = resolve_vertical_flex_bands(
        shell,
        VerticalFlexBandRequest::new(stretch_band(180.0, 240.0, 1.0), None, metrics),
    );

    assert_eq!(
        bands.center_band_frame.y,
        metrics.top_bar_height + metrics.host_bar_height + metrics.separator_thickness * 2.0
    );
    assert_eq!(bands.bottom_frame.height, 0.0);
    assert_eq!(
        bands.status_bar_frame.y + bands.status_bar_frame.height,
        shell.height
    );
}
