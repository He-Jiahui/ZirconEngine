use super::*;

#[test]
fn sdf_atlas_fallback_overlays_failed_spans_for_a_materialized_wrapped_line() {
    let mut wrapped_line = text_batch("efgh");
    wrapped_line.frame = UiFrame::new(10.0, 20.0, 100.0, 24.0);
    wrapped_line.source_range = Some(UiTextRange { start: 4, end: 8 });
    wrapped_line.is_source_isomorphic_layout_line = true;
    wrapped_line.wrap = UiTextWrap::Word;
    let mut native_texts = Vec::new();
    let mut sdf_texts = vec![wrapped_line];

    let report = apply_sdf_atlas_fallbacks(
        &mut native_texts,
        &mut sdf_texts,
        &[SdfAtlasRun {
            glyph_slot_indices: vec![Some(0), None, Some(1), Some(2)],
            glyph_failure_reasons: vec![
                None,
                Some(SdfAtlasAllocationFailureReason::PageLimit),
                None,
                None,
            ],
            allocation_failure_count: 1,
            page_limit_failure_count: 1,
            oversized_failure_count: 0,
            ..Default::default()
        }],
        &[vec![5.0, 6.0, 7.0, 8.0]],
    );

    assert_eq!(native_texts.len(), 1);
    assert_eq!(native_texts[0].text, "f");
    assert_eq!(native_texts[0].frame, UiFrame::new(15.0, 20.0, 6.0, 24.0));
    assert_eq!(native_texts[0].wrap, UiTextWrap::None);
    assert_eq!(sdf_texts.len(), 1);
    assert_eq!(report.fallback_native_overlay_batch_count, 1);
    assert_eq!(report.whole_batch_fallback_text_batch_count, 0);
    assert_eq!(report.mixed_overlay_unsupported_wrap_text_batch_count, 0);
}

#[test]
fn sdf_atlas_fallback_keeps_raw_wrapped_text_as_a_whole_batch_fallback() {
    let mut raw_wrapped_text = text_batch("efgh");
    raw_wrapped_text.wrap = UiTextWrap::Word;
    let mut native_texts = Vec::new();
    let mut sdf_texts = vec![raw_wrapped_text];

    let report = apply_sdf_atlas_fallbacks(
        &mut native_texts,
        &mut sdf_texts,
        &[SdfAtlasRun {
            glyph_slot_indices: vec![Some(0), None, Some(1), Some(2)],
            glyph_failure_reasons: vec![
                None,
                Some(SdfAtlasAllocationFailureReason::PageLimit),
                None,
                None,
            ],
            allocation_failure_count: 1,
            page_limit_failure_count: 1,
            oversized_failure_count: 0,
            ..Default::default()
        }],
        &[vec![5.0, 6.0, 7.0, 8.0]],
    );

    assert_eq!(native_texts.len(), 1);
    assert_eq!(native_texts[0].text, "efgh");
    assert!(sdf_texts.is_empty());
    assert_eq!(report.fallback_native_overlay_batch_count, 0);
    assert_eq!(report.whole_batch_fallback_text_batch_count, 1);
    assert_eq!(report.mixed_overlay_unsupported_wrap_text_batch_count, 1);
}

#[test]
fn sdf_atlas_fallback_rejects_wrapped_visual_text_without_planner_provenance() {
    let mut virtual_wrapped_text = text_batch("\u{0640}");
    virtual_wrapped_text.source_range = Some(UiTextRange { start: 0, end: 2 });
    virtual_wrapped_text.wrap = UiTextWrap::Word;
    let mut native_texts = Vec::new();
    let mut sdf_texts = vec![virtual_wrapped_text];

    let report = apply_sdf_atlas_fallbacks(
        &mut native_texts,
        &mut sdf_texts,
        &[SdfAtlasRun {
            glyph_slot_indices: vec![None],
            glyph_failure_reasons: vec![Some(SdfAtlasAllocationFailureReason::PageLimit)],
            allocation_failure_count: 1,
            page_limit_failure_count: 1,
            oversized_failure_count: 0,
            ..Default::default()
        }],
        &[vec![12.0]],
    );

    assert_eq!(native_texts.len(), 1);
    assert_eq!(native_texts[0].text, "\u{0640}");
    assert!(sdf_texts.is_empty());
    assert_eq!(report.fallback_native_overlay_batch_count, 0);
    assert_eq!(report.whole_batch_fallback_text_batch_count, 1);
    assert_eq!(report.mixed_overlay_unsupported_wrap_text_batch_count, 1);
}
