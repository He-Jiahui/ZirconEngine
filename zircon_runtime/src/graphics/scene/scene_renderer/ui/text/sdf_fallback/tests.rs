use super::*;
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiTextAlign, UiTextDirection, UiTextWrap, UiTextWritingMode,
};

#[test]
fn sdf_fallback_keeps_stable_frame_vectors_in_place_when_all_runs_are_renderable() {
    let mut native_texts = vec![text_batch("Native")];
    let mut sdf_texts = vec![text_batch("Stable")];
    let mut native_decoration_metrics = vec![TextDecorationMetrics::default()];
    let mut cpu_runs = vec![SdfRunCpuPreparation::default()];
    let native_ptr = native_texts.as_ptr();
    let sdf_ptr = sdf_texts.as_ptr();
    let metrics_ptr = native_decoration_metrics.as_ptr();
    let cpu_ptr = cpu_runs.as_ptr();

    let report = apply_sdf_atlas_fallbacks_with_cpu_runs(
        &mut native_texts,
        &mut sdf_texts,
        &[SdfAtlasRun::default()],
        &mut cpu_runs,
        &mut native_decoration_metrics,
    );

    assert_eq!(report, ScreenSpaceUiTextSdfFallbackReport::default());
    assert_eq!(native_texts.as_ptr(), native_ptr);
    assert_eq!(sdf_texts.as_ptr(), sdf_ptr);
    assert_eq!(native_decoration_metrics.as_ptr(), metrics_ptr);
    assert_eq!(cpu_runs.as_ptr(), cpu_ptr);
}

#[test]
fn sdf_fallback_retains_cpu_artifacts_for_surviving_runs_without_reprepare() {
    let mut native_texts = Vec::new();
    let mut sdf_texts = vec![text_batch("Stable"), text_batch("Fallback")];
    let mut native_decoration_metrics = Vec::new();
    let mut cpu_runs = vec![
        SdfRunCpuPreparation {
            glyph_advances: vec![11.0],
            ..Default::default()
        },
        SdfRunCpuPreparation {
            glyph_advances: vec![22.0],
            ..Default::default()
        },
    ];

    let report = apply_sdf_atlas_fallbacks_with_cpu_runs(
        &mut native_texts,
        &mut sdf_texts,
        &[SdfAtlasRun::default()],
        &mut cpu_runs,
        &mut native_decoration_metrics,
    );

    assert_eq!(report.whole_batch_fallback_text_batch_count, 1);
    assert_eq!(sdf_texts.len(), 1);
    assert_eq!(native_texts.len(), 1);
    assert_eq!(cpu_runs.len(), 1);
    assert_eq!(cpu_runs[0].glyph_advances, vec![11.0]);
    assert_eq!(native_decoration_metrics.len(), 1);
}

#[test]
fn sdf_atlas_fallback_moves_failed_sdf_batches_to_native_backend() {
    let native = text_batch("Native");
    let stable_sdf = text_batch("SdfOk");
    let failed_sdf = text_batch("SdfFail");
    let mut native_texts = vec![native];
    let mut sdf_texts = vec![stable_sdf, failed_sdf];

    let report = apply_sdf_atlas_fallbacks(
        &mut native_texts,
        &mut sdf_texts,
        &[
            SdfAtlasRun {
                glyph_slot_indices: vec![Some(0), Some(1), Some(2), Some(3), Some(4)],
                ..Default::default()
            },
            SdfAtlasRun {
                glyph_slot_indices: vec![Some(5), None, None, Some(6)],
                glyph_failure_reasons: vec![
                    None,
                    Some(SdfAtlasAllocationFailureReason::PageLimit),
                    Some(SdfAtlasAllocationFailureReason::OversizedSlot),
                    None,
                ],
                allocation_failure_count: 2,
                page_limit_failure_count: 1,
                oversized_failure_count: 1,
                ..Default::default()
            },
        ],
        &[],
    );

    assert_eq!(
        native_texts
            .iter()
            .map(|text| text.text.as_str())
            .collect::<Vec<_>>(),
        vec!["Native", "SdfFail"]
    );
    assert_eq!(
        sdf_texts
            .iter()
            .map(|text| text.text.as_str())
            .collect::<Vec<_>>(),
        vec!["SdfOk"]
    );
    assert_eq!(
        report,
        ScreenSpaceUiTextSdfFallbackReport {
            fallback_text_batch_count: 1,
            whole_batch_fallback_text_batch_count: 1,
            fallback_native_overlay_batch_count: 0,
            mixed_overlay_unsupported_text_batch_count: 1,
            mixed_overlay_empty_span_text_batch_count: 0,
            mixed_overlay_missing_advances_text_batch_count: 1,
            mixed_overlay_unsupported_writing_mode_text_batch_count: 0,
            mixed_overlay_unsupported_text_direction_text_batch_count: 0,
            mixed_overlay_unsupported_wrap_text_batch_count: 0,
            mixed_overlay_unsupported_justify_text_batch_count: 0,
            mixed_overlay_glyph_advance_mismatch_text_batch_count: 0,
            mixed_overlay_invalid_span_text_batch_count: 0,
            fallback_glyph_count: 2,
            fallback_span_count: 2,
            fallback_source_byte_count: 2,
            page_limit_glyph_count: 1,
            oversized_glyph_count: 1,
            page_limit_span_count: 1,
            oversized_span_count: 1,
            page_limit_source_byte_count: 1,
            oversized_source_byte_count: 1,
        }
    );
}

#[test]
fn sdf_atlas_fallback_overlays_failed_spans_for_horizontal_ltr_text() {
    let native = text_batch("Native");
    let mut mixed_sdf = text_batch("abcdef");
    mixed_sdf.frame = UiFrame::new(10.0, 20.0, 100.0, 24.0);
    let mut native_texts = vec![native];
    let mut sdf_texts = vec![mixed_sdf];

    let report = apply_sdf_atlas_fallbacks(
        &mut native_texts,
        &mut sdf_texts,
        &[SdfAtlasRun {
            glyph_slot_indices: vec![Some(0), None, None, Some(1), Some(2), Some(3)],
            glyph_failure_reasons: vec![
                None,
                Some(SdfAtlasAllocationFailureReason::PageLimit),
                Some(SdfAtlasAllocationFailureReason::PageLimit),
                None,
                None,
                None,
            ],
            allocation_failure_count: 2,
            page_limit_failure_count: 2,
            oversized_failure_count: 0,
            ..Default::default()
        }],
        &[vec![5.0, 6.0, 7.0, 8.0, 9.0, 10.0]],
    );

    assert_eq!(
        native_texts
            .iter()
            .map(|text| text.text.as_str())
            .collect::<Vec<_>>(),
        vec!["Native", "bc"]
    );
    assert_eq!(
        sdf_texts
            .iter()
            .map(|text| text.text.as_str())
            .collect::<Vec<_>>(),
        vec!["abcdef"]
    );
    assert_eq!(native_texts[1].frame, UiFrame::new(15.0, 20.0, 13.0, 24.0));
    assert_eq!(native_texts[1].text_align, UiTextAlign::Left);
    assert_eq!(native_texts[1].wrap, UiTextWrap::None);
    assert_eq!(
        report,
        ScreenSpaceUiTextSdfFallbackReport {
            fallback_text_batch_count: 1,
            whole_batch_fallback_text_batch_count: 0,
            fallback_native_overlay_batch_count: 1,
            mixed_overlay_unsupported_text_batch_count: 0,
            mixed_overlay_empty_span_text_batch_count: 0,
            mixed_overlay_missing_advances_text_batch_count: 0,
            mixed_overlay_unsupported_writing_mode_text_batch_count: 0,
            mixed_overlay_unsupported_text_direction_text_batch_count: 0,
            mixed_overlay_unsupported_wrap_text_batch_count: 0,
            mixed_overlay_unsupported_justify_text_batch_count: 0,
            mixed_overlay_glyph_advance_mismatch_text_batch_count: 0,
            mixed_overlay_invalid_span_text_batch_count: 0,
            fallback_glyph_count: 2,
            fallback_span_count: 1,
            fallback_source_byte_count: 2,
            page_limit_glyph_count: 2,
            oversized_glyph_count: 0,
            page_limit_span_count: 1,
            oversized_span_count: 0,
            page_limit_source_byte_count: 2,
            oversized_source_byte_count: 0,
        }
    );
}

#[test]
fn sdf_atlas_fallback_overlays_failed_spans_for_horizontal_rtl_text() {
    let mut mixed_sdf = text_batch("abcd");
    mixed_sdf.frame = UiFrame::new(10.0, 20.0, 100.0, 24.0);
    mixed_sdf.text_direction = UiTextDirection::RightToLeft;
    mixed_sdf.text_align = UiTextAlign::Start;
    let mut native_texts = Vec::new();
    let mut sdf_texts = vec![mixed_sdf];

    let report = apply_sdf_atlas_fallbacks(
        &mut native_texts,
        &mut sdf_texts,
        &[SdfAtlasRun {
            glyph_slot_indices: vec![Some(0), None, None, Some(1)],
            glyph_failure_reasons: vec![
                None,
                Some(SdfAtlasAllocationFailureReason::PageLimit),
                Some(SdfAtlasAllocationFailureReason::PageLimit),
                None,
            ],
            allocation_failure_count: 2,
            page_limit_failure_count: 2,
            oversized_failure_count: 0,
            ..Default::default()
        }],
        &[vec![5.0, 6.0, 7.0, 8.0]],
    );

    assert_eq!(native_texts.len(), 1);
    assert_eq!(native_texts[0].text, "bc");
    assert_eq!(native_texts[0].frame, UiFrame::new(89.0, 20.0, 13.0, 24.0));
    assert_eq!(native_texts[0].text_align, UiTextAlign::Left);
    assert_eq!(native_texts[0].text_direction, UiTextDirection::RightToLeft);
    assert_eq!(native_texts[0].wrap, UiTextWrap::None);
    assert_eq!(
        sdf_texts
            .iter()
            .map(|text| text.text.as_str())
            .collect::<Vec<_>>(),
        vec!["abcd"]
    );
    assert_eq!(report.fallback_native_overlay_batch_count, 1);
    assert_eq!(report.whole_batch_fallback_text_batch_count, 0);
    assert_eq!(
        report.mixed_overlay_unsupported_text_direction_text_batch_count,
        0
    );
    assert_eq!(report.page_limit_span_count, 1);
    assert_eq!(report.page_limit_source_byte_count, 2);
}

#[test]
fn sdf_atlas_fallback_resolves_auto_direction_for_horizontal_rtl_overlay() {
    let mut mixed_sdf = text_batch("\u{05D0}\u{05D1}cd");
    mixed_sdf.frame = UiFrame::new(10.0, 20.0, 100.0, 24.0);
    mixed_sdf.text_direction = UiTextDirection::Auto;
    mixed_sdf.text_align = UiTextAlign::Start;
    let mut native_texts = Vec::new();
    let mut sdf_texts = vec![mixed_sdf];

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
    assert_eq!(native_texts[0].text, "\u{05D1}");
    assert_eq!(native_texts[0].frame, UiFrame::new(89.0, 20.0, 6.0, 24.0));
    assert_eq!(native_texts[0].text_align, UiTextAlign::Left);
    assert_eq!(native_texts[0].text_direction, UiTextDirection::RightToLeft);
    assert_eq!(
        sdf_texts
            .iter()
            .map(|text| text.text.as_str())
            .collect::<Vec<_>>(),
        vec!["\u{05D0}\u{05D1}cd"]
    );
    assert_eq!(report.fallback_native_overlay_batch_count, 1);
    assert_eq!(report.whole_batch_fallback_text_batch_count, 0);
    assert_eq!(
        report.mixed_overlay_unsupported_text_direction_text_batch_count,
        0
    );
}

#[test]
fn sdf_atlas_fallback_overlays_whole_grapheme_from_resolved_advances() {
    let mut mixed_sdf = text_batch("e\u{301}A");
    mixed_sdf.frame = UiFrame::new(10.0, 20.0, 100.0, 24.0);
    let mut native_texts = Vec::new();
    let mut sdf_texts = vec![mixed_sdf];

    let report = apply_sdf_atlas_fallbacks(
        &mut native_texts,
        &mut sdf_texts,
        &[SdfAtlasRun {
            glyph_slot_indices: vec![Some(0), None, Some(1)],
            glyph_failure_reasons: vec![
                None,
                Some(SdfAtlasAllocationFailureReason::PageLimit),
                None,
            ],
            allocation_failure_count: 1,
            page_limit_failure_count: 1,
            oversized_failure_count: 0,
            ..Default::default()
        }],
        &[vec![20.0, 10.0]],
    );

    assert_eq!(native_texts.len(), 1);
    assert_eq!(native_texts[0].text, "e\u{301}");
    assert_eq!(native_texts[0].frame, UiFrame::new(10.0, 20.0, 20.0, 24.0));
    assert_eq!(
        sdf_texts
            .iter()
            .map(|text| text.text.as_str())
            .collect::<Vec<_>>(),
        vec!["e\u{301}A"]
    );
    assert_eq!(report.fallback_native_overlay_batch_count, 1);
    assert_eq!(report.whole_batch_fallback_text_batch_count, 0);
    assert_eq!(
        report.mixed_overlay_glyph_advance_mismatch_text_batch_count,
        0
    );
    assert_eq!(report.page_limit_span_count, 1);
    assert_eq!(report.page_limit_source_byte_count, "e\u{301}".len());
}

#[test]
fn sdf_atlas_fallback_reports_unsupported_mixed_overlay_layout_reason() {
    let mut vertical_sdf = text_batch("abcd");
    vertical_sdf.writing_mode = UiTextWritingMode::VerticalRl;
    let mut native_texts = Vec::new();
    let mut sdf_texts = vec![vertical_sdf];

    let report = apply_sdf_atlas_fallbacks(
        &mut native_texts,
        &mut sdf_texts,
        &[SdfAtlasRun {
            glyph_slot_indices: vec![Some(0), None, None, Some(1)],
            glyph_failure_reasons: vec![
                None,
                Some(SdfAtlasAllocationFailureReason::PageLimit),
                Some(SdfAtlasAllocationFailureReason::PageLimit),
                None,
            ],
            allocation_failure_count: 2,
            page_limit_failure_count: 2,
            oversized_failure_count: 0,
            ..Default::default()
        }],
        &[vec![5.0, 6.0, 7.0, 8.0]],
    );

    assert_eq!(
        native_texts
            .iter()
            .map(|text| text.text.as_str())
            .collect::<Vec<_>>(),
        vec!["abcd"]
    );
    assert!(sdf_texts.is_empty());
    assert_eq!(report.fallback_text_batch_count, 1);
    assert_eq!(report.whole_batch_fallback_text_batch_count, 1);
    assert_eq!(report.fallback_native_overlay_batch_count, 0);
    assert_eq!(report.mixed_overlay_unsupported_text_batch_count, 1);
    assert_eq!(
        report.mixed_overlay_unsupported_writing_mode_text_batch_count,
        1
    );
    assert_eq!(report.page_limit_span_count, 1);
    assert_eq!(report.page_limit_source_byte_count, 2);
}

#[test]
fn sdf_atlas_fallback_rejects_ambiguous_horizontal_text_direction() {
    let mut mixed_direction_sdf = text_batch("abcd");
    mixed_direction_sdf.text_direction = UiTextDirection::Mixed;
    let mut native_texts = Vec::new();
    let mut sdf_texts = vec![mixed_direction_sdf];

    let report = apply_sdf_atlas_fallbacks(
        &mut native_texts,
        &mut sdf_texts,
        &[SdfAtlasRun {
            glyph_slot_indices: vec![Some(0), None, None, Some(1)],
            glyph_failure_reasons: vec![
                None,
                Some(SdfAtlasAllocationFailureReason::PageLimit),
                Some(SdfAtlasAllocationFailureReason::PageLimit),
                None,
            ],
            allocation_failure_count: 2,
            page_limit_failure_count: 2,
            oversized_failure_count: 0,
            ..Default::default()
        }],
        &[vec![5.0, 6.0, 7.0, 8.0]],
    );

    assert_eq!(
        native_texts
            .iter()
            .map(|text| text.text.as_str())
            .collect::<Vec<_>>(),
        vec!["abcd"]
    );
    assert!(sdf_texts.is_empty());
    assert_eq!(report.fallback_text_batch_count, 1);
    assert_eq!(report.whole_batch_fallback_text_batch_count, 1);
    assert_eq!(report.fallback_native_overlay_batch_count, 0);
    assert_eq!(report.mixed_overlay_unsupported_text_batch_count, 1);
    assert_eq!(
        report.mixed_overlay_unsupported_text_direction_text_batch_count,
        1
    );
}

#[test]
fn sdf_atlas_fallback_groups_failed_glyph_reason_spans() {
    let run = SdfAtlasRun {
        glyph_failure_reasons: vec![
            None,
            Some(SdfAtlasAllocationFailureReason::PageLimit),
            Some(SdfAtlasAllocationFailureReason::PageLimit),
            None,
            Some(SdfAtlasAllocationFailureReason::OversizedSlot),
            Some(SdfAtlasAllocationFailureReason::OversizedSlot),
            Some(SdfAtlasAllocationFailureReason::PageLimit),
        ],
        allocation_failure_count: 5,
        page_limit_failure_count: 3,
        oversized_failure_count: 2,
        ..Default::default()
    };

    assert_eq!(
        fallback_spans_for_text_run("abcdefg", &run),
        vec![
            SdfAtlasGlyphFallbackSpan {
                start_glyph_index: 1,
                glyph_count: 2,
                start_byte_index: 1,
                end_byte_index: 3,
                reason: SdfGlyphFallbackReason::Allocation(
                    SdfAtlasAllocationFailureReason::PageLimit,
                ),
            },
            SdfAtlasGlyphFallbackSpan {
                start_glyph_index: 4,
                glyph_count: 2,
                start_byte_index: 4,
                end_byte_index: 6,
                reason: SdfGlyphFallbackReason::Allocation(
                    SdfAtlasAllocationFailureReason::OversizedSlot,
                ),
            },
            SdfAtlasGlyphFallbackSpan {
                start_glyph_index: 6,
                glyph_count: 1,
                start_byte_index: 6,
                end_byte_index: 7,
                reason: SdfGlyphFallbackReason::Allocation(
                    SdfAtlasAllocationFailureReason::PageLimit,
                ),
            },
        ]
    );
}

#[test]
fn sdf_atlas_fallback_maps_failed_glyph_spans_to_utf8_byte_ranges() {
    let run = SdfAtlasRun {
        glyph_failure_reasons: vec![
            None,
            Some(SdfAtlasAllocationFailureReason::PageLimit),
            Some(SdfAtlasAllocationFailureReason::PageLimit),
            None,
            Some(SdfAtlasAllocationFailureReason::OversizedSlot),
        ],
        allocation_failure_count: 3,
        page_limit_failure_count: 2,
        oversized_failure_count: 1,
        ..Default::default()
    };

    assert_eq!(
        fallback_spans_for_text_run("A中🙂BΩ", &run),
        vec![
            SdfAtlasGlyphFallbackSpan {
                start_glyph_index: 1,
                glyph_count: 2,
                start_byte_index: 1,
                end_byte_index: 8,
                reason: SdfGlyphFallbackReason::Allocation(
                    SdfAtlasAllocationFailureReason::PageLimit,
                ),
            },
            SdfAtlasGlyphFallbackSpan {
                start_glyph_index: 4,
                glyph_count: 1,
                start_byte_index: 9,
                end_byte_index: 11,
                reason: SdfGlyphFallbackReason::Allocation(
                    SdfAtlasAllocationFailureReason::OversizedSlot,
                ),
            },
        ]
    );
}

#[test]
fn sdf_atlas_generation_failure_uses_normal_native_overlay_path() {
    let mut native_texts = Vec::new();
    let mut sdf_texts = vec![text_batch("abc")];
    let run = SdfAtlasRun {
        glyph_slot_indices: vec![Some(0), Some(1), Some(2)],
        glyph_generation_failures: vec![
            None,
            Some(SdfGlyphGenerationError::MissingGlyphOutline(42)),
            None,
        ],
        generation_failure_count: 1,
        ..Default::default()
    };

    let report = apply_sdf_atlas_fallbacks(
        &mut native_texts,
        &mut sdf_texts,
        &[run],
        &[vec![5.0, 6.0, 7.0]],
    );

    assert_eq!(native_texts.len(), 1);
    assert_eq!(native_texts[0].text, "b");
    assert_eq!(native_texts[0].frame, UiFrame::new(5.0, 0.0, 6.0, 24.0));
    assert_eq!(sdf_texts.len(), 1);
    assert_eq!(sdf_texts[0].text, "abc");
    assert_eq!(report.fallback_text_batch_count, 1);
    assert_eq!(report.fallback_native_overlay_batch_count, 1);
    assert_eq!(report.whole_batch_fallback_text_batch_count, 0);
    assert_eq!(report.fallback_glyph_count, 1);
    assert_eq!(report.fallback_span_count, 1);
    assert_eq!(report.fallback_source_byte_count, 1);
    assert_eq!(report.page_limit_glyph_count, 0);
    assert_eq!(report.oversized_glyph_count, 0);
}

fn text_batch(text: &str) -> ScreenSpaceUiTextBatch {
    ScreenSpaceUiTextBatch {
        route_identity:
            crate::graphics::scene::scene_renderer::ui::render::ScreenSpaceUiTextRouteIdentity::new(
                "runtime.sdf-fallback.test",
                zircon_runtime_interface::ui::event_ui::UiNodeId::new(1),
                None,
            ),
        command_generation: 1,
        text: text.to_string(),
        frame: UiFrame::new(0.0, 0.0, 128.0, 24.0),
        clip_frame: None,
        source_range: None,
        glyph_advances: Vec::new(),
        shaped_glyphs: Vec::new(),
        preserve_shaped_glyphs: false,
        glyph_artifact_line: None,
        layout_error: None,
        color: [1.0, 1.0, 1.0, 1.0],
        background_color: None,
        font: Some("res://fonts/default.font.toml".to_string()),
        font_family: Some("Zircon Sans".to_string()),
        language: None,
        font_weight: zircon_runtime_interface::ui::surface::UiResolvedStyle::DEFAULT_FONT_WEIGHT,
        font_size: 16.0,
        line_height: 20.0,
        text_align: UiTextAlign::Left,
        text_direction: UiTextDirection::LeftToRight,
        writing_mode: UiTextWritingMode::HorizontalTb,
        wrap: UiTextWrap::None,
        style: Default::default(),
        distance_field_mode: crate::text::sdf::SdfMode::Sdf,
        text_effects: Default::default(),
        text_decorations: Default::default(),
        text_decoration_baseline: None,
        clip_transform: None,
    }
}
