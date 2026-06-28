use std::collections::BTreeSet;

use crate::asset::assets::{
    FontAssetCmapCoverage, FontAssetCodepointRange, FontAssetFaceStyle, FontAssetMetadata,
    FontAssetParsedFace, FontAssetSourceFormat, FontAssetVariableInstance, FontAssetVariationAxis,
    FontAssetVariationCoord,
};
use ttf_parser::{name_id, Face, FaceParsingError, Style, Tag};

pub(super) fn parse_font_metadata(bytes: &[u8]) -> Result<FontAssetMetadata, FontParseError> {
    if is_woff2(bytes) {
        return Err(FontParseError::Woff2DecodeUnsupported);
    }

    let face_count = ttf_parser::fonts_in_collection(bytes).unwrap_or(1);
    let source_format = if face_count > 1 {
        FontAssetSourceFormat::TrueTypeCollection
    } else {
        FontAssetSourceFormat::Sfnt
    };
    let mut faces = Vec::new();
    for face_index in 0..face_count {
        let face = Face::parse(bytes, face_index)?;
        faces.push(parse_face(bytes, face_index, &face));
    }

    Ok(FontAssetMetadata {
        source_format,
        face_count,
        faces,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum FontParseError {
    Face(FaceParsingError),
    Woff2DecodeUnsupported,
}

impl std::fmt::Display for FontParseError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Face(error) => write!(formatter, "{error}"),
            Self::Woff2DecodeUnsupported => {
                write!(
                    formatter,
                    "WOFF2 decode is not yet wired into the font importer"
                )
            }
        }
    }
}

impl std::error::Error for FontParseError {}

impl From<FaceParsingError> for FontParseError {
    fn from(error: FaceParsingError) -> Self {
        Self::Face(error)
    }
}

fn parse_face(bytes: &[u8], face_index: u32, face: &Face<'_>) -> FontAssetParsedFace {
    let axes = variation_axes(face);
    let named_instances = parse_named_instances(
        face.table_data(Tag::from_bytes(b"fvar")),
        axes.len(),
        |name_id| name_by_id(face, name_id),
    );

    FontAssetParsedFace {
        face_index,
        family: name_by_id(face, name_id::TYPOGRAPHIC_FAMILY)
            .or_else(|| name_by_id(face, name_id::FAMILY)),
        subfamily: name_by_id(face, name_id::TYPOGRAPHIC_SUBFAMILY)
            .or_else(|| name_by_id(face, name_id::SUBFAMILY)),
        full_name: name_by_id(face, name_id::FULL_NAME),
        post_script_name: name_by_id(face, name_id::POST_SCRIPT_NAME),
        weight: face.weight().to_number(),
        width_class: face.width().to_number(),
        style: style_from_face(face.style()),
        variation_axes: axes,
        named_instances,
        cmap: cmap_coverage(bytes, face),
    }
}

fn variation_axes(face: &Face<'_>) -> Vec<FontAssetVariationAxis> {
    face.variation_axes()
        .into_iter()
        .map(|axis| FontAssetVariationAxis {
            tag: tag_to_string(axis.tag),
            min: axis.min_value,
            default: axis.def_value,
            max: axis.max_value,
            name: name_by_id(face, axis.name_id),
            hidden: axis.hidden,
        })
        .collect()
}

fn cmap_coverage(bytes: &[u8], face: &Face<'_>) -> FontAssetCmapCoverage {
    let mut codepoints = BTreeSet::new();
    if let Some(cmap) = face.tables().cmap {
        for subtable in cmap.subtables {
            if !subtable.is_unicode() {
                continue;
            }
            subtable.codepoints(|codepoint| {
                if char::from_u32(codepoint)
                    .and_then(|ch| face.glyph_index(ch))
                    .is_some()
                {
                    codepoints.insert(codepoint);
                }
            });
        }
    }

    if codepoints.is_empty() {
        // Some tiny or unusual fonts expose limited high-level cmap data. Keep the
        // coverage useful for fallback tests by probing the printable BMP range.
        for codepoint in 0x20..=0xFFFF {
            if let Some(ch) = char::from_u32(codepoint) {
                if face.glyph_index(ch).is_some() {
                    codepoints.insert(codepoint);
                }
            }
        }
        if bytes.starts_with(b"ttcf") {
            codepoints.insert(0);
        }
    }

    FontAssetCmapCoverage {
        codepoint_count: codepoints.len() as u32,
        ranges: compact_codepoint_ranges(codepoints),
    }
}

fn compact_codepoint_ranges(codepoints: BTreeSet<u32>) -> Vec<FontAssetCodepointRange> {
    let mut ranges = Vec::new();
    let mut iter = codepoints.into_iter();
    let Some(mut start) = iter.next() else {
        return ranges;
    };
    let mut end = start;
    for codepoint in iter {
        if codepoint == end.saturating_add(1) {
            end = codepoint;
            continue;
        }
        ranges.push(FontAssetCodepointRange { start, end });
        start = codepoint;
        end = codepoint;
    }
    ranges.push(FontAssetCodepointRange { start, end });
    ranges
}

fn name_by_id(face: &Face<'_>, id: u16) -> Option<String> {
    face.names()
        .into_iter()
        .filter(|name| name.name_id == id)
        .filter_map(|name| name.to_string())
        .find(|value| !value.trim().is_empty())
}

fn style_from_face(style: Style) -> FontAssetFaceStyle {
    match style {
        Style::Normal => FontAssetFaceStyle::Normal,
        Style::Italic => FontAssetFaceStyle::Italic,
        Style::Oblique => FontAssetFaceStyle::Oblique,
    }
}

fn parse_named_instances(
    fvar: Option<&[u8]>,
    axis_count: usize,
    mut resolve_name: impl FnMut(u16) -> Option<String>,
) -> Vec<FontAssetVariableInstance> {
    let Some(fvar) = fvar else {
        return Vec::new();
    };
    let Some(axis_count_u16) = read_u16(fvar, 8).map(usize::from) else {
        return Vec::new();
    };
    if axis_count_u16 != axis_count {
        return Vec::new();
    }
    let axes_array_offset = read_u16(fvar, 4).map(usize::from).unwrap_or(0);
    let axis_size = read_u16(fvar, 10).map(usize::from).unwrap_or(20);
    let instance_count = read_u16(fvar, 12).map(usize::from).unwrap_or(0);
    let instance_size = read_u16(fvar, 14).map(usize::from).unwrap_or(0);
    if axes_array_offset == 0 || axis_size < 20 || instance_size < 4 + axis_count * 4 {
        return Vec::new();
    }

    let Some(instances_offset) = axes_array_offset.checked_add(axis_count * axis_size) else {
        return Vec::new();
    };
    let axis_tags = parse_axis_tags(fvar, axes_array_offset, axis_count, axis_size);
    let mut instances = Vec::new();
    for index in 0..instance_count {
        let Some(offset) = instances_offset.checked_add(index * instance_size) else {
            break;
        };
        let Some(subfamily_name_id) = read_u16(fvar, offset) else {
            break;
        };
        let mut coordinates = Vec::new();
        for axis_index in 0..axis_count {
            let coord_offset = offset + 4 + axis_index * 4;
            let Some(value) = read_fixed(fvar, coord_offset) else {
                continue;
            };
            if let Some(tag) = axis_tags.get(axis_index) {
                coordinates.push(FontAssetVariationCoord {
                    tag: tag.clone(),
                    value,
                });
            }
        }

        let post_script_name = if instance_size >= 6 + axis_count * 4 {
            read_u16(fvar, offset + 4 + axis_count * 4).and_then(|id| resolve_name(id))
        } else {
            None
        };
        instances.push(FontAssetVariableInstance {
            name: resolve_name(subfamily_name_id),
            post_script_name,
            coordinates,
        });
    }
    instances
}

fn parse_axis_tags(
    fvar: &[u8],
    axes_array_offset: usize,
    axis_count: usize,
    axis_size: usize,
) -> Vec<String> {
    (0..axis_count)
        .filter_map(|axis_index| {
            let offset = axes_array_offset.checked_add(axis_index * axis_size)?;
            let tag = fvar.get(offset..offset + 4)?;
            Some(String::from_utf8_lossy(tag).into_owned())
        })
        .collect()
}

fn read_u16(data: &[u8], offset: usize) -> Option<u16> {
    let bytes = data.get(offset..offset + 2)?;
    Some(u16::from_be_bytes([bytes[0], bytes[1]]))
}

fn read_i32(data: &[u8], offset: usize) -> Option<i32> {
    let bytes = data.get(offset..offset + 4)?;
    Some(i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn read_fixed(data: &[u8], offset: usize) -> Option<f32> {
    read_i32(data, offset).map(|value| value as f32 / 65_536.0)
}

fn tag_to_string(tag: Tag) -> String {
    String::from_utf8_lossy(&tag.to_bytes()).into_owned()
}

fn is_woff2(bytes: &[u8]) -> bool {
    bytes.starts_with(b"wOF2")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fira_regular() -> Vec<u8> {
        std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("assets")
                .join("fonts")
                .join("FiraSans-Regular.ttf"),
        )
        .unwrap()
    }

    #[test]
    fn text_font_cmap_coverage_bitset_matches_face() {
        let bytes = fira_regular();
        let metadata = parse_font_metadata(&bytes).unwrap();
        let face = Face::parse(&bytes, 0).unwrap();
        let coverage = &metadata.faces[0].cmap;

        assert!(coverage.codepoint_count > 0);
        for ch in ['A', 'a', '0'] {
            assert_eq!(
                coverage.contains_codepoint(ch as u32),
                face.glyph_index(ch).is_some()
            );
        }
    }

    #[test]
    fn text_font_static_face_reports_no_variable_axes() {
        let bytes = fira_regular();
        let metadata = parse_font_metadata(&bytes).unwrap();

        assert_eq!(metadata.face_count, 1);
        assert!(metadata.faces[0].variation_axes.is_empty());
        assert!(metadata.faces[0].named_instances.is_empty());
    }

    #[test]
    fn text_font_parse_ttf_extracts_os2_name_metadata() {
        let bytes = fira_regular();
        let metadata = parse_font_metadata(&bytes).unwrap();
        let face = &metadata.faces[0];

        assert_eq!(metadata.source_format, FontAssetSourceFormat::Sfnt);
        assert_eq!(face.face_index, 0);
        assert!(face
            .family
            .as_deref()
            .is_some_and(|family| family.contains("Fira")));
        assert_eq!(face.weight, 400);
        assert_eq!(face.width_class, 5);
        assert_eq!(face.style, FontAssetFaceStyle::Normal);
    }

    #[test]
    fn text_font_parse_ttc_enumerates_faces() {
        let regular = fira_regular();
        let mut second_face = regular.clone();
        patch_os2_weight(&mut second_face, 700);
        let collection = ttc_from_fonts(&[regular.as_slice(), second_face.as_slice()]);

        let metadata = parse_font_metadata(&collection).unwrap();

        assert_eq!(
            metadata.source_format,
            FontAssetSourceFormat::TrueTypeCollection
        );
        assert_eq!(metadata.face_count, 2);
        assert_eq!(metadata.faces[0].face_index, 0);
        assert_eq!(metadata.faces[1].face_index, 1);
        assert_eq!(metadata.faces[0].weight, 400);
        assert_eq!(metadata.faces[1].weight, 700);
    }

    fn ttc_from_fonts(fonts: &[&[u8]]) -> Vec<u8> {
        let header_len = 12 + fonts.len() * 4;
        let mut output = vec![0; header_len];
        output[0..4].copy_from_slice(b"ttcf");
        output[4..8].copy_from_slice(&0x0001_0000_u32.to_be_bytes());
        output[8..12].copy_from_slice(&(fonts.len() as u32).to_be_bytes());

        for (font_index, font) in fonts.iter().enumerate() {
            pad_to_four(&mut output);
            let directory_offset = output.len();
            let offset_slot = 12 + font_index * 4;
            output[offset_slot..offset_slot + 4]
                .copy_from_slice(&(directory_offset as u32).to_be_bytes());

            let table_count = u16::from_be_bytes([font[4], font[5]]) as usize;
            let directory_len = 12 + table_count * 16;
            output.extend_from_slice(&font[..directory_len]);
            for table_index in 0..table_count {
                let record_offset = 12 + table_index * 16;
                let source_offset = read_u32(font, record_offset + 8) as usize;
                let source_len = read_u32(font, record_offset + 12) as usize;
                pad_to_four(&mut output);
                let target_offset = output.len();
                output.extend_from_slice(&font[source_offset..source_offset + source_len]);
                output[directory_offset + record_offset + 8..directory_offset + record_offset + 12]
                    .copy_from_slice(&(target_offset as u32).to_be_bytes());
            }
        }
        output
    }

    fn patch_os2_weight(bytes: &mut [u8], weight: u16) {
        let offset = sfnt_table_offset(bytes, b"OS/2").unwrap() + 4;
        bytes[offset..offset + 2].copy_from_slice(&weight.to_be_bytes());
    }

    fn sfnt_table_offset(bytes: &[u8], tag: &[u8; 4]) -> Option<usize> {
        let table_count = u16::from_be_bytes([*bytes.get(4)?, *bytes.get(5)?]) as usize;
        for table_index in 0..table_count {
            let record_offset = 12 + table_index * 16;
            if bytes.get(record_offset..record_offset + 4)? != &tag[..] {
                continue;
            }
            return Some(read_u32(bytes, record_offset + 8) as usize);
        }
        None
    }

    fn read_u32(data: &[u8], offset: usize) -> u32 {
        u32::from_be_bytes(data[offset..offset + 4].try_into().unwrap())
    }

    fn pad_to_four(data: &mut Vec<u8>) {
        while data.len() % 4 != 0 {
            data.push(0);
        }
    }
}
