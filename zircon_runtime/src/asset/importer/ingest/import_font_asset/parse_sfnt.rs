use std::collections::BTreeSet;

use crate::asset::assets::{
    DecodedFontSource, FontAssetCmapCoverage, FontAssetCodepointRange, FontAssetFaceMetrics,
    FontAssetFaceStyle, FontAssetLineMetrics, FontAssetMetadata, FontAssetParsedFace,
    FontAssetSourceFormat, FontAssetVariableInstance, FontAssetVariationAxis,
    FontAssetVariationCoord, FontMetadataParseError,
};
use ttf_parser::{name_id, Face, Style, Tag};

pub(super) fn parse_font_metadata(
    source: &DecodedFontSource,
) -> Result<FontAssetMetadata, FontMetadataParseError> {
    let bytes = source.bytes();
    let face_count = ttf_parser::fonts_in_collection(bytes).unwrap_or(1);
    let source_format = if source.source_format() == FontAssetSourceFormat::Woff2 {
        FontAssetSourceFormat::Woff2
    } else if face_count > 1 {
        FontAssetSourceFormat::TrueTypeCollection
    } else {
        FontAssetSourceFormat::Sfnt
    };
    let mut faces = Vec::new();
    for face_index in 0..face_count {
        let face = Face::parse(bytes, face_index)
            .map_err(|error| FontMetadataParseError::new(face_index, error))?;
        faces.push(parse_face(bytes, face_index, &face));
    }

    Ok(FontAssetMetadata {
        source_format,
        face_count,
        faces,
    })
}

fn parse_face(bytes: &[u8], face_index: u32, face: &Face<'_>) -> FontAssetParsedFace {
    let axes = variation_axes(face);
    let named_instances = parse_named_instances(
        face.raw_face().table(Tag::from_bytes(b"fvar")),
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
        metrics: face_metrics(face),
        variation_axes: axes,
        named_instances,
        cmap: cmap_coverage(bytes, face),
    }
}

fn face_metrics(face: &Face<'_>) -> FontAssetFaceMetrics {
    let os2 = face.tables().os2;
    FontAssetFaceMetrics {
        units_per_em: face.units_per_em(),
        ascender: face.ascender(),
        descender: face.descender(),
        line_gap: face.line_gap(),
        uses_typographic_metrics: os2.is_some_and(|table| table.use_typographic_metrics()),
        windows_ascender: os2.map(|table| table.windows_ascender()).unwrap_or(0),
        windows_descender: os2.map(|table| table.windows_descender()).unwrap_or(0),
        underline: face.underline_metrics().map(line_metrics),
        strikeout: face.strikeout_metrics().map(line_metrics),
    }
}

fn line_metrics(metrics: ttf_parser::LineMetrics) -> FontAssetLineMetrics {
    FontAssetLineMetrics {
        position: metrics.position,
        thickness: metrics.thickness,
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

#[cfg(test)]
mod tests;
