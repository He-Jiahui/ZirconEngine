use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::asset::assets::standalone_sfnt_face;
use crate::asset::FontAssetFaceMetrics;
use crate::text::FontFaceId;

use super::{FontDatabase, FontDatabaseError, StoredFontSource};
use crate::text::font::coverage::FontCoverage;
use crate::text::font::face_metadata::FontFaceMetadata;

impl FontDatabase {
    pub(crate) fn face_receipt_metadata(
        &self,
        face: FontFaceId,
    ) -> Result<FontFaceReceiptMetadata, FontDatabaseError> {
        let stored = self
            .face(face)
            .ok_or(FontDatabaseError::UnknownFace(face))?;
        let metadata = self.face_metadata(face)?;
        Ok(FontFaceReceiptMetadata {
            family_name: metadata.family_name().map(str::to_owned),
            postscript_name: metadata.postscript_name().map(str::to_owned),
            face_index: stored.descriptor.face_index,
            resource_path: stored.resource_path.clone(),
            resource_sha256: stored.resource_sha256.or(metadata.resource_sha256()),
            raster_sha256: metadata.raster_sha256(),
        })
    }

    pub(crate) fn face_bytes(&self, face: FontFaceId) -> Result<Arc<[u8]>, FontDatabaseError> {
        let stored = self
            .face(face)
            .ok_or(FontDatabaseError::UnknownFace(face))?;
        if let Some(bytes) = stored.source_bytes.get() {
            return Ok(Arc::clone(bytes));
        }
        let materialized = match &stored.source {
            StoredFontSource::SharedBytes(bytes) => Arc::clone(bytes),
            StoredFontSource::FontDb { .. } => {
                let backend = self
                    .backend_face_id(face)
                    .ok_or(FontDatabaseError::BackendFaceUnavailable(face))?;
                self.backend_database
                    .with_face_data(backend, |bytes, _| Arc::<[u8]>::from(bytes))
                    .ok_or(FontDatabaseError::FaceBytesUnavailable(face))?
            }
        };
        let _ = stored.source_bytes.set(Arc::clone(&materialized));
        Ok(stored.source_bytes.get().map_or(materialized, Arc::clone))
    }

    pub(crate) fn face_index(&self, face: FontFaceId) -> Result<u32, FontDatabaseError> {
        Ok(self
            .face(face)
            .ok_or(FontDatabaseError::UnknownFace(face))?
            .descriptor
            .face_index)
    }

    pub(in crate::text::font) fn face_metadata(
        &self,
        face: FontFaceId,
    ) -> Result<&FontFaceMetadata, FontDatabaseError> {
        let stored = self
            .face(face)
            .ok_or(FontDatabaseError::UnknownFace(face))?;
        Ok(stored.metadata.get_or_init(|| {
            self.metadata_build_count.fetch_add(1, Ordering::Relaxed);
            self.load_face_metadata(face)
        }))
    }

    fn load_face_metadata(&self, face: FontFaceId) -> FontFaceMetadata {
        let Some(stored) = self.face(face) else {
            return FontFaceMetadata::from_sfnt_bytes(&[], 0);
        };
        let face_index = stored.descriptor.face_index;
        let has_resource_path = stored.resource_path.is_some();
        let is_fontdb_source = matches!(&stored.source, StoredFontSource::FontDb { .. });
        let admitted_sha256 = stored.resource_sha256;
        let Ok(bytes) = self.face_bytes(face) else {
            return FontFaceMetadata::from_sfnt_bytes(&[], face_index);
        };
        let metadata = FontFaceMetadata::from_sfnt_bytes(bytes.as_ref(), face_index);
        if let Some(sha256) = admitted_sha256.or_else(|| {
            (has_resource_path && is_fontdb_source).then(|| Sha256::digest(bytes.as_ref()).into())
        }) {
            metadata.with_resource_sha256(sha256)
        } else {
            metadata
        }
    }

    pub(crate) fn face_metrics(
        &self,
        face: FontFaceId,
    ) -> Result<Option<FontAssetFaceMetrics>, FontDatabaseError> {
        Ok(self.face_metadata(face)?.face_metrics())
    }

    pub(crate) fn face_source_identity(
        &self,
        face: FontFaceId,
    ) -> Result<[u8; 16], FontDatabaseError> {
        Ok(self.face_metadata(face)?.source_identity())
    }

    pub(in crate::text) fn face_glyph_id(
        &self,
        face: FontFaceId,
        codepoint: char,
    ) -> Result<Option<u16>, FontDatabaseError> {
        self.face_metadata(face)
            .map(|metadata| metadata.glyph_id(codepoint))
    }

    pub(crate) fn face_metadata_build_count(&self) -> u64 {
        self.metadata_build_count.load(Ordering::Relaxed)
    }

    pub(crate) fn standalone_face_bytes(
        &self,
        face: FontFaceId,
    ) -> Result<Arc<[u8]>, FontDatabaseError> {
        let bytes = self.face_bytes(face)?;
        let face_index = self.face_index(face)?;
        if face_index == 0 && !bytes.starts_with(b"ttcf") {
            return Ok(bytes);
        }
        let stored = self
            .face(face)
            .ok_or(FontDatabaseError::UnknownFace(face))?;
        if let Some(bytes) = stored.standalone_bytes.get() {
            return Ok(Arc::clone(bytes));
        }
        let materialized = standalone_sfnt_face(bytes.as_ref(), face_index)
            .map(|bytes| Arc::from(bytes.into_boxed_slice()))
            .map_err(|source| FontDatabaseError::FaceExtraction { face_index, source })?;
        let _ = stored.standalone_bytes.set(Arc::clone(&materialized));
        Ok(stored
            .standalone_bytes
            .get()
            .map_or(materialized, Arc::clone))
    }

    pub(in crate::text::font) fn face_covers_all(
        &self,
        face: FontFaceId,
        codepoints: &[char],
    ) -> bool {
        codepoints
            .iter()
            .all(|codepoint| self.face_covers_codepoint(face, *codepoint))
    }

    pub(in crate::text::font) fn face_covers_codepoint(
        &self,
        face: FontFaceId,
        codepoint: char,
    ) -> bool {
        if !codepoint_requires_font_coverage(codepoint) {
            return true;
        }
        self.record_fallback_coverage_probe();
        self.coverage_for(face)
            .is_some_and(|coverage| coverage.contains(codepoint))
    }

    pub(in crate::text::font) fn face_coverage_count(
        &self,
        face: FontFaceId,
        codepoints: &[char],
    ) -> usize {
        codepoints
            .iter()
            .filter(|codepoint| codepoint_requires_font_coverage(**codepoint))
            .filter(|codepoint| self.face_covers_codepoint(face, **codepoint))
            .count()
    }

    fn coverage_for(&self, face: FontFaceId) -> Option<&FontCoverage> {
        self.face_metadata(face)
            .ok()
            .map(FontFaceMetadata::coverage)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FontFaceReceiptMetadata {
    pub(crate) family_name: Option<String>,
    pub(crate) postscript_name: Option<String>,
    pub(crate) face_index: u32,
    pub(crate) resource_path: Option<PathBuf>,
    pub(crate) resource_sha256: Option<[u8; 32]>,
    pub(crate) raster_sha256: [u8; 32],
}

/// Joiners, variation selectors, and emoji tags participate in shaping
/// sequences but do not require standalone glyphs in a font's ordinary cmap.
/// Keeping them in the fallback cache key preserves sequence identity while
/// excluding them from face coverage avoids rejecting a font that can shape
/// the sequence through GSUB or a Unicode variation subtable.
pub(in crate::text::font) fn codepoint_requires_font_coverage(codepoint: char) -> bool {
    !matches!(
        codepoint,
        '\u{200C}'
            | '\u{200D}'
            | '\u{FE00}'..='\u{FE0F}'
            | '\u{E0020}'..='\u{E007F}'
            | '\u{E0100}'..='\u{E01EF}'
    )
}
