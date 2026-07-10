use std::collections::BTreeSet;

use zircon_runtime_interface::ui::surface::{normalize_ui_text_language_tag, UiResolvedStyle};

use crate::graphics::scene::scene_renderer::ui::render::{
    ScreenSpaceUiShapedGlyph, ScreenSpaceUiTextBatch,
};
use crate::graphics::scene::scene_renderer::ui::sdf_char_run::sdf_scalar_requires_atlas_slot;
use crate::graphics::scene::scene_renderer::ui::sdf_params::SdfBakeParams;

use super::SdfAtlasGlyphKey;

pub(super) fn collect_sdf_atlas_text_keys(
    texts: &[ScreenSpaceUiTextBatch],
) -> (
    BTreeSet<SdfAtlasGlyphKey>,
    Vec<Vec<Option<SdfAtlasGlyphKey>>>,
) {
    let mut unique_keys = BTreeSet::<SdfAtlasGlyphKey>::new();
    let mut run_keys = Vec::with_capacity(texts.len());

    for text in texts {
        let glyph_keys = if text.shaped_glyphs.is_empty() {
            scalar_keys(text)
        } else {
            shaped_keys(text)
        };
        unique_keys.extend(glyph_keys.iter().flatten().cloned());
        run_keys.push(glyph_keys);
    }

    (unique_keys, run_keys)
}

fn scalar_keys(text: &ScreenSpaceUiTextBatch) -> Vec<Option<SdfAtlasGlyphKey>> {
    text.text
        .chars()
        .map(|glyph| {
            sdf_scalar_requires_atlas_slot(glyph).then(|| glyph_key(text, glyph, None, None))
        })
        .collect()
}

fn shaped_keys(text: &ScreenSpaceUiTextBatch) -> Vec<Option<SdfAtlasGlyphKey>> {
    text.shaped_glyphs
        .iter()
        .map(|glyph| shaped_key(text, glyph))
        .collect()
}

fn shaped_key(
    text: &ScreenSpaceUiTextBatch,
    glyph: &ScreenSpaceUiShapedGlyph,
) -> Option<SdfAtlasGlyphKey> {
    glyph.requires_atlas_slot.then(|| {
        glyph_key(
            text,
            glyph.source_scalar,
            Some(glyph.glyph_id),
            glyph.font_id.map(|font_id| font_id.0),
        )
    })
}

fn glyph_key(
    text: &ScreenSpaceUiTextBatch,
    glyph: char,
    glyph_id: Option<u32>,
    font_id: Option<u64>,
) -> SdfAtlasGlyphKey {
    SdfAtlasGlyphKey {
        glyph,
        glyph_id,
        font_id,
        font: text.font.clone(),
        font_family: text.font_family.clone(),
        language: normalize_ui_text_language_tag(text.language.as_deref()),
        font_weight: UiResolvedStyle::normalized_font_weight(text.font_weight),
        bake_params: SdfBakeParams::default(),
    }
}
