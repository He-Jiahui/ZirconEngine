use super::*;

#[test]
fn frame_run_index_preserves_empty_segments_and_global_run_bases() {
    let index =
        ScreenSpaceUiTextFrameRunIndex::from_segment_run_counts([[2, 3], [0, 1], [4, 0]]);

    assert_eq!(index.native_run_count(), 6);
    assert_eq!(index.sdf_run_count(), 4);
    assert_eq!(
        index.spans(),
        &[
            ScreenSpaceUiTextSegmentRunSpan {
                segment_index: 0,
                native_run_base: 0,
                native_run_count: 2,
                sdf_run_base: 0,
                sdf_run_count: 3,
            },
            ScreenSpaceUiTextSegmentRunSpan {
                segment_index: 1,
                native_run_base: 2,
                native_run_count: 0,
                sdf_run_base: 3,
                sdf_run_count: 1,
            },
            ScreenSpaceUiTextSegmentRunSpan {
                segment_index: 2,
                native_run_base: 2,
                native_run_count: 4,
                sdf_run_base: 4,
                sdf_run_count: 0,
            },
        ]
    );
}

#[test]
fn empty_frame_run_index_has_zero_counts_and_no_spans() {
    let index = ScreenSpaceUiTextFrameRunIndex::from_segment_run_counts([]);

    assert!(index.spans().is_empty());
    assert_eq!(index.native_run_count(), 0);
    assert_eq!(index.sdf_run_count(), 0);
}
