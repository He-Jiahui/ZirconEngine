use std::path::Path;

use glyphon::fontdb;

use crate::text::{
    FontFaceDescriptor, FontFamilyName, FontStretch, FontStyle, FontWeight, VariationCoords,
};

use super::database::FontSourceKey;
use super::face_metadata::FontFaceMetadata;

fn family_from_source_path(source_path: &Path) -> String {
    source_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(|stem| stem.replace(['_', '-'], " "))
        .unwrap_or_else(|| "Zircon Sans".to_string())
}

pub(super) fn descriptor_from_font_metadata(
    metadata: &FontFaceMetadata,
    family: Option<&str>,
    source_path: &Path,
    face_index: u32,
) -> FontFaceDescriptor {
    let family = family
        .map(str::trim)
        .filter(|family| !family.is_empty())
        .map(FontFamilyName::from)
        .or_else(|| metadata.discovered_family().cloned())
        .unwrap_or_else(|| FontFamilyName::from(family_from_source_path(source_path)));
    FontFaceDescriptor {
        family,
        weight: metadata.weight(),
        style: metadata.style(),
        stretch: metadata.stretch(),
        face_index,
        variations: VariationCoords::default(),
    }
}

pub(super) fn stretch_from_ttf_width_class(width_class: u16) -> FontStretch {
    let percent = match width_class {
        1 => 50,
        2 => 63,
        3 => 75,
        4 => 88,
        5 => 100,
        6 => 113,
        7 => 125,
        8 => 150,
        9 => 200,
        _ => 100,
    };
    FontStretch::clamped(percent)
}

pub(super) fn descriptor_from_fontdb_face(info: &fontdb::FaceInfo) -> Option<FontFaceDescriptor> {
    let family = info.families.first()?.0.as_str();
    Some(FontFaceDescriptor {
        family: FontFamilyName::from(family),
        weight: FontWeight::clamped(info.weight.0),
        style: style_from_fontdb(info.style),
        stretch: FontStretch::clamped(info.stretch.to_number()),
        face_index: info.index,
        variations: VariationCoords::default(),
    })
}

fn style_from_fontdb(style: fontdb::Style) -> FontStyle {
    match style {
        fontdb::Style::Normal => FontStyle::Normal,
        fontdb::Style::Italic => FontStyle::Italic,
        fontdb::Style::Oblique => FontStyle::Oblique(0.0),
    }
}

pub(super) fn source_key_from_fontdb_source(
    source: &fontdb::Source,
    face_index: u32,
) -> Option<FontSourceKey> {
    match source {
        fontdb::Source::File(path) => Some(FontSourceKey::from_path(path, face_index)),
        fontdb::Source::SharedFile(path, _) => Some(FontSourceKey::from_path(path, face_index)),
        fontdb::Source::Binary(_) => None,
    }
}
