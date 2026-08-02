use std::collections::BTreeSet;
use std::sync::Arc;

use zircon_runtime_interface::ui::surface::{UiResolvedStyle, normalize_ui_text_language_tag};

use crate::core::framework::text::TextFontFaceHandle;
use crate::graphics::scene::scene_renderer::ui::render::{
    ScreenSpaceUiGlyphArtifactLine, ScreenSpaceUiShapedGlyph, ScreenSpaceUiTextBatch,
};
use crate::text::sdf::{SdfBakeParams, sdf_scalar_requires_atlas_slot};

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
            language: normalize_ui_text_language_tag(text.language.as_deref())
                .map(Arc::<str>::from),
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
mod tests {
    use std::sync::Arc;

    use zircon_runtime_interface::ui::event_ui::UiNodeId;
    use zircon_runtime_interface::ui::layout::UiFrame;
    use zircon_runtime_interface::ui::surface::{
        UiResolvedStyle, UiResolvedTextLine, UiTextAlign, UiTextDirection, UiTextRange, UiTextWrap,
        UiTextWritingMode,
    };

    use super::collect_sdf_atlas_text_keys;
    use crate::core::framework::text::{
        TextFontFaceHandle, TextGlyph, TextGlyphFlags, TextGlyphRotation,
    };
    use crate::graphics::scene::scene_renderer::ui::render::{
        ScreenSpaceUiGlyphArtifactLine, ScreenSpaceUiTextBatch, ScreenSpaceUiTextRouteIdentity,
    };
    use crate::text::sdf::SdfMode;
    use crate::text::{ResolvedTextGlyphArtifact, ResolvedTextGlyphArtifactLine};

    #[test]
    fn glyph_artifact_ligature_keys_match_the_text_owned_glyph_line() {
        let artifact = Arc::new(ResolvedTextGlyphArtifact {
            source_text: Arc::from("fi"),
            source_text_origin: 0,
            font_generation: 7,
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
                    font_face: Some(TextFontFaceHandle::new(3, 5)),
                    font_instance: Some(TextFontFaceHandle::new(4, 6)),
                    rotation: TextGlyphRotation::None,
                    bidi_level: 0,
                    flags: TextGlyphFlags::default(),
                    requires_rasterization: true,
                }],
                layout_line: UiResolvedTextLine {
                    text: "fi".to_string(),
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
        });
        let text = ScreenSpaceUiTextBatch {
            route_identity: ScreenSpaceUiTextRouteIdentity::new(
                "runtime.sdf-atlas.artifact-key.test",
                UiNodeId::new(1),
                None,
            ),
            command_generation: 1,
            text: "fi".to_string(),
            frame: UiFrame::new(0.0, 0.0, 12.0, 20.0),
            clip_frame: None,
            source_range: Some(UiTextRange { start: 0, end: 2 }),
            glyph_advances: vec![12.0],
            shaped_glyphs: Vec::new(),
            preserve_shaped_glyphs: true,
            glyph_artifact_line: Some(ScreenSpaceUiGlyphArtifactLine {
                artifact,
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
            writing_mode: UiTextWritingMode::HorizontalTb,
            wrap: UiTextWrap::None,
            style: Default::default(),
            distance_field_mode: SdfMode::Sdf,
            text_effects: Default::default(),
            text_decorations: Default::default(),
            text_decoration_baseline: None,
            clip_transform: None,
        };

        let (unique_keys, runs) = collect_sdf_atlas_text_keys(&[text]);

        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].len(), 1);
        let key = runs[0][0].as_ref().expect("ligature needs an atlas key");
        assert_eq!(key.glyph, 'f');
        assert_eq!(key.glyph_id, Some(0xfb01));
        assert_eq!(key.font_id, Some(TextFontFaceHandle::new(3, 5)));
        assert_eq!(key.font_instance_id, Some(TextFontFaceHandle::new(4, 6)));
        assert_eq!(unique_keys.len(), 1);
    }
}

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
