use std::sync::Arc;

use sha2::{Digest, Sha256};
use ttf_parser::{name_id, Face, GlyphId, Style as TtfStyle};

use crate::asset::{FontAssetFaceMetrics, FontAssetLineMetrics};
use crate::text::{FontFamilyName, FontStretch, FontStyle, FontWeight, VariationCoords};

use super::coverage::FontCoverage;
use super::descriptors::stretch_from_ttf_width_class;
use super::instance::{canonical_variation_coords, quantized_axis_value};

const FONT_FACE_SOURCE_HASH_DOMAIN: &[u8] = b"zircon-font-face-source-v1";
const WEIGHT_AXIS_TAG: u32 = u32::from_be_bytes(*b"wght");

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct FontVariationAxis {
    pub(super) tag: u32,
    pub(super) min_value: f32,
    pub(super) default_value: f32,
    pub(super) max_value: f32,
}

/// Owned, generation-local projection of the SFNT tables used by shaping,
/// raster, fallback, vertical layout, and text decoration consumers.
///
/// `ttf_parser::Face` borrows the source bytes, so retaining it in the shared
/// database would couple lifetimes across backends. This artifact eagerly
/// copies the small table projections and the optional vertical advance array
/// once, allowing every hot consumer to remain parse-free.
#[derive(Clone, Debug)]
pub(super) struct FontFaceMetadata {
    parsed: bool,
    family: Option<FontFamilyName>,
    postscript_name: Option<String>,
    weight: FontWeight,
    style: FontStyle,
    stretch: FontStretch,
    axes: Arc<[FontVariationAxis]>,
    face_metrics: Option<FontAssetFaceMetrics>,
    vertical_advances: Arc<[u16]>,
    glyph_map: FontGlyphMap,
    coverage: FontCoverage,
    source_identity: [u8; 16],
    resource_sha256: Option<[u8; 32]>,
    raster_sha256: [u8; 32],
}

impl FontFaceMetadata {
    pub(super) fn from_sfnt_bytes(bytes: &[u8], face_index: u32) -> Self {
        let source_identity = source_identity(bytes, face_index);
        let raster_sha256 = Sha256::digest(bytes).into();
        let Ok(face) = Face::parse(bytes, face_index) else {
            return Self::unknown_with_raster_sha256(source_identity, raster_sha256);
        };
        let axes = face
            .variation_axes()
            .into_iter()
            .map(|axis| FontVariationAxis {
                tag: u32::from_be_bytes(axis.tag.to_bytes()),
                min_value: axis.min_value,
                default_value: axis.def_value,
                max_value: axis.max_value,
            })
            .collect::<Vec<_>>();
        let vertical_advances = vertical_advances(&face);
        let glyph_map = FontGlyphMap::from_face(&face);
        let coverage = glyph_map.coverage();
        Self {
            parsed: true,
            family: face_family_name(&face).map(FontFamilyName::from),
            postscript_name: ttf_name_by_id(&face, name_id::POST_SCRIPT_NAME),
            weight: FontWeight::clamped(face.weight().to_number()),
            style: style_from_ttf(face.style()),
            stretch: stretch_from_ttf_width_class(face.width().to_number()),
            axes: Arc::from(axes.into_boxed_slice()),
            face_metrics: Some(face_metrics(&face)),
            vertical_advances,
            glyph_map,
            coverage,
            source_identity,
            resource_sha256: None,
            raster_sha256,
        }
    }

    pub(super) fn with_resource_sha256(mut self, resource_sha256: [u8; 32]) -> Self {
        self.resource_sha256 = Some(resource_sha256);
        self
    }

    pub(super) fn with_coverage(mut self, coverage: FontCoverage) -> Self {
        self.coverage = coverage;
        self
    }

    pub(super) fn discovered_family(&self) -> Option<&FontFamilyName> {
        self.family.as_ref()
    }

    pub(super) fn family_name(&self) -> Option<&str> {
        self.family.as_ref().map(|family| family.as_str())
    }

    pub(super) fn postscript_name(&self) -> Option<&str> {
        self.postscript_name.as_deref()
    }

    pub(super) const fn weight(&self) -> FontWeight {
        self.weight
    }

    pub(super) const fn style(&self) -> FontStyle {
        self.style
    }

    pub(super) const fn stretch(&self) -> FontStretch {
        self.stretch
    }

    pub(super) const fn face_metrics(&self) -> Option<FontAssetFaceMetrics> {
        self.face_metrics
    }

    pub(super) const fn coverage(&self) -> &FontCoverage {
        &self.coverage
    }

    pub(super) fn glyph_id(&self, codepoint: char) -> Option<u16> {
        self.glyph_map.glyph_id(codepoint)
    }

    pub(super) const fn source_identity(&self) -> [u8; 16] {
        self.source_identity
    }

    pub(super) const fn resource_sha256(&self) -> Option<[u8; 32]> {
        self.resource_sha256
    }

    pub(super) const fn raster_sha256(&self) -> [u8; 32] {
        self.raster_sha256
    }

    pub(super) fn vertical_advance(&self, glyph_id: u32) -> Option<u16> {
        let glyph_id = usize::try_from(glyph_id).ok()?;
        self.vertical_advances
            .get(glyph_id)
            .copied()
            .filter(|advance| *advance > 0)
    }

    pub(super) fn effective_variations(
        &self,
        variations: &VariationCoords,
        font_weight: Option<u16>,
    ) -> VariationCoords {
        if !self.parsed {
            return canonical_variation_coords(variations).unwrap_or_else(|_| variations.clone());
        }
        if self.axes.is_empty() {
            return VariationCoords::default();
        }

        let mut weighted = variations.clone();
        let weight_axis =
            font_weight.and_then(|_| self.axes.iter().find(|axis| axis.tag == WEIGHT_AXIS_TAG));
        if let (Some(font_weight), Some(_)) = (font_weight, weight_axis) {
            weighted.0.push((WEIGHT_AXIS_TAG, f32::from(font_weight)));
        }
        let Ok(weighted) = canonical_variation_coords(&weighted) else {
            return variations.clone();
        };
        VariationCoords(
            weighted
                .0
                .into_iter()
                .filter_map(|(tag, value)| {
                    let axis = if tag == WEIGHT_AXIS_TAG {
                        weight_axis.or_else(|| self.axes.iter().find(|axis| axis.tag == tag))?
                    } else {
                        self.axes.iter().find(|axis| axis.tag == tag)?
                    };
                    let value = quantized_axis_value(
                        value,
                        axis.min_value,
                        axis.default_value,
                        axis.max_value,
                    );
                    (value != axis.default_value).then_some((tag, value))
                })
                .collect(),
        )
    }

    #[cfg(test)]
    fn unknown(source_identity: [u8; 16]) -> Self {
        Self::unknown_with_raster_sha256(source_identity, [0; 32])
    }

    fn unknown_with_raster_sha256(source_identity: [u8; 16], raster_sha256: [u8; 32]) -> Self {
        Self {
            parsed: false,
            family: None,
            postscript_name: None,
            weight: FontWeight::NORMAL,
            style: FontStyle::Normal,
            stretch: FontStretch::NORMAL,
            axes: Arc::from([]),
            face_metrics: None,
            vertical_advances: Arc::from([]),
            glyph_map: FontGlyphMap::default(),
            coverage: FontCoverage::Unknown,
            source_identity,
            resource_sha256: None,
            raster_sha256,
        }
    }
}

#[derive(Clone, Debug, Default)]
struct FontGlyphMap {
    entries: Arc<[(u32, u16)]>,
}

impl FontGlyphMap {
    fn from_face(face: &Face<'_>) -> Self {
        let Some(cmap) = face.tables().cmap else {
            return Self::default();
        };
        let mut entries = Vec::new();
        for subtable in cmap.subtables {
            if !subtable.is_unicode() {
                continue;
            }
            subtable.codepoints(|codepoint| {
                let Some(glyph_id) = char::from_u32(codepoint).and_then(|ch| face.glyph_index(ch))
                else {
                    return;
                };
                entries.push((codepoint, glyph_id.0));
            });
        }
        entries.sort_unstable_by_key(|(codepoint, _)| *codepoint);
        entries.dedup_by_key(|(codepoint, _)| *codepoint);
        Self {
            entries: Arc::from(entries.into_boxed_slice()),
        }
    }

    fn coverage(&self) -> FontCoverage {
        FontCoverage::from_sorted_unique_codepoints(
            self.entries.iter().map(|(codepoint, _)| *codepoint),
        )
    }

    fn glyph_id(&self, codepoint: char) -> Option<u16> {
        self.entries
            .binary_search_by_key(&(codepoint as u32), |(codepoint, _)| *codepoint)
            .ok()
            .map(|index| self.entries[index].1)
    }
}

fn vertical_advances(face: &Face<'_>) -> Arc<[u16]> {
    let glyph_count = face.number_of_glyphs();
    if glyph_count == 0 || face.glyph_ver_advance(GlyphId(0)).is_none() {
        return Arc::from([]);
    }
    Arc::from(
        (0..glyph_count)
            .map(|glyph_id| face.glyph_ver_advance(GlyphId(glyph_id)).unwrap_or(0))
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    )
}

fn face_metrics(face: &Face<'_>) -> FontAssetFaceMetrics {
    FontAssetFaceMetrics {
        units_per_em: face.units_per_em(),
        ascender: face.ascender(),
        descender: face.descender(),
        line_gap: face.line_gap(),
        uses_typographic_metrics: face
            .tables()
            .os2
            .is_some_and(|table| table.use_typographic_metrics()),
        windows_ascender: face
            .tables()
            .os2
            .map(|table| table.windows_ascender())
            .unwrap_or(0),
        windows_descender: face
            .tables()
            .os2
            .map(|table| table.windows_descender())
            .unwrap_or(0),
        underline: face.underline_metrics().map(asset_line_metrics),
        strikeout: face.strikeout_metrics().map(asset_line_metrics),
    }
}

fn asset_line_metrics(metrics: ttf_parser::LineMetrics) -> FontAssetLineMetrics {
    FontAssetLineMetrics {
        position: metrics.position,
        thickness: metrics.thickness,
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

fn source_identity(bytes: &[u8], face_index: u32) -> [u8; 16] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(FONT_FACE_SOURCE_HASH_DOMAIN);
    hasher.update(&face_index.to_le_bytes());
    hasher.update(bytes);
    let mut identity = [0_u8; 16];
    identity.copy_from_slice(&hasher.finalize().as_bytes()[..16]);
    identity
}

#[cfg(test)]
#[path = "face_metadata/tests/cached_weight_axis_tests.rs"]
mod cached_weight_axis_tests;

#[cfg(test)]
#[path = "tests/face_metadata.rs"]
mod tests;
