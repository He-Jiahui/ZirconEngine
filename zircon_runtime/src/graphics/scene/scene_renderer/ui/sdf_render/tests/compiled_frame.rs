use std::sync::Arc;

use super::super::compiled_frame::PreparedSdfFrameInputs;
use super::text_batch;
use crate::core::framework::text::{TextGlyph, TextGlyphFlags, TextGlyphRotation};
use crate::core::math::UVec2;
use crate::graphics::scene::scene_renderer::ui::render::ScreenSpaceUiGlyphArtifactLine;
use crate::text::{ResolvedTextGlyphArtifact, ResolvedTextGlyphArtifactLine};
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiResolvedTextLine, UiTextDirection, UiTextRange, UiTextWritingMode,
};

#[test]
fn compiled_frame_rebuilds_when_refreshed_artifact_line_changes() {
    let mut prepared_text = text_batch("A", UiFrame::new(0.0, 0.0, 24.0, 24.0));
    let artifact = artifact();
    prepared_text.glyph_artifact_line = Some(artifact_line(Arc::clone(&artifact), [0.0, 0.0]));

    let mut refreshed_text = prepared_text.clone();
    refreshed_text.glyph_artifact_line = Some(artifact_line(artifact, [3.0, 0.0]));

    let mut inputs = PreparedSdfFrameInputs::default();
    inputs.replace(
        UVec2::new(96, 96),
        std::slice::from_ref(&prepared_text),
        &[],
        &[],
        &[],
    );

    assert!(!inputs.matches(
        UVec2::new(96, 96),
        std::slice::from_ref(&refreshed_text),
        &[],
        &[],
        &[],
    ));
}

#[test]
fn compiled_frame_rebuilds_when_source_range_changes() {
    let mut prepared_text = text_batch("fi", UiFrame::new(0.0, 0.0, 24.0, 24.0));
    prepared_text.source_range = Some(UiTextRange { start: 0, end: 2 });

    let mut rebased_text = prepared_text.clone();
    rebased_text.source_range = Some(UiTextRange { start: 12, end: 14 });

    let mut inputs = PreparedSdfFrameInputs::default();
    inputs.replace(
        UVec2::new(96, 96),
        std::slice::from_ref(&prepared_text),
        &[],
        &[],
        &[],
    );

    assert!(!inputs.matches(
        UVec2::new(96, 96),
        std::slice::from_ref(&rebased_text),
        &[],
        &[],
        &[],
    ));
}

fn artifact() -> Arc<ResolvedTextGlyphArtifact> {
    Arc::new(ResolvedTextGlyphArtifact {
        source_text: Arc::from("A"),
        source_text_origin: 0,
        font_generation: 7,
        style: UiResolvedStyle::default(),
        writing_mode: UiTextWritingMode::HorizontalTb,
        lines: Vec::new(),
    })
}

fn artifact_line(
    artifact: Arc<ResolvedTextGlyphArtifact>,
    offset: [f32; 2],
) -> ScreenSpaceUiGlyphArtifactLine {
    ScreenSpaceUiGlyphArtifactLine {
        artifact,
        line_index: 0,
        refreshed_line: Some(Arc::new(ResolvedTextGlyphArtifactLine {
            glyphs: vec![TextGlyph {
                glyph_id: 65,
                source_range: 0..1,
                visual_range: 0..1,
                advance: 12.0,
                position: [0.0, 0.0],
                offset,
                font_face: None,
                font_instance: None,
                rotation: TextGlyphRotation::None,
                bidi_level: 0,
                flags: TextGlyphFlags::default(),
                requires_rasterization: true,
            }],
            layout_line: UiResolvedTextLine {
                text: "A".to_string(),
                frame: UiFrame::new(0.0, 0.0, 24.0, 24.0),
                source_range: UiTextRange { start: 0, end: 1 },
                visual_range: UiTextRange { start: 0, end: 1 },
                measured_width: 12.0,
                glyph_advances: vec![12.0],
                baseline: 16.0,
                direction: UiTextDirection::LeftToRight,
                runs: Vec::new(),
                ellipsized: false,
            },
        })),
        font_generation: 7,
    }
}
