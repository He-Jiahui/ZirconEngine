//! Offline font distance-field baking.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use ttf_parser::Face;

use crate::asset::assets::{decode_font_source, standalone_sfnt_face};
use crate::core::math::UVec2;
use crate::graphics::text::sdf::{
    generate_distance_field_glyph, sdf_font_source_hash, sdf_offline_artifact_path, SdfBakeParams,
    SdfMode, SdfOfflineArtifact, SdfOfflineArtifactIdentity,
};

use super::pack::{pack_generated_glyphs, GeneratedGlyph};
use super::{FontSdfBakeError, FontSdfBakeMode, FontSdfBakeRequest, FontSdfGlyphSelection};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FontSdfBakeReport {
    pub requested_codepoint_count: usize,
    pub mapped_glyph_count: usize,
    pub generated_glyph_count: usize,
    pub skipped_glyph_count: usize,
    pub page_count: usize,
    pub encoded_len: usize,
}

pub struct FontSdfBakeArtifact {
    identity: SdfOfflineArtifactIdentity,
    encoded: Vec<u8>,
    report: FontSdfBakeReport,
}

impl FontSdfBakeArtifact {
    pub fn bytes(&self) -> &[u8] {
        &self.encoded
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.encoded
    }

    pub fn report(&self) -> FontSdfBakeReport {
        self.report
    }

    pub fn artifact_path(&self, cache_root: impl AsRef<Path>) -> PathBuf {
        sdf_offline_artifact_path(cache_root.as_ref(), &self.identity)
    }
}

pub fn bake_font_sdf_artifact(
    font_bytes: &[u8],
    request: &FontSdfBakeRequest,
) -> Result<FontSdfBakeArtifact, FontSdfBakeError> {
    request.validate()?;
    let decoded = decode_font_source(font_bytes.to_vec())
        .map_err(|error| FontSdfBakeError::DecodeFont(error.to_string()))?;
    let standalone =
        standalone_sfnt_face(decoded.bytes(), request.face_index).map_err(|error| {
            FontSdfBakeError::ExtractFace {
                face_index: request.face_index,
                message: error.to_string(),
            }
        })?;
    let face = Face::parse(&standalone, 0)
        .map_err(|error| FontSdfBakeError::ParseFace(error.to_string()))?;
    let codepoints = selected_codepoints(&face, &request.selection)?;
    let glyph_map = mapped_glyphs(&face, &codepoints);
    if glyph_map.is_empty() {
        return Err(FontSdfBakeError::NoMappedGlyphs);
    }

    let params = SdfBakeParams {
        mode: mode(request.mode),
        bake_em_px: request.bake_em_px,
        spread_px_milli: request.spread_px_milli,
    }
    .normalized();
    let mut skipped_glyph_count = 0_usize;
    let mut generated = Vec::with_capacity(glyph_map.len());
    for (glyph_id, codepoint) in &glyph_map {
        match generate_distance_field_glyph(&standalone, 0, *glyph_id, params) {
            Ok(data) => generated.push(GeneratedGlyph {
                codepoint: *codepoint,
                glyph_id: u32::from(*glyph_id),
                data,
            }),
            Err(_) => skipped_glyph_count = skipped_glyph_count.saturating_add(1),
        }
    }
    if generated.is_empty() {
        return Err(FontSdfBakeError::NoGeneratedGlyphs {
            skipped_count: skipped_glyph_count,
        });
    }
    let (pages, glyphs) = pack_generated_glyphs(generated, request.page_size)?;
    let identity = SdfOfflineArtifactIdentity {
        asset_guid: request.asset_guid.clone(),
        face_index: request.face_index,
        variation_hash: request.variation_hash,
        source_hash: sdf_font_source_hash(&standalone),
        params,
    };
    let artifact = SdfOfflineArtifact::new(
        identity.clone(),
        UVec2::splat(request.page_size),
        pages,
        glyphs,
    )
    .map_err(|error| FontSdfBakeError::Artifact(error.to_string()))?;
    let page_count = artifact.pages().len();
    let generated_glyph_count = artifact.glyphs().len();
    let encoded = artifact
        .encode()
        .map_err(|error| FontSdfBakeError::Artifact(error.to_string()))?;
    let report = FontSdfBakeReport {
        requested_codepoint_count: codepoints.len(),
        mapped_glyph_count: glyph_map.len(),
        generated_glyph_count,
        skipped_glyph_count,
        page_count,
        encoded_len: encoded.len(),
    };
    Ok(FontSdfBakeArtifact {
        identity,
        encoded,
        report,
    })
}

fn selected_codepoints(
    face: &Face<'_>,
    selection: &FontSdfGlyphSelection,
) -> Result<Vec<u32>, FontSdfBakeError> {
    let mut codepoints = BTreeSet::new();
    match selection {
        FontSdfGlyphSelection::AllCmap => {
            let cmap = face.tables().cmap.ok_or(FontSdfBakeError::NoMappedGlyphs)?;
            for subtable in cmap
                .subtables
                .into_iter()
                .filter(|table| table.is_unicode())
            {
                subtable.codepoints(|codepoint| {
                    if char::from_u32(codepoint).is_some() {
                        codepoints.insert(codepoint);
                    }
                });
            }
        }
        FontSdfGlyphSelection::Codepoints(values) => {
            for codepoint in values {
                if char::from_u32(*codepoint).is_none() {
                    return Err(FontSdfBakeError::InvalidCodepoint(*codepoint));
                }
                codepoints.insert(*codepoint);
            }
        }
    }
    if codepoints.is_empty() {
        return Err(FontSdfBakeError::EmptySelection);
    }
    Ok(codepoints.into_iter().collect())
}

fn mapped_glyphs(face: &Face<'_>, codepoints: &[u32]) -> BTreeMap<u16, u32> {
    let mut glyphs = BTreeMap::new();
    for codepoint in codepoints {
        if let Some(glyph_id) =
            char::from_u32(*codepoint).and_then(|scalar| face.glyph_index(scalar))
        {
            glyphs.entry(glyph_id.0).or_insert(*codepoint);
        }
    }
    glyphs
}

fn mode(mode: FontSdfBakeMode) -> SdfMode {
    match mode {
        FontSdfBakeMode::Sdf => SdfMode::Sdf,
        FontSdfBakeMode::Msdf => SdfMode::Msdf,
        FontSdfBakeMode::Mtsdf => SdfMode::Mtsdf,
    }
}
