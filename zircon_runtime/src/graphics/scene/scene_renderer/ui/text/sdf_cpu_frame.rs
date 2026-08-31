use crate::asset::ProjectAssetManager;
use crate::graphics::scene::scene_renderer::ui::render::{
    ScreenSpaceUiGlyphArtifactCacheIdentity, ScreenSpaceUiGlyphArtifactLine,
    ScreenSpaceUiShapedGlyph, ScreenSpaceUiTextBatch,
};
use crate::text::TextRenderState;
use crate::text::font::TextDecorationMetrics;
use crate::text::sdf::SdfRunCpuPreparation;

use super::ScreenSpaceUiTextFrameProductGeneration;

#[derive(Default)]
pub(super) struct SdfTextCpuFrame {
    valid: bool,
    retained_frame_generation: Option<ScreenSpaceUiTextFrameProductGeneration>,
    prepared_sdf_texts: Vec<PreparedSdfCpuText>,
    prepared_native_texts: Vec<PreparedSdfCpuText>,
    sdf_runs: Vec<SdfRunCpuPreparation>,
    native_decoration_metrics: Vec<TextDecorationMetrics>,
}

#[derive(Clone)]
struct PreparedSdfCpuText {
    text: String,
    glyph_advances: Vec<f32>,
    shaped_glyphs: Vec<ScreenSpaceUiShapedGlyph>,
    glyph_artifact_identity: Option<ScreenSpaceUiGlyphArtifactCacheIdentity>,
    font: Option<String>,
    font_family: Option<String>,
    language: Option<String>,
    font_weight: u16,
    font_size: f32,
    writing_mode: zircon_runtime_interface::ui::surface::UiTextWritingMode,
}

impl SdfTextCpuFrame {
    pub(super) fn prepare(
        &mut self,
        sdf_texts: &[ScreenSpaceUiTextBatch],
        native_texts: &[ScreenSpaceUiTextBatch],
        text_state: &mut TextRenderState,
        asset_manager: &ProjectAssetManager,
    ) -> bool {
        self.prepare_with_retained_generation(
            sdf_texts,
            native_texts,
            text_state,
            asset_manager,
            None,
        )
    }

    pub(super) fn prepare_retained(
        &mut self,
        sdf_texts: &[ScreenSpaceUiTextBatch],
        native_texts: &[ScreenSpaceUiTextBatch],
        text_state: &mut TextRenderState,
        asset_manager: &ProjectAssetManager,
        generation: ScreenSpaceUiTextFrameProductGeneration,
    ) -> bool {
        self.prepare_with_retained_generation(
            sdf_texts,
            native_texts,
            text_state,
            asset_manager,
            Some(generation),
        )
    }

    pub(super) fn prepare_retained_segments<'a, SdfSegments, NativeSegments>(
        &mut self,
        sdf_segments: SdfSegments,
        native_segments: NativeSegments,
        text_state: &mut TextRenderState,
        asset_manager: &ProjectAssetManager,
        generation: ScreenSpaceUiTextFrameProductGeneration,
    ) -> bool
    where
        SdfSegments: Clone + Iterator<Item = &'a [ScreenSpaceUiTextBatch]>,
        NativeSegments: Clone + Iterator<Item = &'a [ScreenSpaceUiTextBatch]>,
    {
        self.prepare_with_retained_text_iter(
            sdf_segments.flat_map(|segment| segment.iter()),
            native_segments.flat_map(|segment| segment.iter()),
            text_state,
            asset_manager,
            Some(generation),
        )
    }

    fn prepare_with_retained_generation(
        &mut self,
        sdf_texts: &[ScreenSpaceUiTextBatch],
        native_texts: &[ScreenSpaceUiTextBatch],
        text_state: &mut TextRenderState,
        asset_manager: &ProjectAssetManager,
        retained_generation: Option<ScreenSpaceUiTextFrameProductGeneration>,
    ) -> bool {
        self.prepare_with_retained_text_iter(
            sdf_texts.iter(),
            native_texts.iter(),
            text_state,
            asset_manager,
            retained_generation,
        )
    }

    fn prepare_with_retained_text_iter<'a, SdfTexts, NativeTexts>(
        &mut self,
        sdf_texts: SdfTexts,
        native_texts: NativeTexts,
        text_state: &mut TextRenderState,
        asset_manager: &ProjectAssetManager,
        retained_generation: Option<ScreenSpaceUiTextFrameProductGeneration>,
    ) -> bool
    where
        SdfTexts: Clone + Iterator<Item = &'a ScreenSpaceUiTextBatch>,
        NativeTexts: Clone + Iterator<Item = &'a ScreenSpaceUiTextBatch>,
    {
        if self.valid
            && retained_generation.is_some()
            && self.retained_frame_generation == retained_generation
        {
            return true;
        }
        if self.matches_iter(sdf_texts.clone(), native_texts.clone()) {
            self.retained_frame_generation = retained_generation;
            return true;
        }

        text_state.prepare_sdf_runs_cpu_iter_into(
            sdf_texts.clone(),
            asset_manager,
            &mut self.sdf_runs,
        );
        text_state.prepare_sdf_decoration_metrics_iter_into(
            native_texts.clone(),
            asset_manager,
            &mut self.native_decoration_metrics,
        );
        replace_prepared_texts_iter(&mut self.prepared_sdf_texts, sdf_texts);
        replace_prepared_texts_iter(&mut self.prepared_native_texts, native_texts);
        self.retained_frame_generation = retained_generation;
        self.valid = true;
        false
    }

    pub(super) fn outputs_mut(
        &mut self,
    ) -> (
        &mut Vec<SdfRunCpuPreparation>,
        &mut Vec<TextDecorationMetrics>,
    ) {
        (&mut self.sdf_runs, &mut self.native_decoration_metrics)
    }

    pub(super) fn outputs(&self) -> (&[SdfRunCpuPreparation], &[TextDecorationMetrics]) {
        (&self.sdf_runs, &self.native_decoration_metrics)
    }

    pub(super) fn invalidate(&mut self) {
        self.valid = false;
        self.retained_frame_generation = None;
    }

    fn matches_iter<'a, SdfTexts, NativeTexts>(
        &self,
        sdf_texts: SdfTexts,
        native_texts: NativeTexts,
    ) -> bool
    where
        SdfTexts: IntoIterator<Item = &'a ScreenSpaceUiTextBatch>,
        NativeTexts: IntoIterator<Item = &'a ScreenSpaceUiTextBatch>,
    {
        self.valid
            && text_cpu_inputs_match_iter(&self.prepared_sdf_texts, sdf_texts)
            && text_cpu_inputs_match_iter(&self.prepared_native_texts, native_texts)
    }
}

fn replace_prepared_texts_iter<'a, Texts>(prepared: &mut Vec<PreparedSdfCpuText>, texts: Texts)
where
    Texts: IntoIterator<Item = &'a ScreenSpaceUiTextBatch>,
{
    prepared.clear();
    prepared.extend(texts.into_iter().map(PreparedSdfCpuText::from));
}

fn text_cpu_inputs_match_iter<'a, Texts>(prepared: &[PreparedSdfCpuText], texts: Texts) -> bool
where
    Texts: IntoIterator<Item = &'a ScreenSpaceUiTextBatch>,
{
    let mut texts = texts.into_iter();
    prepared
        .iter()
        .all(|prepared| texts.next().is_some_and(|text| prepared.matches(text)))
        && texts.next().is_none()
}

impl From<&ScreenSpaceUiTextBatch> for PreparedSdfCpuText {
    fn from(text: &ScreenSpaceUiTextBatch) -> Self {
        Self {
            text: text.text.clone(),
            glyph_advances: text.glyph_advances.clone(),
            shaped_glyphs: text.shaped_glyphs.clone(),
            glyph_artifact_identity: text
                .glyph_artifact_line
                .as_ref()
                .map(ScreenSpaceUiGlyphArtifactLine::cache_identity),
            font: text.font.clone(),
            font_family: text.font_family.clone(),
            language: text.language.clone(),
            font_weight: text.font_weight,
            font_size: text.font_size,
            writing_mode: text.writing_mode,
        }
    }
}

impl PreparedSdfCpuText {
    fn matches(&self, text: &ScreenSpaceUiTextBatch) -> bool {
        self.text == text.text
            && self.glyph_advances == text.glyph_advances
            && self.shaped_glyphs == text.shaped_glyphs
            && self.glyph_artifact_identity
                == text
                    .glyph_artifact_line
                    .as_ref()
                    .map(ScreenSpaceUiGlyphArtifactLine::cache_identity)
            && self.font == text.font
            && self.font_family == text.font_family
            && self.language == text.language
            && self.font_weight == text.font_weight
            && self.font_size == text.font_size
            && self.writing_mode == text.writing_mode
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use zircon_runtime_interface::ui::event_ui::UiNodeId;
    use zircon_runtime_interface::ui::layout::UiFrame;
    use zircon_runtime_interface::ui::surface::{
        UiResolvedStyle, UiResolvedTextLine, UiTextAlign, UiTextDirection, UiTextRange, UiTextWrap,
        UiTextWritingMode,
    };

    use super::{PreparedSdfCpuText, text_cpu_inputs_match_iter};
    use crate::core::framework::text::{TextGlyph, TextGlyphFlags, TextGlyphRotation};
    use crate::graphics::scene::scene_renderer::ui::render::{
        ScreenSpaceUiGlyphArtifactLine, ScreenSpaceUiTextBatch, ScreenSpaceUiTextRouteIdentity,
    };
    use crate::text::sdf::SdfMode;
    use crate::text::{ResolvedTextGlyphArtifact, ResolvedTextGlyphArtifactLine};

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

    fn artifact_text_batch(
        glyph_id: u32,
        writing_mode: UiTextWritingMode,
    ) -> ScreenSpaceUiTextBatch {
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
}
