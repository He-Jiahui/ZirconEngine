use crate::graphics::scene::scene_renderer::ui::render::{
    ScreenSpaceUiGlyphArtifactCacheIdentity, ScreenSpaceUiGlyphArtifactLine,
    ScreenSpaceUiShapedGlyph, ScreenSpaceUiTextBatch,
};
use crate::text::sdf::SdfMode;

#[derive(Default)]
pub(super) struct PreparedSdfAtlasTexts {
    texts: Vec<PreparedSdfAtlasText>,
}

struct PreparedSdfAtlasText {
    text: String,
    shaped_glyphs: Vec<ScreenSpaceUiShapedGlyph>,
    glyph_artifact_identity: Option<ScreenSpaceUiGlyphArtifactCacheIdentity>,
    font: Option<String>,
    font_family: Option<String>,
    language: Option<String>,
    font_weight: u16,
    writing_mode: zircon_runtime_interface::ui::surface::UiTextWritingMode,
    distance_field_mode: SdfMode,
}

impl PreparedSdfAtlasTexts {
    pub(super) fn matches_iter<'a, Texts>(&self, texts: Texts) -> bool
    where
        Texts: IntoIterator<Item = &'a ScreenSpaceUiTextBatch>,
    {
        let mut texts = texts.into_iter();
        self.texts
            .iter()
            .all(|prepared| texts.next().is_some_and(|text| prepared.matches(text)))
            && texts.next().is_none()
    }

    pub(super) fn replace_iter<'a, Texts>(&mut self, texts: Texts)
    where
        Texts: IntoIterator<Item = &'a ScreenSpaceUiTextBatch>,
    {
        self.texts.clear();
        self.texts
            .extend(texts.into_iter().map(PreparedSdfAtlasText::new));
    }

    pub(super) fn clear(&mut self) {
        self.texts.clear();
    }
}

impl PreparedSdfAtlasText {
    fn new(text: &ScreenSpaceUiTextBatch) -> Self {
        Self {
            text: text.text.clone(),
            shaped_glyphs: text.shaped_glyphs.clone(),
            glyph_artifact_identity: text
                .glyph_artifact_line
                .as_ref()
                .map(ScreenSpaceUiGlyphArtifactLine::cache_identity),
            font: text.font.clone(),
            font_family: text.font_family.clone(),
            language: text.language.clone(),
            font_weight: text.font_weight,
            writing_mode: text.writing_mode,
            distance_field_mode: text.distance_field_mode,
        }
    }

    fn matches(&self, text: &ScreenSpaceUiTextBatch) -> bool {
        self.text == text.text
            && self.shaped_glyphs.as_slice() == text.shaped_glyphs.as_slice()
            && self.glyph_artifact_identity
                == text
                    .glyph_artifact_line
                    .as_ref()
                    .map(ScreenSpaceUiGlyphArtifactLine::cache_identity)
            && self.font == text.font
            && self.font_family == text.font_family
            && self.language == text.language
            && self.font_weight == text.font_weight
            && self.writing_mode == text.writing_mode
            && self.distance_field_mode == text.distance_field_mode
    }
}
