use std::str::FromStr;

use crate::core::framework::text::TextDirection;
use rustybuzz::{
    script, ttf_parser::Tag, Direction, Feature, Language, Script, UnicodeBuffer, Variation,
};

use crate::text::font::FontDatabase;
use crate::text::{FontFaceId, InstancedFaceId, OpenTypeFeature};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::text::shaping) struct HorizontalBackendGlyph {
    pub(in crate::text::shaping) glyph_id: u32,
    pub(in crate::text::shaping) source_offset: usize,
    pub(in crate::text::shaping) advance: f32,
    pub(in crate::text::shaping) x_offset: f32,
    pub(in crate::text::shaping) y_offset: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::text::shaping) struct HorizontalBackendRun {
    pub(in crate::text::shaping) glyphs: Vec<HorizontalBackendGlyph>,
}

#[allow(clippy::too_many_arguments)]
pub(in crate::text::shaping) fn shape_horizontal_run(
    database: &FontDatabase,
    face_id: FontFaceId,
    instance_id: Option<InstancedFaceId>,
    text: &str,
    direction: TextDirection,
    script_tag: &str,
    language: Option<&str>,
    features: &[OpenTypeFeature],
    include_kerning: bool,
    font_weight: u16,
    font_size: f32,
) -> Option<HorizontalBackendRun> {
    if text.is_empty() {
        return Some(HorizontalBackendRun { glyphs: Vec::new() });
    }

    let variations = database
        .effective_instance_variations_shared(face_id, instance_id, font_weight)
        .ok()?;
    let language = language.and_then(|value| Language::from_str(value).ok());
    let bytes = database.face_bytes(face_id).ok()?;
    let face_index = database.face_index(face_id).ok()?;
    let mut face = rustybuzz::Face::from_slice(bytes.as_ref(), face_index)?;
    let variations = variations
        .0
        .iter()
        .map(|(tag, value)| Variation {
            tag: Tag::from_bytes(&tag.to_be_bytes()),
            value: *value,
        })
        .collect::<Vec<_>>();
    face.set_variations(&variations);
    let scale = font_size.max(1.0) / face.units_per_em().max(1) as f32;

    let mut buffer = UnicodeBuffer::new();
    buffer.push_str(text);
    buffer.set_direction(match direction {
        TextDirection::RightToLeft => Direction::RightToLeft,
        TextDirection::Auto | TextDirection::LeftToRight | TextDirection::Mixed => {
            Direction::LeftToRight
        }
    });
    if let Some(language) = language {
        buffer.set_language(language);
    }
    if let Some(script) = explicit_script(script_tag) {
        buffer.set_script(script);
    }
    buffer.guess_segment_properties();

    let mut projected_features = features
        .iter()
        .map(|feature| Feature::new(Tag::from_bytes(&feature.tag), feature.value, ..))
        .collect::<Vec<_>>();
    if !include_kerning {
        projected_features.push(Feature::new(Tag::from_bytes(b"kern"), 0, ..));
    }

    let shaped = rustybuzz::shape(&face, &projected_features, buffer);
    let glyphs = shaped
        .glyph_infos()
        .iter()
        .zip(shaped.glyph_positions())
        .map(|(info, position)| HorizontalBackendGlyph {
            glyph_id: info.glyph_id,
            source_offset: info.cluster as usize,
            advance: position.x_advance as f32 * scale,
            x_offset: position.x_offset as f32 * scale,
            y_offset: position.y_offset as f32 * scale,
        })
        .collect::<Vec<_>>();
    (!glyphs.is_empty()).then_some(HorizontalBackendRun { glyphs })
}

fn explicit_script(script_tag: &str) -> Option<Script> {
    let script = Script::from_str(script_tag).ok()?;
    (!matches!(script, script::COMMON | script::INHERITED | script::UNKNOWN)).then_some(script)
}
