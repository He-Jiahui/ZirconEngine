use super::{translate_frame, TableAxes, TrackMetrics, UiFrame};
use crate::text::TextLayoutGeometryBudget;

fn budget() -> TextLayoutGeometryBudget {
    TextLayoutGeometryBudget::new(1_000.0, 4_000.0).expect("valid test budget")
}

#[test]
fn track_metrics_include_gap_in_origins_spans_and_total() {
    let metrics =
        TrackMetrics::new(vec![10.0, 20.0, 30.0], 2.0, budget()).expect("valid track geometry");

    assert_eq!(metrics.origin(0), Some(0.0));
    assert_eq!(metrics.origin(1), Some(12.0));
    assert_eq!(metrics.origin(2), Some(34.0));
    assert_eq!(metrics.span_extent(0, 1), 10.0);
    assert_eq!(metrics.span_extent(0, 2), 32.0);
    assert_eq!(metrics.span_extent(1, 2), 52.0);
    assert_eq!(metrics.total_extent(), 64.0);
}

#[test]
fn empty_and_clamped_track_queries_are_safe() {
    let empty = TrackMetrics::new(Vec::new(), 4.0, budget()).expect("valid empty track geometry");
    assert_eq!(empty.origin(0), None);
    assert_eq!(empty.span_extent(0, 1), 0.0);
    assert_eq!(empty.total_extent(), 0.0);

    let metrics = TrackMetrics::new(vec![10.0, 20.0], 3.0, budget()).expect("valid track geometry");
    assert_eq!(metrics.origin(2), None);
    assert_eq!(metrics.span_extent(1, usize::MAX), 20.0);
    assert_eq!(metrics.span_extent(2, 1), 0.0);
    assert_eq!(metrics.span_extent(0, 0), 0.0);
}

#[test]
fn gap_aware_metrics_map_consistently_across_writing_modes() {
    let columns =
        TrackMetrics::new(vec![10.0, 20.0, 30.0], 2.0, budget()).expect("valid column geometry");
    let rows = TrackMetrics::new(vec![5.0, 7.0], 1.0, budget()).expect("valid row geometry");
    let container = UiFrame::new(100.0, 200.0, 300.0, 400.0);
    let inline_start = columns.origin(1).unwrap();
    let block_start = rows.origin(0).unwrap();
    let inline_extent = columns.span_extent(1, 2);
    let block_extent = rows.span_extent(0, 2);

    assert_eq!(
        TableAxes::HorizontalTb.physical_frame(
            container,
            inline_start,
            block_start,
            inline_extent,
            block_extent,
        ),
        UiFrame::new(112.0, 200.0, 52.0, 13.0),
    );
    assert_eq!(
        TableAxes::VerticalRl.physical_frame(
            container,
            inline_start,
            block_start,
            inline_extent,
            block_extent,
        ),
        UiFrame::new(387.0, 212.0, 13.0, 52.0),
    );
}

#[test]
fn table_cell_translation_keeps_extreme_absolute_coordinates_finite() {
    let mut frame = UiFrame::new(f32::MAX, f32::MAX, 1.0, 1.0);

    translate_frame(&mut frame, f32::MAX, f32::MAX);

    assert_eq!(frame.x, f32::MAX);
    assert_eq!(frame.y, f32::MAX);
    assert!(frame.x.is_finite());
    assert!(frame.y.is_finite());
}
