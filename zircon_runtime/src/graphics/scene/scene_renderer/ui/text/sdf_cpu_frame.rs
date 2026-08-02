use crate::asset::ProjectAssetManager;
use crate::graphics::scene::scene_renderer::ui::render::{
    ScreenSpaceUiGlyphArtifactCacheIdentity, ScreenSpaceUiGlyphArtifactLine,
    ScreenSpaceUiShapedGlyph, ScreenSpaceUiTextBatch,
};
use crate::text::TextRenderState;
use crate::text::font::TextDecorationMetrics;
use crate::text::sdf::SdfRunCpuPreparation;

#[derive(Default)]
pub(super) struct SdfTextCpuFrame {
    valid: bool,
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
        if self.matches(sdf_texts, native_texts) {
            return true;
        }

        text_state.prepare_sdf_runs_cpu_into(sdf_texts, asset_manager, &mut self.sdf_runs);
        text_state.prepare_sdf_decoration_metrics_into(
            native_texts,
            asset_manager,
            &mut self.native_decoration_metrics,
        );
        replace_prepared_texts(&mut self.prepared_sdf_texts, sdf_texts);
        replace_prepared_texts(&mut self.prepared_native_texts, native_texts);
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
    }

    fn matches(
        &self,
        sdf_texts: &[ScreenSpaceUiTextBatch],
        native_texts: &[ScreenSpaceUiTextBatch],
    ) -> bool {
        self.valid
            && text_cpu_inputs_match(&self.prepared_sdf_texts, sdf_texts)
            && text_cpu_inputs_match(&self.prepared_native_texts, native_texts)
    }
}

fn replace_prepared_texts(
    prepared: &mut Vec<PreparedSdfCpuText>,
    texts: &[ScreenSpaceUiTextBatch],
) {
    prepared.clear();
    prepared.extend(texts.iter().map(PreparedSdfCpuText::from));
}

fn text_cpu_inputs_match(
    prepared: &[PreparedSdfCpuText],
    texts: &[ScreenSpaceUiTextBatch],
) -> bool {
    prepared.len() == texts.len()
        && prepared
            .iter()
            .zip(texts)
            .all(|(prepared, text)| prepared.matches(text))
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

    use super::PreparedSdfCpuText;
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
        let refreshed_line = replacement
            .glyph_artifact_line
            .as_ref()
            .and_then(|line| line.artifact.lines.first())
            .and_then(Option::as_ref)
            .expect("replacement artifact line")
            .clone();
        let mut refreshed = horizontal.clone();
        let artifact_line = refreshed
            .glyph_artifact_line
            .as_mut()
            .expect("original artifact line");
        artifact_line.refreshed_line = Some(Arc::new(refreshed_line));
        artifact_line.font_generation = 8;
        assert!(!prepared.matches(&refreshed));
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
            text: "fi".to_string(),
            frame,
            clip_frame: None,
            source_range: Some(UiTextRange { start: 0, end: 2 }),
            glyph_advances: vec![24.0],
            shaped_glyphs: Vec::new(),
            preserve_shaped_glyphs: true,
            glyph_artifact_line: Some(ScreenSpaceUiGlyphArtifactLine {
                artifact: Arc::new(ResolvedTextGlyphArtifact {
                    source_text: Arc::from("fi"),
                    source_text_origin: 0,
                    font_generation: 7,
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
                }),
                line_index: 0,
                refreshed_line: None,
                font_generation: 7,
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
