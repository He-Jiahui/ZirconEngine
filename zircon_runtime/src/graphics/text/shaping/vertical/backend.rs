use std::str::FromStr;

use rustybuzz::{ttf_parser::Tag, Direction, Feature, Language, UnicodeBuffer};

use crate::core::framework::render::{FontFaceId, OpenTypeFeature};
use crate::graphics::text::font::FontDatabase;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum VerticalBackendDirection {
    TopToBottom,
    BottomToTop,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct VerticalBackendGlyph {
    pub(super) glyph_id: u32,
    pub(super) source_offset: usize,
    pub(super) y_advance: f32,
    pub(super) x_offset: f32,
    pub(super) y_offset: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct VerticalBackendRun {
    pub(super) glyphs: Vec<VerticalBackendGlyph>,
}

pub(super) fn shape_vertical_run(
    database: &FontDatabase,
    face: FontFaceId,
    text: &str,
    direction: VerticalBackendDirection,
    language: Option<&str>,
    features: &[OpenTypeFeature],
    include_kerning: bool,
    font_size: f32,
) -> Option<VerticalBackendRun> {
    if text.is_empty() {
        return Some(VerticalBackendRun { glyphs: Vec::new() });
    }

    let bytes = database.face_bytes(face).ok()?;
    let face_index = database.face_index(face).ok()?;
    let face = rustybuzz::Face::from_slice(bytes.as_ref(), face_index)?;
    let scale = font_size.max(1.0) / face.units_per_em().max(1) as f32;
    let mut buffer = UnicodeBuffer::new();
    buffer.push_str(text);
    buffer.set_direction(match direction {
        VerticalBackendDirection::TopToBottom => Direction::TopToBottom,
        VerticalBackendDirection::BottomToTop => Direction::BottomToTop,
    });
    if let Some(language) = language.and_then(|value| Language::from_str(value).ok()) {
        buffer.set_language(language);
    }
    buffer.guess_segment_properties();

    let mut projected_features = features
        .iter()
        .map(|feature| Feature::new(Tag::from_bytes(&feature.tag), feature.value, ..))
        .collect::<Vec<_>>();
    if !include_kerning {
        projected_features.push(Feature::new(Tag::from_bytes(b"kern"), 0, ..));
        projected_features.push(Feature::new(Tag::from_bytes(b"vkrn"), 0, ..));
    }

    let shaped = rustybuzz::shape(&face, &projected_features, buffer);
    let glyphs = shaped
        .glyph_infos()
        .iter()
        .zip(shaped.glyph_positions())
        .map(|(info, position)| VerticalBackendGlyph {
            glyph_id: info.glyph_id,
            source_offset: info.cluster as usize,
            y_advance: position.y_advance as f32 * scale,
            x_offset: position.x_offset as f32 * scale,
            y_offset: position.y_offset as f32 * scale,
        })
        .collect::<Vec<_>>();
    (!glyphs.is_empty()).then_some(VerticalBackendRun { glyphs })
}
