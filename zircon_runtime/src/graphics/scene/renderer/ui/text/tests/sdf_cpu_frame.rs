use std::sync::Arc;

use zircon_runtime_interface::ui::event_ui::UiNodeId;
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiResolvedTextLine, UiTextAlign, UiTextDirection, UiTextRange, UiTextWrap,
    UiTextWritingMode,
};

use super::{replace_segment_output, text_cpu_inputs_match_iter, PreparedSdfCpuText};
use crate::core::framework::text::{TextGlyph, TextGlyphFlags, TextGlyphRotation};
use crate::graphics::scene::scene_renderer::ui::render::{
    ScreenSpaceUiGlyphArtifactLine, ScreenSpaceUiTextBatch, ScreenSpaceUiTextRouteIdentity,
};
use crate::text::sdf::SdfMode;
use crate::text::{ResolvedTextGlyphArtifact, ResolvedTextGlyphArtifactLine};

#[test]
fn cpu_segment_output_patch_preserves_unrelated_ranges() {
    let mut output = vec![10, 20, 21, 30];
    let mut ranges = vec![0..1, 1..3, 3..4];

    replace_segment_output(&mut output, &mut ranges, 1, &[80]);

    assert_eq!(output, vec![10, 80, 30]);
    assert_eq!(ranges, vec![0..1, 1..2, 2..3]);
}

#[test]
fn cpu_retained_local_patch_has_no_all_segment_scan() {
    let source = include_str!("../sdf_cpu_frame.rs");
    let local = source
        .split("pub(super) fn prepare_retained_frame")
        .nth(1)
        .expect("retained CPU frame prepare")
        .split("fn prepare_with_retained_generation")
        .next()
        .expect("retained CPU frame boundary");

    assert!(!local.contains("frame.segment_products().iter()"));
    assert!(local.contains("journal.changed_segment_indices()"));
    assert!(local.contains("frame.segment_products()[retained_segment_count..]"));
}

#[test]
fn cpu_snapshot_rejects_changed_text_owned_glyph_or_writing_mode() {
    let horizontal = artifact_text_batch(0xfb01, UiTextWritingMode::HorizontalTb);
    let prepared = PreparedSdfCpuText::from(&horizontal);

    let replacement = artifact_text_batch(0xfb02, UiTextWritingMode::HorizontalTb);
    assert!(!prepared.matches(&replacement));

    let mut vertical = horizontal.clone();
    vertical.writing_mode = UiTextWritingMode::VerticalRl;
    assert!(!prepared.matches(&vertical));

    let replacement = artifact_text_batch(0xfb02, UiTextWritingMode::HorizontalTb);
    let mut republished = horizontal.clone();
    let artifact_line = republished
        .glyph_artifact_line
        .as_mut()
        .expect("original artifact line");
    artifact_line.artifact = Arc::clone(
        &replacement
            .glyph_artifact_line
            .as_ref()
            .expect("replacement artifact line")
            .artifact,
    );
    Arc::make_mut(&mut artifact_line.artifact).font_generation = 8;
    artifact_line.font_generation = 8;
    assert!(!prepared.matches(&republished));
}

#[test]
fn cpu_snapshot_segment_stream_preserves_flat_order_and_change_detection() {
    let first = artifact_text_batch(0xfb01, UiTextWritingMode::HorizontalTb);
    let second = artifact_text_batch(0xfb02, UiTextWritingMode::HorizontalTb);
    let prepared = vec![
        PreparedSdfCpuText::from(&first),
        PreparedSdfCpuText::from(&second),
    ];
    let empty: &[ScreenSpaceUiTextBatch] = &[];
    let segments = [
        empty,
        std::slice::from_ref(&first),
        empty,
        std::slice::from_ref(&second),
    ];

    assert!(text_cpu_inputs_match_iter(
        &prepared,
        segments.into_iter().flatten(),
    ));

    let changed = artifact_text_batch(0xfb03, UiTextWritingMode::HorizontalTb);
    let changed_segments = [std::slice::from_ref(&first), std::slice::from_ref(&changed)];
    assert!(!text_cpu_inputs_match_iter(
        &prepared,
        changed_segments.into_iter().flatten(),
    ));
}

fn artifact_text_batch(glyph_id: u32, writing_mode: UiTextWritingMode) -> ScreenSpaceUiTextBatch {
    let frame = UiFrame::new(0.0, 0.0, 24.0, 24.0);
    ScreenSpaceUiTextBatch {
        route_identity: ScreenSpaceUiTextRouteIdentity::new(
            "runtime.sdf-cpu-frame.artifact.test",
            UiNodeId::new(1),
            None,
        ),
        command_generation: 1,
        raster_scale: 1.0,
        text: "fi".to_string(),
        frame,
        clip_frame: None,
        source_range: Some(UiTextRange { start: 0, end: 2 }),
        is_source_isomorphic_layout_line: false,
        glyph_advances: vec![24.0],
        shaped_glyphs: Vec::new(),
        preserve_shaped_glyphs: true,
        glyph_artifact_line: Some(ScreenSpaceUiGlyphArtifactLine {
            artifact: Arc::new(ResolvedTextGlyphArtifact {
                source_text: Arc::from("fi"),
                source_text_origin: 0,
                font_generation: 7,
                font_lease: crate::text::ResolvedTextGlyphArtifactFontLease::process_default(),
                style: UiResolvedStyle::default(),
                writing_mode,
                lines: vec![Some(ResolvedTextGlyphArtifactLine {
                    glyphs: vec![TextGlyph {
                        glyph_id,
                        source_range: 0..2,
                        visual_range: 0..1,
                        advance: 24.0,
                        position: [0.0, 0.0],
                        offset: [0.0, 0.0],
                        font_face: None,
                        font_instance: None,
                        rotation: TextGlyphRotation::None,
                        bidi_level: 0,
                        flags: TextGlyphFlags::default(),
                        requires_rasterization: true,
                    }],
                    layout_line: UiResolvedTextLine {
                        text: "fi".to_string(),
                        placement_frame: UiFrame::default(),
                        frame,
                        source_range: UiTextRange { start: 0, end: 2 },
                        visual_range: UiTextRange { start: 0, end: 1 },
                        measured_width: 24.0,
                        glyph_advances: vec![24.0],
                        baseline: 16.0,
                        direction: UiTextDirection::LeftToRight,
                        runs: Vec::new(),
                        ellipsized: false,
                    },
                })],
                logical_virtual_line_sequences: None,
            }),
            line_index: 0,
            font_generation: 7,
            glyph_range: 0..1,
        }),
        layout_error: None,
        color: [1.0, 1.0, 1.0, 1.0],
        background_color: None,
        font: None,
        font_family: None,
        language: None,
        font_weight: UiResolvedStyle::DEFAULT_FONT_WEIGHT,
        font_size: 16.0,
        line_height: 20.0,
        text_align: UiTextAlign::Left,
        text_direction: UiTextDirection::LeftToRight,
        writing_mode,
        wrap: UiTextWrap::None,
        style: Default::default(),
        distance_field_mode: SdfMode::Sdf,
        text_effects: Default::default(),
        text_decorations: Default::default(),
        text_decoration_baseline: None,
        clip_transform: None,
    }
}
