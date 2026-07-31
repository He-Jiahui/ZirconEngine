use super::{FontDatabase, StoredFontFace, StoredFontSource};
use crate::text::FaceIndex;

impl FontDatabase {
    /// Compares only inputs that can change shaping, fallback, or raster output.
    ///
    /// Face order is significant because `FontFaceId` is an index into `faces`.
    /// Runtime caches, diagnostics, and derived backend indexes are deliberately
    /// excluded: publishing those would invalidate every text cache without
    /// changing a rendered glyph. Shared byte sources take the `Arc::ptr_eq`
    /// fast path; the byte comparison only handles independently materialized
    /// databases that carry the same font payload.
    pub(in crate::text::font) fn has_same_render_inputs(&self, other: &Self) -> bool {
        self.fallback_families == other.fallback_families
            && self.project_composite_font == other.project_composite_font
            && self.default_ui_family == other.default_ui_family
            && self.faces.len() == other.faces.len()
            && self
                .faces
                .iter()
                .zip(&other.faces)
                .all(|(left, right)| stored_face_render_inputs_equal(left, right))
    }
}

fn stored_face_render_inputs_equal(left: &StoredFontFace, right: &StoredFontFace) -> bool {
    left.active == right.active
        && left.descriptor == right.descriptor
        && stored_source_render_inputs_equal(
            &left.source,
            &right.source,
            left.descriptor.face_index,
        )
}

fn stored_source_render_inputs_equal(
    left: &StoredFontSource,
    right: &StoredFontSource,
    face_index: FaceIndex,
) -> bool {
    match (left, right) {
        (StoredFontSource::SharedBytes(left), StoredFontSource::SharedBytes(right)) => {
            std::sync::Arc::ptr_eq(left, right) || left.as_ref() == right.as_ref()
        }
        (StoredFontSource::FontDb { source: left }, StoredFontSource::FontDb { source: right }) => {
            let left = super::super::descriptors::source_key_from_fontdb_source(left, face_index);
            let right = super::super::descriptors::source_key_from_fontdb_source(right, face_index);
            left.is_some() && left == right
        }
        _ => false,
    }
}
