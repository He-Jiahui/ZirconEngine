//! Generation-owned parsed font source shared by runtime and offline SDF batches.

use std::sync::Arc;

use self_cell::self_cell;
use ttf_parser::Face;

use crate::core::runtime::tasks::{parallel_for, TaskPool};
use crate::text::VariationCoords;

use super::fdsm_gen::{generate_distance_field_glyph_from_face, parse_distance_field_face};
use super::{
    sdf_font_source_hash, sdf_variation_hash, SdfBakeParams, SdfGlyphData, SdfGlyphGenerationError,
};

type ParsedSdfFace<'a> = Face<'a>;

self_cell!(
    struct ParsedSdfFaceCell {
        owner: Arc<[u8]>,

        #[covariant]
        dependent: ParsedSdfFace,
    }
);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct SdfGenerationSourceHandle {
    generation: u64,
    index: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SdfGenerationSourceReport {
    pub(crate) source_byte_len: usize,
    pub(crate) source_hash_count: usize,
    pub(crate) face_parse_count: usize,
    pub(crate) variation_coordinate_count: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SdfGenerationBatchGlyph {
    pub(crate) glyph_id: u16,
    pub(crate) result: Result<SdfGlyphData, SdfGlyphGenerationError>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SdfGenerationBatchReport {
    pub(crate) requested_glyph_count: usize,
    pub(crate) unique_glyph_count: usize,
    pub(crate) duplicate_glyph_count: usize,
    pub(crate) generated_glyph_count: usize,
    pub(crate) failed_glyph_count: usize,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct SdfGenerationBatch {
    pub(crate) glyphs: Vec<SdfGenerationBatchGlyph>,
    pub(crate) report: SdfGenerationBatchReport,
}

pub(crate) struct SdfGenerationSourceContext {
    handle: SdfGenerationSourceHandle,
    source_face_index: u32,
    source_hash: [u8; 32],
    variation_hash: [u8; 32],
    report: SdfGenerationSourceReport,
    parsed: ParsedSdfFaceCell,
}

impl SdfGenerationSourceHandle {
    pub(crate) const fn new(value: u64) -> Self {
        Self {
            generation: 0,
            index: value,
        }
    }

    pub(crate) const fn for_generation(generation: u64, index: u64) -> Self {
        Self { generation, index }
    }
}

impl SdfGenerationSourceContext {
    pub(crate) fn new(
        handle: SdfGenerationSourceHandle,
        font_bytes: Arc<[u8]>,
        face_index: u32,
        variations: Arc<VariationCoords>,
    ) -> Result<Self, SdfGlyphGenerationError> {
        let source_hash = sdf_font_source_hash(font_bytes.as_ref());
        Self::from_hashed_source(handle, font_bytes, source_hash, 1, face_index, variations)
    }

    pub(crate) fn from_hashed_source(
        handle: SdfGenerationSourceHandle,
        font_bytes: Arc<[u8]>,
        source_hash: [u8; 32],
        source_hash_count: usize,
        face_index: u32,
        variations: Arc<VariationCoords>,
    ) -> Result<Self, SdfGlyphGenerationError> {
        let variation_hash = sdf_variation_hash(variations.as_ref());
        let source_byte_len = font_bytes.len();
        let variation_coordinate_count = variations.0.len();
        let parsed = ParsedSdfFaceCell::try_new(font_bytes, |bytes| {
            parse_distance_field_face(bytes.as_ref(), face_index, variations.as_ref())
        })?;
        Ok(Self {
            handle,
            source_face_index: face_index,
            source_hash,
            variation_hash,
            report: SdfGenerationSourceReport {
                source_byte_len,
                source_hash_count,
                face_parse_count: 1,
                variation_coordinate_count,
            },
            parsed,
        })
    }

    pub(crate) const fn handle(&self) -> SdfGenerationSourceHandle {
        self.handle
    }

    pub(crate) const fn source_hash(&self) -> [u8; 32] {
        self.source_hash
    }

    pub(crate) const fn variation_hash(&self) -> [u8; 32] {
        self.variation_hash
    }

    pub(crate) const fn report(&self) -> SdfGenerationSourceReport {
        self.report
    }

    pub(crate) fn with_face<R>(&self, operation: impl FnOnce(&Face<'_>) -> R) -> R {
        self.parsed.with_dependent(|_, face| operation(face))
    }

    pub(crate) fn generate_batch(
        &self,
        params: SdfBakeParams,
        glyph_ids: &[u16],
    ) -> SdfGenerationBatch {
        let requested_glyph_count = glyph_ids.len();
        let mut glyphs = pending_batch_glyphs(glyph_ids);
        self.parsed.with_dependent(|_, face| {
            generate_glyph_slice(face, self.source_face_index, params, &mut glyphs)
        });
        finish_batch(requested_glyph_count, glyphs)
    }

    pub(crate) fn generate_batch_with_pool(
        &self,
        pool: &TaskPool,
        params: SdfBakeParams,
        glyph_ids: &[u16],
    ) -> SdfGenerationBatch {
        let requested_glyph_count = glyph_ids.len();
        let mut glyphs = pending_batch_glyphs(glyph_ids);
        self.parsed.with_dependent(|_, face| {
            parallel_for(pool, &mut glyphs, 1, |chunk| {
                generate_glyph_slice(face, self.source_face_index, params, chunk);
            });
        });
        finish_batch(requested_glyph_count, glyphs)
    }
}

fn pending_batch_glyphs(glyph_ids: &[u16]) -> Vec<SdfGenerationBatchGlyph> {
    let mut unique_glyph_ids = glyph_ids.to_vec();
    unique_glyph_ids.sort_unstable();
    unique_glyph_ids.dedup();
    unique_glyph_ids
        .into_iter()
        .map(|glyph_id| SdfGenerationBatchGlyph {
            glyph_id,
            result: Err(SdfGlyphGenerationError::MissingGlyphOutline(glyph_id)),
        })
        .collect()
}

fn generate_glyph_slice(
    face: &Face<'_>,
    source_face_index: u32,
    params: SdfBakeParams,
    glyphs: &mut [SdfGenerationBatchGlyph],
) {
    for glyph in glyphs {
        glyph.result = generate_distance_field_glyph_from_face(
            face,
            source_face_index,
            glyph.glyph_id,
            params,
        );
    }
}

fn finish_batch(
    requested_glyph_count: usize,
    glyphs: Vec<SdfGenerationBatchGlyph>,
) -> SdfGenerationBatch {
    let unique_glyph_count = glyphs.len();
    let generated_glyph_count = glyphs.iter().filter(|glyph| glyph.result.is_ok()).count();
    SdfGenerationBatch {
        glyphs,
        report: SdfGenerationBatchReport {
            requested_glyph_count,
            unique_glyph_count,
            duplicate_glyph_count: requested_glyph_count.saturating_sub(unique_glyph_count),
            generated_glyph_count,
            failed_glyph_count: unique_glyph_count.saturating_sub(generated_glyph_count),
        },
    }
}
