use std::collections::BTreeSet;
use std::sync::Arc;

use zircon_runtime_interface::ui::surface::UiResolvedStyle;

use crate::core::framework::text::TextFontFaceHandle;
use crate::graphics::scene::scene_renderer::ui::render::{
    ScreenSpaceUiGlyphArtifactLine, ScreenSpaceUiShapedGlyph, ScreenSpaceUiTextBatch,
};
use crate::text::{
    sdf::{sdf_scalar_requires_atlas_slot, SdfBakeParams},
    text_language_cache_identity,
};

use super::SdfAtlasGlyphKey;

#[cfg(test)]
pub(super) fn collect_sdf_atlas_text_keys(
    texts: &[ScreenSpaceUiTextBatch],
) -> (
    BTreeSet<SdfAtlasGlyphKey>,
    Vec<Vec<Option<SdfAtlasGlyphKey>>>,
) {
    collect_sdf_atlas_text_keys_iter(texts.iter())
}

pub(super) fn collect_sdf_atlas_text_keys_iter<'a, Texts>(
    texts: Texts,
) -> (
    BTreeSet<SdfAtlasGlyphKey>,
    Vec<Vec<Option<SdfAtlasGlyphKey>>>,
)
where
    Texts: IntoIterator<Item = &'a ScreenSpaceUiTextBatch>,
{
    let mut unique_keys = BTreeSet::<SdfAtlasGlyphKey>::new();
    let mut run_keys = Vec::new();

    for text in texts {
        let identity = SdfAtlasTextIdentity::new(text);
        let glyph_keys = if let Some(artifact_line) = text.glyph_artifact_line.as_ref() {
            artifact_keys(text, &identity, artifact_line)
        } else if text.shaped_glyphs.is_empty() {
            scalar_keys(text, &identity)
        } else {
            shaped_keys(text, &identity)
        };
        unique_keys.extend(glyph_keys.iter().flatten().cloned());
        run_keys.push(glyph_keys);
    }

    (unique_keys, run_keys)
}

struct SdfAtlasTextIdentity {
    font: Option<Arc<str>>,
    font_family: Option<Arc<str>>,
    language: Option<Arc<str>>,
}

impl SdfAtlasTextIdentity {
    fn new(text: &ScreenSpaceUiTextBatch) -> Self {
        Self {
            font: text.font.as_deref().map(Arc::<str>::from),
            font_family: text.font_family.as_deref().map(Arc::<str>::from),
            language: text_language_cache_identity(text.language.as_deref()).map(Arc::<str>::from),
        }
    }
}

fn artifact_keys(
    text: &ScreenSpaceUiTextBatch,
    identity: &SdfAtlasTextIdentity,
    artifact_line: &ScreenSpaceUiGlyphArtifactLine,
) -> Vec<Option<SdfAtlasGlyphKey>> {
    artifact_line
        .glyphs()
        .unwrap_or_default()
        .iter()
        .map(|glyph| {
            let source_scalar = artifact_line.source_scalar(glyph);
            (glyph.requires_rasterization && sdf_scalar_requires_atlas_slot(source_scalar)).then(
                || {
                    glyph_key(
                        text,
                        identity,
                        source_scalar,
                        Some(glyph.glyph_id),
                        glyph.font_face,
                        glyph.font_instance,
                    )
                },
            )
        })
        .collect()
}

#[cfg(test)]
#[path = "tests/text_keys.rs"]
mod tests;

fn scalar_keys(
    text: &ScreenSpaceUiTextBatch,
    identity: &SdfAtlasTextIdentity,
) -> Vec<Option<SdfAtlasGlyphKey>> {
    text.text
        .chars()
        .map(|glyph| {
            sdf_scalar_requires_atlas_slot(glyph)
                .then(|| glyph_key(text, identity, glyph, None, None, None))
        })
        .collect()
}

fn shaped_keys(
    text: &ScreenSpaceUiTextBatch,
    identity: &SdfAtlasTextIdentity,
) -> Vec<Option<SdfAtlasGlyphKey>> {
    text.shaped_glyphs
        .iter()
        .map(|glyph| shaped_key(text, identity, glyph))
        .collect()
}

fn shaped_key(
    text: &ScreenSpaceUiTextBatch,
    identity: &SdfAtlasTextIdentity,
    glyph: &ScreenSpaceUiShapedGlyph,
) -> Option<SdfAtlasGlyphKey> {
    glyph.requires_atlas_slot.then(|| {
        glyph_key(
            text,
            identity,
            glyph.source_scalar,
            Some(glyph.glyph_id),
            glyph.font_id,
            glyph.font_instance_id,
        )
    })
}

fn glyph_key(
    text: &ScreenSpaceUiTextBatch,
    identity: &SdfAtlasTextIdentity,
    glyph: char,
    glyph_id: Option<u32>,
    font_id: Option<TextFontFaceHandle>,
    font_instance_id: Option<TextFontFaceHandle>,
) -> SdfAtlasGlyphKey {
    SdfAtlasGlyphKey {
        glyph,
        glyph_id,
        font_id,
        font_instance_id,
        font: identity.font.clone(),
        font_family: identity.font_family.clone(),
        language: identity.language.clone(),
        font_weight: UiResolvedStyle::normalized_font_weight(text.font_weight),
        bake_params: SdfBakeParams {
            mode: text.distance_field_mode,
            ..SdfBakeParams::default()
        },
    }
}
