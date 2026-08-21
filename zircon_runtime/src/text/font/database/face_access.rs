use std::sync::atomic::Ordering;
use std::sync::Arc;

use crate::asset::assets::standalone_sfnt_face;
use crate::asset::FontAssetFaceMetrics;
use crate::text::FontFaceId;

use super::{FontDatabase, FontDatabaseError, StoredFontSource};
use crate::text::font::coverage::FontCoverage;
use crate::text::font::face_metadata::FontFaceMetadata;

impl FontDatabase {
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
        match &stored.source {
            StoredFontSource::SharedBytes(bytes) => {
                FontFaceMetadata::from_sfnt_bytes(bytes.as_ref(), stored.descriptor.face_index)
            }
            StoredFontSource::FontDb { .. } => self
                .backend_face_id(face)
                .and_then(|backend| {
                    self.backend_database
                        .with_face_data(backend, |bytes, face_index| {
                            FontFaceMetadata::from_sfnt_bytes(bytes, face_index)
                        })
                })
                .unwrap_or_else(|| {
                    FontFaceMetadata::from_sfnt_bytes(&[], stored.descriptor.face_index)
                }),
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

/// Joiners, variation selectors, and emoji tags participate in shaping
/// sequences but do not require standalone glyphs in a font's ordinary cmap.
/// Keeping them in the fallback cache key preserves sequence identity while
/// excluding them from face coverage avoids rejecting a font that can shape
/// the sequence through GSUB or a Unicode variation subtable.
fn codepoint_requires_font_coverage(codepoint: char) -> bool {
    !matches!(
        codepoint,
        '\u{200C}'
            | '\u{200D}'
            | '\u{FE00}'..='\u{FE0F}'
            | '\u{E0020}'..='\u{E007F}'
            | '\u{E0100}'..='\u{E01EF}'
    )
}
