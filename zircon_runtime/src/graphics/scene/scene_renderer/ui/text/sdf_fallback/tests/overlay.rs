use std::sync::Arc;

use super::super::*;
use super::text_batch;
use crate::core::framework::text::TextLayoutError;
use crate::graphics::scene::scene_renderer::ui::render::{
    ScreenSpaceUiGlyphArtifactLine, ScreenSpaceUiShapedGlyph,
    text_decorations::ScreenSpaceUiTextDecorations,
    text_effects::{ScreenSpaceUiTextEffects, ScreenSpaceUiTextGlow, ScreenSpaceUiTextOutline},
    text_projection::ScreenSpaceUiTextClipTransform,
};
use crate::text::{ResolvedTextGlyphArtifact, ShapedGlyphRotation};
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiTextDirection, UiTextRange, UiTextRunPaintStyle, UiTextWritingMode,
};

#[test]
fn sdf_atlas_fallback_overlay_discards_source_layout_metadata() {
    let mut mixed_sdf = text_batch("abcdef");
    mixed_sdf.raster_scale = 1.5;
    mixed_sdf.source_range = Some(UiTextRange { start: 48, end: 54 });
    mixed_sdf.glyph_advances = vec![5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    mixed_sdf.shaped_glyphs = vec![ScreenSpaceUiShapedGlyph {
        glyph_id: 42,
        font_id: None,
        font_instance_id: None,
        source_scalar: 'a',
        source_range: UiTextRange { start: 48, end: 49 },
        advance: 5.0,
        offset_x: 0.0,
        offset_y: 0.0,
        rotation: ShapedGlyphRotation::None,
        requires_atlas_slot: true,
    }];
    mixed_sdf.preserve_shaped_glyphs = true;
    mixed_sdf.glyph_artifact_line = Some(artifact_line_for_test("abcdef", 48));
    mixed_sdf.layout_error = Some(TextLayoutError::LayoutFailed);
    mixed_sdf.clip_frame = Some(UiFrame::new(1.0, 2.0, 64.0, 20.0));
    mixed_sdf.background_color = Some([0.1, 0.2, 0.3, 0.4]);
    mixed_sdf.style = UiTextRunPaintStyle {
        strong: true,
        emphasis: true,
        code: false,
    };
    mixed_sdf.text_effects = ScreenSpaceUiTextEffects {
        outline: Some(ScreenSpaceUiTextOutline {
            width_px: 1.5,
            color: [0.1, 0.2, 0.3, 0.4],
        }),
        shadow: None,
        glow: Some(ScreenSpaceUiTextGlow {
            radius_px: 3.0,
            color: [0.5, 0.6, 0.7, 0.8],
        }),
    };
    mixed_sdf.text_decorations = ScreenSpaceUiTextDecorations {
        underline: true,
        strikethrough: true,
        underline_color: [0.2, 0.3, 0.4, 0.5],
        strikethrough_color: [0.6, 0.7, 0.8, 0.9],
    };
    mixed_sdf.text_decoration_baseline = Some(13.0);
    mixed_sdf.clip_transform = Some(ScreenSpaceUiTextClipTransform::from_rows([
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.25, 0.5, 0.0, 1.0],
    ]));
    let expected_clip_frame = mixed_sdf.clip_frame;
    let expected_background_color = mixed_sdf.background_color;
    let expected_style = mixed_sdf.style;
    let expected_text_effects = mixed_sdf.text_effects;
    let expected_text_decorations = mixed_sdf.text_decorations;
    let expected_text_decoration_baseline = mixed_sdf.text_decoration_baseline;
    let expected_clip_transform = mixed_sdf.clip_transform;
    let expected_raster_scale = mixed_sdf.raster_scale;
    let overlay = mixed_sdf.native_fallback_overlay(
        "bc".to_string(),
        UiFrame::new(5.0, 0.0, 13.0, 24.0),
        UiTextDirection::LeftToRight,
    );
    assert_eq!(overlay.text, "bc");
    assert_eq!(overlay.source_range, None);
    assert!(overlay.glyph_advances.is_empty());
    assert!(overlay.shaped_glyphs.is_empty());
    assert!(!overlay.preserve_shaped_glyphs);
    assert!(overlay.glyph_artifact_line.is_none());
    assert_eq!(overlay.layout_error, None);
    assert_eq!(overlay.clip_frame, expected_clip_frame);
    assert_eq!(overlay.background_color, expected_background_color);
    assert_eq!(overlay.style, expected_style);
    assert_eq!(overlay.text_effects, expected_text_effects);
    assert_eq!(overlay.text_decorations, expected_text_decorations);
    assert_eq!(
        overlay.text_decoration_baseline,
        expected_text_decoration_baseline
    );
    assert_eq!(overlay.clip_transform, expected_clip_transform);
    assert_eq!(overlay.raster_scale, expected_raster_scale);
}

#[test]
fn sdf_atlas_fallback_reuses_cpu_run_for_repeated_native_overlay() {
    let source_sdf = text_batch("abcdef");
    let native_prefix = text_batch("prefix");
    let native_suffix = text_batch("suffix");
    let atlas_run = SdfAtlasRun {
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
        ..Default::default()
    };
    let mut cpu_runs = vec![SdfRunCpuPreparation {
        glyph_advances: vec![5.0, 6.0, 7.0, 8.0, 9.0, 10.0],
        decoration_metrics: TextDecorationMetrics {
            ascender_px: 9.0,
            ..Default::default()
        },
        ..Default::default()
    }];
    let native_prefix_metric = TextDecorationMetrics {
        ascender_px: 3.0,
        ..Default::default()
    };
    let native_suffix_metric = TextDecorationMetrics {
        ascender_px: 4.0,
        ..Default::default()
    };
    let stale_overlay_metric = TextDecorationMetrics {
        ascender_px: 99.0,
        ..Default::default()
    };
    let mut native_decoration_metrics = vec![
        native_prefix_metric,
        native_suffix_metric,
        stale_overlay_metric,
    ];

    let mut native_texts = vec![native_prefix.clone(), native_suffix.clone()];
    let mut sdf_texts = vec![source_sdf.clone()];
    let first = apply_sdf_atlas_fallbacks_with_cpu_runs(
        &mut native_texts,
        &mut sdf_texts,
        std::slice::from_ref(&atlas_run),
        &mut cpu_runs,
        &mut native_decoration_metrics,
    );

    assert_eq!(native_texts.len(), 3);
    assert_eq!(
        native_texts
            .iter()
            .map(|text| text.text.as_str())
            .collect::<Vec<_>>(),
        vec!["prefix", "suffix", "bc"]
    );
    assert_eq!(sdf_texts.len(), 1);
    assert_eq!(sdf_texts[0].text, source_sdf.text);
    assert_eq!(cpu_runs.len(), 1);
    assert_eq!(
        native_decoration_metrics,
        vec![
            native_prefix_metric,
            native_suffix_metric,
            cpu_runs[0].decoration_metrics,
        ]
    );
    assert!(!first.needs_sdf_cpu_rebuild());

    // A new render frame starts from the original explicit native texts. The cached SDF run
    // remains valid, while the previous overlay's dynamic decoration metric must be replaced.
    native_texts = vec![native_prefix, native_suffix];
    sdf_texts = vec![source_sdf];
    let second = apply_sdf_atlas_fallbacks_with_cpu_runs(
        &mut native_texts,
        &mut sdf_texts,
        std::slice::from_ref(&atlas_run),
        &mut cpu_runs,
        &mut native_decoration_metrics,
    );

    assert_eq!(native_texts.len(), 3);
    assert_eq!(
        native_texts
            .iter()
            .map(|text| text.text.as_str())
            .collect::<Vec<_>>(),
        vec!["prefix", "suffix", "bc"]
    );
    assert_eq!(sdf_texts.len(), 1);
    assert_eq!(cpu_runs.len(), 1);
    assert_eq!(
        native_decoration_metrics,
        vec![
            native_prefix_metric,
            native_suffix_metric,
            cpu_runs[0].decoration_metrics,
        ]
    );
    assert!(!second.needs_sdf_cpu_rebuild());

    // Once the fallback disappears, no dynamic overlay metric may remain in the cached frame.
    native_texts = vec![text_batch("prefix"), text_batch("suffix")];
    sdf_texts = vec![text_batch("abcdef")];
    let stable = apply_sdf_atlas_fallbacks_with_cpu_runs(
        &mut native_texts,
        &mut sdf_texts,
        &[SdfAtlasRun::default()],
        &mut cpu_runs,
        &mut native_decoration_metrics,
    );

    assert_eq!(
        native_decoration_metrics,
        vec![native_prefix_metric, native_suffix_metric]
    );
    assert_eq!(
        native_texts
            .iter()
            .map(|text| text.text.as_str())
            .collect::<Vec<_>>(),
        vec!["prefix", "suffix"]
    );
    assert!(!stable.needs_sdf_cpu_rebuild());
}

#[test]
fn sdf_atlas_fallback_promotes_shaped_ligature_to_whole_native() {
    let mut shaped_sdf = text_batch("fi");
    shaped_sdf.shaped_glyphs = vec![ScreenSpaceUiShapedGlyph {
        glyph_id: 0xfb01,
        font_id: None,
        font_instance_id: None,
        source_scalar: 'f',
        source_range: UiTextRange { start: 0, end: 2 },
        advance: 12.0,
        offset_x: 0.0,
        offset_y: 0.0,
        rotation: ShapedGlyphRotation::None,
        requires_atlas_slot: true,
    }];
    let mut native_texts = Vec::new();
    let mut sdf_texts = vec![shaped_sdf];
    let mut cpu_runs = vec![SdfRunCpuPreparation {
        glyph_advances: vec![6.0, 6.0],
        ..Default::default()
    }];
    let mut native_decoration_metrics = Vec::new();

    let report = apply_sdf_atlas_fallbacks_with_cpu_runs(
        &mut native_texts,
        &mut sdf_texts,
        &[SdfAtlasRun {
            glyph_slot_indices: vec![None],
            glyph_failure_reasons: vec![Some(SdfAtlasAllocationFailureReason::PageLimit)],
            allocation_failure_count: 1,
            page_limit_failure_count: 1,
            ..Default::default()
        }],
        &mut cpu_runs,
        &mut native_decoration_metrics,
    );

    assert_eq!(native_texts.len(), 1);
    assert_eq!(native_texts[0].text, "fi");
    assert!(sdf_texts.is_empty());
    assert!(cpu_runs.is_empty());
    assert_eq!(native_decoration_metrics.len(), 1);
    assert_eq!(report.fallback_native_overlay_batch_count, 0);
    assert_eq!(report.whole_batch_fallback_text_batch_count, 1);
    assert_eq!(
        report.mixed_overlay_shaped_glyph_geometry_text_batch_count,
        1
    );
    assert_eq!(report.fallback_glyph_count, 1);
    assert_eq!(report.fallback_span_count, 0);
    assert_eq!(report.fallback_source_byte_count, 0);
    assert!(report.needs_sdf_cpu_rebuild());
}

#[test]
fn sdf_atlas_fallback_promotes_glyph_artifact_to_whole_native() {
    let mut artifact_sdf = text_batch("fi");
    artifact_sdf.glyph_artifact_line = Some(artifact_line_for_test("fi", 0));
    let mut native_texts = Vec::new();
    let mut sdf_texts = vec![artifact_sdf];
    let mut cpu_runs = vec![SdfRunCpuPreparation {
        glyph_advances: vec![6.0, 6.0],
        ..Default::default()
    }];
    let mut native_decoration_metrics = Vec::new();

    let report = apply_sdf_atlas_fallbacks_with_cpu_runs(
        &mut native_texts,
        &mut sdf_texts,
        &[SdfAtlasRun {
            glyph_slot_indices: vec![None],
            glyph_failure_reasons: vec![Some(SdfAtlasAllocationFailureReason::PageLimit)],
            allocation_failure_count: 1,
            page_limit_failure_count: 1,
            ..Default::default()
        }],
        &mut cpu_runs,
        &mut native_decoration_metrics,
    );

    assert_eq!(native_texts.len(), 1);
    assert_eq!(native_texts[0].text, "fi");
    assert!(sdf_texts.is_empty());
    assert!(cpu_runs.is_empty());
    assert_eq!(native_decoration_metrics.len(), 1);
    assert_eq!(report.fallback_native_overlay_batch_count, 0);
    assert_eq!(report.whole_batch_fallback_text_batch_count, 1);
    assert_eq!(
        report.mixed_overlay_shaped_glyph_geometry_text_batch_count,
        1
    );
    assert_eq!(report.mixed_overlay_empty_span_text_batch_count, 0);
    assert_eq!(report.fallback_glyph_count, 1);
    assert_eq!(report.fallback_span_count, 0);
    assert_eq!(report.fallback_source_byte_count, 0);
    assert!(report.needs_sdf_cpu_rebuild());
}

fn artifact_line_for_test(
    source_text: &str,
    source_text_origin: usize,
) -> ScreenSpaceUiGlyphArtifactLine {
    ScreenSpaceUiGlyphArtifactLine {
        artifact: Arc::new(ResolvedTextGlyphArtifact {
            source_text: Arc::from(source_text),
            source_text_origin,
            font_generation: 7,
            font_lease: crate::text::ResolvedTextGlyphArtifactFontLease::process_default(),
            style: UiResolvedStyle::default(),
            writing_mode: UiTextWritingMode::HorizontalTb,
            lines: Vec::new(),
            logical_virtual_line_sequences: None,
        }),
        line_index: 0,
        font_generation: 7,
        glyph_range: 0..0,
    }
}
