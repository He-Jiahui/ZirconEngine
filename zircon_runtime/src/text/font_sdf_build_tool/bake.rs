//! Offline font distance-field baking.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use ttf_parser::Face;

use crate::asset::assets::{decode_font_source, standalone_sfnt_face};
use crate::core::math::UVec2;
use crate::core::runtime::tasks::TaskPool;
use crate::text::VariationCoords;
use crate::text::sdf::{
    SdfBakeParams, SdfGenerationSourceContext, SdfGenerationSourceHandle, SdfMode,
    SdfOfflineArtifact, SdfOfflineArtifactIdentity, sdf_offline_artifact_path,
};

use super::pack::{GeneratedGlyph, pack_generated_glyphs};
use super::{FontSdfBakeError, FontSdfBakeMode, FontSdfBakeRequest, FontSdfGlyphSelection};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FontSdfBakeReport {
    pub requested_codepoint_count: usize,
    pub mapped_glyph_count: usize,
    pub generated_glyph_count: usize,
    pub skipped_glyph_count: usize,
    pub page_count: usize,
    pub encoded_len: usize,
    pub source_context_count: usize,
    pub source_hash_count: usize,
    pub face_parse_count: usize,
    pub generation_batch_count: usize,
    pub generation_requested_glyph_count: usize,
    pub generation_unique_glyph_count: usize,
    pub generation_duplicate_glyph_count: usize,
    pub generation_worker_count: usize,
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
    generation_pool: &TaskPool,
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
    let params = SdfBakeParams {
        mode: mode(request.mode),
        bake_em_px: request.bake_em_px,
        spread_px_milli: request.spread_px_milli,
    }
    .normalized();
    let source = SdfGenerationSourceContext::new(
        SdfGenerationSourceHandle::new(0),
        Arc::from(standalone.into_boxed_slice()),
        0,
        Arc::new(VariationCoords::default()),
    )
    .map_err(|error| FontSdfBakeError::ParseFace(error.to_string()))?;
    let (codepoints, glyph_map) = source.with_face(|face| {
        let codepoints = selected_codepoints(face, &request.selection)?;
        let glyph_map = mapped_glyphs(face, &codepoints);
        Ok::<_, FontSdfBakeError>((codepoints, glyph_map))
    })?;
    if glyph_map.is_empty() {
        return Err(FontSdfBakeError::NoMappedGlyphs);
    }

    let generation = source.generate_batch_with_pool(
        generation_pool,
        params,
        &glyph_map.keys().copied().collect::<Vec<_>>(),
    );
    let skipped_glyph_count = generation.report.failed_glyph_count;
    let mut generated = Vec::with_capacity(glyph_map.len());
    for glyph in generation.glyphs {
        match glyph.result {
            Ok(data) => generated.push(GeneratedGlyph {
                codepoint: glyph_map[&glyph.glyph_id],
                glyph_id: u32::from(glyph.glyph_id),
                data,
            }),
            Err(_) => {}
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
        variation_hash: request.variation_hash.into(),
        source_hash: source.source_hash(),
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
        source_context_count: 1,
        source_hash_count: source.report().source_hash_count,
        face_parse_count: source.report().face_parse_count,
        generation_batch_count: 1,
        generation_requested_glyph_count: generation.report.requested_glyph_count,
        generation_unique_glyph_count: generation.report.unique_glyph_count,
        generation_duplicate_glyph_count: generation.report.duplicate_glyph_count,
        generation_worker_count: generation_pool.parallelism(),
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
