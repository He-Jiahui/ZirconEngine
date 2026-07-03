use std::path::Path;

use glyphon::fontdb;
use ttf_parser::{name_id, Face, Style as TtfStyle};

use crate::core::framework::render::{
    FontFaceDescriptor, FontFamilyName, FontStretch, FontStyle, FontWeight, VariationCoords,
};

use super::database::FontSourceKey;

fn family_from_source_path(source_path: &Path) -> String {
    source_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(|stem| stem.replace(['_', '-'], " "))
        .unwrap_or_else(|| "Zircon Sans".to_string())
}

pub(super) fn descriptor_from_font_bytes(
    bytes: &[u8],
    family: Option<&str>,
    source_path: &Path,
    face_index: u32,
) -> FontFaceDescriptor {
    let parsed = Face::parse(bytes, face_index).ok();
    let family = family
        .map(str::trim)
        .filter(|family| !family.is_empty())
        .map(FontFamilyName::from)
        .or_else(|| {
            parsed
                .as_ref()
                .and_then(face_family_name)
                .map(FontFamilyName::from)
        })
        .unwrap_or_else(|| FontFamilyName::from(family_from_source_path(source_path)));

    let Some(face) = parsed else {
        let mut descriptor = FontFaceDescriptor::regular(family);
        descriptor.face_index = face_index;
        return descriptor;
    };

    FontFaceDescriptor {
        family,
        weight: FontWeight::clamped(face.weight().to_number()),
        style: style_from_ttf(face.style()),
        stretch: stretch_from_ttf_width_class(face.width().to_number()),
        face_index,
        variations: VariationCoords::default(),
    }
}

fn face_family_name(face: &Face<'_>) -> Option<String> {
    ttf_name_by_id(face, name_id::TYPOGRAPHIC_FAMILY)
        .or_else(|| ttf_name_by_id(face, name_id::FAMILY))
}

fn ttf_name_by_id(face: &Face<'_>, id: u16) -> Option<String> {
    face.names()
        .into_iter()
        .filter(|name| name.name_id == id)
        .filter_map(|name| name.to_string())
        .find(|value| !value.trim().is_empty())
}

fn style_from_ttf(style: TtfStyle) -> FontStyle {
    match style {
        TtfStyle::Normal => FontStyle::Normal,
        TtfStyle::Italic => FontStyle::Italic,
        TtfStyle::Oblique => FontStyle::Oblique(0.0),
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
