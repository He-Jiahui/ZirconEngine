use crate::graphics::scene::scene_renderer::ui::render::{
    ScreenSpaceUiGlyphArtifactCacheIdentity, ScreenSpaceUiGlyphArtifactLine,
    ScreenSpaceUiShapedGlyph, ScreenSpaceUiTextBatch,
};
use crate::text::sdf::SdfMode;

#[derive(Default)]
/// 非分段图集路径的输入快照；比较文本、塑形、字体及字形产物身份，稳定帧可复用槽规划。
pub(super) struct PreparedSdfAtlasTexts {
    texts: Vec<PreparedSdfAtlasText>,
}

struct PreparedSdfAtlasText {
    text: String,
    shaped_glyphs: Vec<ScreenSpaceUiShapedGlyph>,
    // TODO: [CR-R02-runtime_wave12_graphics_ui_atlas_sdf-0005] 快照仅存产物地址而不持有 Arc；尚缺跨帧地址不被复用的保活证据，需追踪父级引用并验证同代数产物替换后的缓存失效。
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
