use std::sync::Arc;

use zircon_runtime_interface::ui::event_ui::UiNodeId;
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiResolvedTextLine, UiTextAlign, UiTextDirection, UiTextRange, UiTextWrap,
    UiTextWritingMode,
};

use super::{collect_sdf_atlas_text_keys, collect_sdf_atlas_text_keys_iter};
use crate::core::framework::text::{
    TextFontCollectionHandle, TextFontFaceHandle, TextGlyph, TextGlyphFlags, TextGlyphRotation,
};
use crate::graphics::scene::scene_renderer::ui::render::{
    ScreenSpaceUiGlyphArtifactLine, ScreenSpaceUiTextBatch, ScreenSpaceUiTextRouteIdentity,
};
use crate::text::sdf::SdfMode;
use crate::text::{ResolvedTextGlyphArtifact, ResolvedTextGlyphArtifactLine};

const TEST_FONT_COLLECTION: TextFontCollectionHandle = TextFontCollectionHandle::new(1);

#[test]
fn glyph_artifact_ligature_keys_match_the_text_owned_glyph_line() {
    let artifact = Arc::new(ResolvedTextGlyphArtifact {
        source_text: Arc::from("fi"),
        source_text_origin: 0,
        font_generation: 7,
        font_lease: crate::text::ResolvedTextGlyphArtifactFontLease::process_default(),
        style: UiResolvedStyle::default(),
        writing_mode: UiTextWritingMode::HorizontalTb,
        lines: vec![Some(ResolvedTextGlyphArtifactLine {
            glyphs: vec![TextGlyph {
                glyph_id: 0xfb01,
                source_range: 0..2,
                visual_range: 0..1,
                advance: 12.0,
                position: [0.0, 0.0],
                offset: [0.0, 0.0],
                font_face: Some(TextFontFaceHandle::new(TEST_FONT_COLLECTION, 3, 5)),
                font_instance: Some(TextFontFaceHandle::new(TEST_FONT_COLLECTION, 4, 6)),
                rotation: TextGlyphRotation::None,
                bidi_level: 0,
                flags: TextGlyphFlags::default(),
                requires_rasterization: true,
            }],
            layout_line: UiResolvedTextLine {
                text: "fi".to_string(),
                placement_frame: UiFrame::default(),
                frame: UiFrame::new(0.0, 0.0, 12.0, 20.0),
                source_range: UiTextRange { start: 0, end: 2 },
                visual_range: UiTextRange { start: 0, end: 1 },
                measured_width: 12.0,
                glyph_advances: vec![12.0],
                baseline: 16.0,
                direction: UiTextDirection::LeftToRight,
                runs: Vec::new(),
                ellipsized: false,
            },
        })],
        logical_virtual_line_sequences: None,
    });
    let text = ScreenSpaceUiTextBatch {
        route_identity: ScreenSpaceUiTextRouteIdentity::new(
            "runtime.sdf-atlas.artifact-key.test",
            UiNodeId::new(1),
            None,
        ),
        command_generation: 1,
        raster_scale: 1.0,
        text: "fi".to_string(),
        frame: UiFrame::new(0.0, 0.0, 12.0, 20.0),
        clip_frame: None,
        source_range: Some(UiTextRange { start: 0, end: 2 }),
        is_source_isomorphic_layout_line: false,
        glyph_advances: vec![12.0],
        shaped_glyphs: Vec::new(),
        preserve_shaped_glyphs: true,
        glyph_artifact_line: Some(ScreenSpaceUiGlyphArtifactLine {
            artifact,
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
        writing_mode: UiTextWritingMode::HorizontalTb,
        wrap: UiTextWrap::None,
        style: Default::default(),
        distance_field_mode: SdfMode::Sdf,
        text_effects: Default::default(),
        text_decorations: Default::default(),
        text_decoration_baseline: None,
        clip_transform: None,
    };

    let (unique_keys, runs) = collect_sdf_atlas_text_keys(std::slice::from_ref(&text));
    let empty: &[ScreenSpaceUiTextBatch] = &[];
    let segments = [empty, std::slice::from_ref(&text), empty];
    let (segmented_unique_keys, segmented_runs) =
        collect_sdf_atlas_text_keys_iter(segments.into_iter().flatten());

    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].len(), 1);
    let key = runs[0][0].as_ref().expect("ligature needs an atlas key");
    assert_eq!(key.glyph, 'f');
    assert_eq!(key.glyph_id, Some(0xfb01));
    assert_eq!(
        key.font_id,
        Some(TextFontFaceHandle::new(TEST_FONT_COLLECTION, 3, 5))
    );
    assert_eq!(
        key.font_instance_id,
        Some(TextFontFaceHandle::new(TEST_FONT_COLLECTION, 4, 6))
    );
    assert_eq!(unique_keys.len(), 1);
    assert_eq!(segmented_unique_keys, unique_keys);
    assert_eq!(segmented_runs, runs);
}
