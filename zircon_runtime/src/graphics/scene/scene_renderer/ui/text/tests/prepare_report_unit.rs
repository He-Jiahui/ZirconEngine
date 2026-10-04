use super::*;

#[test]
fn resolved_text_report_merges_segment_counts_and_payload_sizes() {
    let mut frame = ScreenSpaceUiResolvedTextReport {
        native_text_batch_count: 2,
        sdf_text_batch_count: 1,
        batch_residency: ScreenSpaceUiTextBatchResidencyReport {
            materialized_batch_count: 3,
            text_byte_count: 12,
            glyph_advance_byte_count: 8,
        },
        post_layout_stale_artifact_batch_rejection_count: 1,
        layout_fallbacks: TextLayoutFallbackReport::default(),
    };
    let mut segment_fallbacks = TextLayoutFallbackReport::default();
    segment_fallbacks.fallback_count = 2;
    segment_fallbacks.invalid_language_count = 1;

    frame.merge(ScreenSpaceUiResolvedTextReport {
        native_text_batch_count: 1,
        sdf_text_batch_count: 4,
        batch_residency: ScreenSpaceUiTextBatchResidencyReport {
            materialized_batch_count: 5,
            text_byte_count: 20,
            glyph_advance_byte_count: 16,
        },
        post_layout_stale_artifact_batch_rejection_count: 3,
        layout_fallbacks: segment_fallbacks,
    });

    assert_eq!(frame.native_text_batch_count, 3);
    assert_eq!(frame.sdf_text_batch_count, 5);
    assert_eq!(frame.batch_residency.materialized_batch_count, 8);
    assert_eq!(frame.batch_residency.text_byte_count, 32);
    assert_eq!(frame.batch_residency.glyph_advance_byte_count, 24);
    assert_eq!(frame.post_layout_stale_artifact_batch_rejection_count, 4);
    assert_eq!(frame.layout_fallbacks.fallback_count, 2);
    assert_eq!(frame.layout_fallbacks.invalid_language_count, 1);
}
