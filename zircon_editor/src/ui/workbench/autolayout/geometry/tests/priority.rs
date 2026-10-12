use super::*;
use crate::ui::workbench::autolayout::StretchMode;

fn stretch_band(min: f32, preferred: f32) -> AxisConstraint {
    AxisConstraint {
        min,
        max: -1.0,
        preferred,
        priority: 50,
        weight: 1.0,
        stretch_mode: StretchMode::Stretch,
    }
}

fn priority_bands(height: f32, with_bottom: bool) -> (PriorityBandHeights, VerticalFlexBands) {
    let metrics = WorkbenchChromeMetrics::default();
    let request = VerticalFlexBandRequest::new(
        stretch_band(40.0, 80.0),
        with_bottom.then(|| stretch_band(60.0, 90.0)),
        metrics,
    );
    let minimums = priority_band_minimums(&request);
    let resolved = resolve_vertical_flex_bands(ShellSizePx::new(640.0, height), request);
    (minimums, resolved)
}

#[test]
fn astra_editor_layout_short_heights_follow_lexicographic_band_priority() {
    let metrics = WorkbenchChromeMetrics::default();
    let separator = metrics.separator_thickness;
    let (minimums, _) = priority_bands(0.0, true);
    let top_end = minimums.top;
    let host_start = top_end + separator + 1.0;
    let host_end = top_end + separator + minimums.host;
    let center_start = host_end + separator + 1.0;

    for height in [
        0.0,
        1.0,
        23.0,
        24.0,
        25.0,
        56.0,
        57.0,
        58.0,
        80.0,
        81.0,
        82.0,
        83.0,
        120.0,
        420.0,
        host_start - 1.0,
        host_start,
        host_end,
        center_start,
    ] {
        let (_, bands) = priority_bands(height, true);
        assert!(bands.center_band_frame.y.is_finite());
        assert!(bands.center_band_frame.height.is_finite());
        assert!(bands.bottom_frame.height.is_finite());
        assert!(bands.status_bar_frame.height.is_finite());
        assert!(bands.center_band_frame.y >= 0.0);
        assert!(bands.center_band_frame.y + bands.center_band_frame.height <= height + 0.001);
        assert!(bands.bottom_frame.y + bands.bottom_frame.height <= height + 0.001);
        assert!(bands.status_bar_frame.y + bands.status_bar_frame.height <= height + 0.001);
    }

    assert_eq!(
        priority_bands(host_start - 1.0, true).1.center_band_frame.y,
        top_end
    );
    assert_eq!(
        priority_bands(host_start, true).1.center_band_frame.y,
        host_start
    );
    assert_eq!(
        priority_bands(center_start, true)
            .1
            .center_band_frame
            .height,
        1.0
    );
}

#[test]
fn astra_editor_layout_bottom_strip_precedes_status_when_height_is_exhausted() {
    let metrics = WorkbenchChromeMetrics::default();
    let (minimums, _) = priority_bands(0.0, true);
    let separator = metrics.separator_thickness;
    let center_end = minimums.top + separator + minimums.host + separator + minimums.center;
    let bottom_start = center_end + separator + 1.0;
    let bottom_end = center_end + separator + minimums.bottom;
    let status_start = bottom_end + separator + 1.0;

    assert_eq!(
        priority_bands(bottom_start - 1.0, true)
            .1
            .bottom_frame
            .height,
        0.0
    );
    assert_eq!(
        priority_bands(bottom_start, true).1.bottom_frame.height,
        1.0
    );
    assert_eq!(
        priority_bands(status_start - 1.0, true)
            .1
            .status_bar_frame
            .height,
        0.0
    );
    assert_eq!(
        priority_bands(status_start, true).1.status_bar_frame.height,
        1.0
    );
}

#[test]
fn astra_editor_layout_zero_bands_do_not_consume_separators() {
    let (_, bands) = priority_bands(1.0, true);
    assert_eq!(bands.center_band_frame.y, 1.0);
    assert_eq!(bands.center_band_frame.height, 0.0);
    assert_eq!(bands.bottom_frame.y, 1.0);
    assert_eq!(bands.status_bar_frame.y, 1.0);

    let (_, without_bottom) = priority_bands(83.0, false);
    assert_eq!(without_bottom.bottom_frame.height, 0.0);
    assert!(without_bottom.status_bar_frame.y + without_bottom.status_bar_frame.height <= 83.001);
}

#[test]
fn astra_editor_layout_normal_height_keeps_existing_solver_contract() {
    let metrics = WorkbenchChromeMetrics::default();
    for height in [420.0, 620.0] {
        let request = VerticalFlexBandRequest::new(
            stretch_band(180.0, 240.0),
            Some(stretch_band(120.0, 148.0)),
            metrics,
        );
        let bands = resolve_vertical_flex_bands(ShellSizePx::new(900.0, height), request);
        assert_eq!(
            bands.center_band_frame.y,
            metrics.top_bar_height + metrics.host_bar_height + metrics.separator_thickness * 2.0
        );
        assert_eq!(
            bands.status_bar_frame.y + bands.status_bar_frame.height,
            height
        );
    }
}
