use super::geometry::vertical_scrollbar_geometry;
use super::style::{workbench_scrollbar_metrics_from_host, WorkbenchScrollbarMetrics};
use super::{asset::asset_tree_viewport_frame, hierarchy_content_extent};
use crate::ui::retained_host::hierarchy_pointer::constants::{ROW_GAP, ROW_HEIGHT, ROW_Y};
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_theme::METRICS;

#[test]
fn scrollbar_metrics_match_unreal_starship_baseline() {
    let metrics = workbench_scrollbar_metrics_from_host(METRICS);

    assert_eq!(metrics.thickness, 8.0);
    assert_eq!(metrics.radius, 4.0);
    assert_eq!(metrics.track_inset, 1.0);
    assert_eq!(metrics.min_thumb_length, 24.0);
}

#[test]
fn vertical_scrollbar_thumb_uses_relative_scroll_extent() {
    let viewport = frame(10.0, 20.0, 100.0, 200.0);
    let geometry = vertical_scrollbar_geometry(&viewport, 300.0, 800.0, test_metrics())
        .expect("overflowing content should draw a scrollbar");

    assert_eq!(geometry.track, frame(101.0, 21.0, 8.0, 198.0));
    assert!((geometry.thumb.x - 101.0).abs() < 0.001);
    assert!((geometry.thumb.y - 95.25).abs() < 0.001);
    assert!((geometry.thumb.width - 8.0).abs() < 0.001);
    assert!((geometry.thumb.height - 49.5).abs() < 0.001);
}

#[test]
fn vertical_scrollbar_clamps_thumb_to_track_end() {
    let viewport = frame(0.0, 0.0, 40.0, 100.0);
    let geometry = vertical_scrollbar_geometry(&viewport, 999.0, 300.0, test_metrics())
        .expect("overflowing content should draw a scrollbar");

    assert!((geometry.thumb.y + geometry.thumb.height - 99.0).abs() < 0.001);
}

#[test]
fn vertical_scrollbar_is_absent_when_content_fits() {
    let viewport = frame(0.0, 0.0, 80.0, 200.0);

    assert!(vertical_scrollbar_geometry(&viewport, 0.0, 200.0, test_metrics()).is_none());
    assert!(vertical_scrollbar_geometry(&viewport, 0.0, 120.0, test_metrics()).is_none());
}

#[test]
fn hierarchy_content_extent_uses_row_metrics_not_pixel_coordinates() {
    assert_eq!(hierarchy_content_extent(0), 0.0);
    assert_eq!(
        hierarchy_content_extent(3),
        ROW_Y * 2.0 + 3.0 * ROW_HEIGHT + 2.0 * ROW_GAP
    );
}

#[test]
fn asset_tree_viewport_tracks_pointer_tree_header_formula() {
    let body = frame(20.0, 30.0, 220.0, 260.0);
    let viewport = asset_tree_viewport_frame(&body);

    assert_eq!(viewport.x, 20.0);
    assert_eq!(viewport.y, 79.0);
    assert_eq!(viewport.width, 220.0);
    assert_eq!(viewport.height, 211.0);
}

fn test_metrics() -> WorkbenchScrollbarMetrics {
    WorkbenchScrollbarMetrics {
        thickness: 8.0,
        radius: 4.0,
        track_inset: 1.0,
        min_thumb_length: 24.0,
    }
}

fn frame(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    FrameRect {
        x,
        y,
        width,
        height,
    }
}
