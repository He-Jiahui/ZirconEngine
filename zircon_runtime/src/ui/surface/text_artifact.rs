use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fmt;
use std::sync::Arc;

use zircon_runtime_interface::ui::surface::{UiResolvedTextLayout, UiResolvedTextLine};

use crate::core::framework::text::{TextFontFaceHandle, TextGlyph};
use crate::text::font::{
    FontCollectionSnapshot, FontHandleResolverSnapshot, resolve_font_handle_batch_from_snapshot,
};
use crate::text::{
    ResolvedTextGlyphArtifact, VariationCoords, resolve_resolved_text_glyph_artifact,
    resolved_text_line_requires_visual_fallback,
};

/// A runtime-owned, zero-copy visual glyph line that is valid only for its matching resolved layout.
///
/// The opaque artifact stays inside the runtime. Consumers receive the canonical `TextGlyph` slice
/// through a retained font-generation lease, so an in-flight frame never reinterprets glyph IDs
/// against a different font database after publication.
#[derive(Clone, Debug)]
pub struct UiResolvedTextGlyphArtifactLine {
    artifact: Arc<ResolvedTextGlyphArtifact>,
    line_index: usize,
    font_collection: FontCollectionSnapshot,
    font_handles: FontHandleResolverSnapshot,
}

/// Current process-owner font publication generation for Editor-host and standalone consumers.
///
/// Process-owned consumers must include this in their cache identity when they hold a raster face
/// beyond a single layout call. Core/session consumers use their injected collection revision
/// instead. A generation change invalidates glyph IDs and source snapshots together.
pub fn current_resolved_text_font_generation() -> u64 {
    crate::text::font::shared_font_database_generation()
}

impl UiResolvedTextGlyphArtifactLine {
    /// Returns the immutable, visual-order glyph slice retained by this generation lease.
    pub fn glyphs(&self) -> Option<&[TextGlyph]> {
        self.artifact
            .lines
            .get(self.line_index)
            .and_then(Option::as_ref)
            .map(|line| line.glyphs.as_slice())
    }

    /// Returns the layout line paired with this leased glyph slice.
    pub fn layout_line(&self) -> Option<&UiResolvedTextLine> {
        self.artifact
            .lines
            .get(self.line_index)
            .and_then(Option::as_ref)
            .map(|line| &line.layout_line)
    }

    /// The text-owned font generation used for cache identity by a consuming raster path.
    pub const fn font_generation(&self) -> u64 {
        self.font_collection.generation()
    }

    /// Returns whether two line views borrow the same immutable resolved-layout artifact.
    ///
    /// A multi-line raster consumer must reject a mixed collection instead of resolving faces from
    /// one layout and glyphs from another layout that happens to share the same font generation.
    pub fn shares_artifact_layout_with(&self, other: &Self) -> bool {
        self.font_collection.collection_id() == other.font_collection.collection_id()
            && self.font_collection.generation() == other.font_collection.generation()
            && Arc::ptr_eq(&self.artifact, &other.artifact)
    }

    /// Clones the immutable font-generation lease for all artifact lines in this resolved layout.
    ///
    /// Both clones are O(1) `Arc` operations; font bytes and the handle registry are not copied.
    pub fn face_snapshot(&self) -> Option<UiTextGlyphArtifactFaceSnapshot> {
        crate::profile_scope!("runtime", "text.surface", "artifact_face_snapshot");
        self.glyphs()?;
        Some(UiTextGlyphArtifactFaceSnapshot {
            font_collection: self.font_collection.clone(),
            font_handles: self.font_handles.clone(),
        })
    }

    /// Resolves every raster glyph face from this line using a matching layout-scoped font snapshot.
    ///
    /// The returned table preserves the source face, TTC collection index, and effective instance
    /// variations recorded by shaping. A missing handle, a generation race, or an inconsistent
    /// face/instance pair rejects the complete line so consumers can retain their fallback path.
    pub fn raster_faces_from_snapshot(
        &self,
        snapshot: &UiTextGlyphArtifactFaceSnapshot,
    ) -> Option<UiTextGlyphArtifactRasterFaces> {
        self.raster_faces_from_line_indices_with_snapshot(snapshot, [self.line_index])
    }

    /// Resolves the exact raster faces for every line in this immutable artifact with one batch.
    ///
    /// Callers use this only after proving their visual lines all share this artifact through
    /// `shares_artifact_layout_with`. The batch de-duplicates exact `(face, instance)` pairs over
    /// the complete artifact, matching a shaped sequence's layout-level face ownership rather than
    /// acquiring one font-handle registry snapshot per wrapped visual line.
    pub fn artifact_raster_faces_from_snapshot(
        &self,
        snapshot: &UiTextGlyphArtifactFaceSnapshot,
    ) -> Option<UiTextGlyphArtifactRasterFaces> {
        self.raster_faces_from_line_indices_with_snapshot(snapshot, 0..self.artifact.lines.len())
    }

    /// Resolves all raster faces from this immutable artifact without cloning a database for an
    /// artifact that has no raster glyphs.
    pub fn artifact_raster_faces(&self) -> Option<UiTextGlyphArtifactRasterFaces> {
        self.raster_faces_from_line_indices(0..self.artifact.lines.len())
    }

    fn raster_faces_from_line_indices(
        &self,
        line_indices: impl IntoIterator<Item = usize>,
    ) -> Option<UiTextGlyphArtifactRasterFaces> {
        crate::profile_scope!("runtime", "text.surface", "artifact_raster_face_resolution");
        let (pairs, pair_indices) = self.raster_face_pairs_from_line_indices(line_indices)?;
        if pairs.is_empty() {
            return Some(UiTextGlyphArtifactRasterFaces {
                font_generation: self.font_generation(),
                faces: Vec::new(),
                pair_indices,
            });
        }

        let snapshot = self.face_snapshot()?;
        self.raster_faces_from_pairs(&snapshot, pairs, pair_indices)
    }

    fn raster_faces_from_line_indices_with_snapshot(
        &self,
        snapshot: &UiTextGlyphArtifactFaceSnapshot,
        line_indices: impl IntoIterator<Item = usize>,
    ) -> Option<UiTextGlyphArtifactRasterFaces> {
        crate::profile_scope!("runtime", "text.surface", "artifact_raster_face_resolution");
        if snapshot.font_collection.collection_id() != self.font_collection.collection_id()
            || snapshot.font_collection.generation() != self.font_collection.generation()
        {
            return None;
        }

        let (pairs, pair_indices) = self.raster_face_pairs_from_line_indices(line_indices)?;
        self.raster_faces_from_pairs(snapshot, pairs, pair_indices)
    }

    fn raster_face_pairs_from_line_indices(
        &self,
        line_indices: impl IntoIterator<Item = usize>,
    ) -> Option<(
        Vec<(TextFontFaceHandle, Option<TextFontFaceHandle>)>,
        HashMap<(TextFontFaceHandle, Option<TextFontFaceHandle>), usize>,
    )> {
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        let mut scanned_glyph_count = 0usize;
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        let mut raster_glyph_count = 0usize;
        let mut pairs = Vec::new();
        let mut pair_indices = HashMap::new();
        for line_index in line_indices {
            let line = self.artifact.lines.get(line_index)?.as_ref()?;
            #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
            {
                scanned_glyph_count += line.glyphs.len();
            }
            for glyph in line
                .glyphs
                .iter()
                .filter(|glyph| glyph.requires_rasterization)
            {
                #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
                {
                    raster_glyph_count += 1;
                }
                let pair = (glyph.font_face?, glyph.font_instance);
                if let Entry::Vacant(entry) = pair_indices.entry(pair) {
                    let index = pairs.len();
                    pairs.push(pair);
                    entry.insert(index);
                }
            }
        }

        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        {
            crate::profile_counter!(
                "runtime",
                "artifact_raster_face_scanned_glyph_count",
                scanned_glyph_count
            );
            crate::profile_counter!(
                "runtime",
                "artifact_raster_face_candidate_glyph_count",
                raster_glyph_count
            );
            crate::profile_counter!(
                "runtime",
                "artifact_raster_face_unique_pair_count",
                pairs.len()
            );
        }

        Some((pairs, pair_indices))
    }

    fn raster_faces_from_pairs(
        &self,
        snapshot: &UiTextGlyphArtifactFaceSnapshot,
        pairs: Vec<(TextFontFaceHandle, Option<TextFontFaceHandle>)>,
        pair_indices: HashMap<(TextFontFaceHandle, Option<TextFontFaceHandle>), usize>,
    ) -> Option<UiTextGlyphArtifactRasterFaces> {
        let handles = pairs
            .iter()
            .map(|(font_face, font_instance)| (Some(*font_face), *font_instance))
            .collect::<Vec<_>>();
        let resolved = resolve_font_handle_batch_from_snapshot(&snapshot.font_handles, &handles);
        if resolved.len() != pairs.len() {
            return None;
        }

        let mut faces = Vec::with_capacity(pairs.len());
        for ((font_face, font_instance), (resolved_face, resolved_instance)) in
            pairs.into_iter().zip(resolved)
        {
            let face = resolved_face?;
            let variations = match (font_instance, resolved_instance) {
                (None, None) => None,
                (Some(_), Some(instance)) => {
                    let instance = snapshot
                        .font_collection
                        .database()
                        .font_instance(instance)?;
                    (instance.face == face).then(|| instance.variations.clone())
                }
                _ => return None,
            };
            faces.push(UiTextGlyphArtifactRasterFace {
                font_face,
                font_instance,
                font_generation: self.font_generation(),
                bytes: snapshot.font_collection.database().face_bytes(face).ok()?,
                collection_index: snapshot.font_collection.database().face_index(face).ok()?,
                source_identity: snapshot
                    .font_collection
                    .database()
                    .face_source_identity(face)
                    .ok()?,
                variations,
            });
        }

        Some(UiTextGlyphArtifactRasterFaces {
            font_generation: self.font_generation(),
            faces,
            pair_indices,
        })
    }

    /// Resolves a single visual line without cloning a database when it has no raster glyphs.
    pub fn raster_faces(&self) -> Option<UiTextGlyphArtifactRasterFaces> {
        self.raster_faces_from_line_indices([self.line_index])
    }
}

/// A private runtime-font snapshot shared by every artifact line in one resolved layout.
pub struct UiTextGlyphArtifactFaceSnapshot {
    font_collection: FontCollectionSnapshot,
    font_handles: FontHandleResolverSnapshot,
}

impl UiTextGlyphArtifactFaceSnapshot {
    pub const fn font_generation(&self) -> u64 {
        self.font_collection.generation()
    }
}

impl fmt::Debug for UiTextGlyphArtifactFaceSnapshot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("UiTextGlyphArtifactFaceSnapshot")
            .field("font_collection", &self.font_collection)
            .field("font_handles", &self.font_handles)
            .finish_non_exhaustive()
    }
}

/// Immutable runtime-font input for one exact shaped face used by an artifact line.
#[derive(Clone)]
pub struct UiTextGlyphArtifactRasterFace {
    font_face: TextFontFaceHandle,
    font_instance: Option<TextFontFaceHandle>,
    font_generation: u64,
    bytes: Arc<[u8]>,
    collection_index: u32,
    source_identity: [u8; 16],
    variations: Option<VariationCoords>,
}

impl UiTextGlyphArtifactRasterFace {
    pub const fn font_face(&self) -> TextFontFaceHandle {
        self.font_face
    }

    pub const fn font_instance(&self) -> Option<TextFontFaceHandle> {
        self.font_instance
    }

    pub const fn font_generation(&self) -> u64 {
        self.font_generation
    }

    /// A cloned shared allocation, never a copy of the SFNT/TTC source bytes.
    pub fn bytes(&self) -> Arc<[u8]> {
        Arc::clone(&self.bytes)
    }

    /// The collection index to use when the font source contains a TTC collection.
    pub const fn collection_index(&self) -> u32 {
        self.collection_index
    }

    /// Stable source identity for a retained raster cache key.
    pub const fn source_identity(&self) -> [u8; 16] {
        self.source_identity
    }

    /// The effective variation coordinates recorded by shaping, when an instance was selected.
    pub fn variations(&self) -> Option<&VariationCoords> {
        self.variations.as_ref()
    }
}

impl fmt::Debug for UiTextGlyphArtifactRasterFace {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("UiTextGlyphArtifactRasterFace")
            .field("font_face", &self.font_face)
            .field("font_instance", &self.font_instance)
            .field("font_generation", &self.font_generation)
            .field("byte_len", &self.bytes.len())
            .field("collection_index", &self.collection_index)
            .field("source_identity", &self.source_identity)
            .field("variations", &self.variations)
            .finish()
    }
}

/// O(1) lookup table for the runtime faces referenced by one artifact line.
#[derive(Clone, Debug)]
pub struct UiTextGlyphArtifactRasterFaces {
    font_generation: u64,
    faces: Vec<UiTextGlyphArtifactRasterFace>,
    pair_indices: HashMap<(TextFontFaceHandle, Option<TextFontFaceHandle>), usize>,
}

impl UiTextGlyphArtifactRasterFaces {
    pub const fn font_generation(&self) -> u64 {
        self.font_generation
    }

    pub fn faces(&self) -> &[UiTextGlyphArtifactRasterFace] {
        self.faces.as_slice()
    }

    /// Resolves a glyph by its exact shaped face/instance handle pair in O(1).
    pub fn face_for(&self, glyph: &TextGlyph) -> Option<&UiTextGlyphArtifactRasterFace> {
        let index = self
            .pair_indices
            .get(&(glyph.font_face?, glyph.font_instance))?;
        self.faces.get(*index)
    }
}

/// Borrows one exact resolved-layout line from its runtime-owned glyph artifact without cloning glyphs.
///
/// A missing artifact, synthetic line, changed line DTO, or a generation race while acquiring the
/// initial lease returns `None`. Callers must use their established fallback path in those cases.
pub fn resolved_text_glyph_artifact_line(
    layout: &UiResolvedTextLayout,
    line_index: usize,
) -> Option<UiResolvedTextGlyphArtifactLine> {
    let layout_line = layout.lines.get(line_index)?;
    if resolved_text_line_requires_visual_fallback(layout_line) {
        return None;
    }
    let artifact = layout
        .rich_text_artifact
        .as_ref()
        .and_then(resolve_resolved_text_glyph_artifact)?;
    let artifact_line = artifact.lines.get(line_index)?.as_ref()?;
    if artifact.font_generation != artifact.font_lease.generation()
        || artifact_line.layout_line != *layout_line
    {
        return None;
    }
    let font_collection = artifact.font_lease.font_collection().clone();
    let font_handles = artifact.font_lease.font_handles().clone();
    let requires_font_handles = artifact_line
        .glyphs
        .iter()
        .filter(|glyph| glyph.requires_rasterization)
        .any(|glyph| glyph.font_face.is_some() || glyph.font_instance.is_some());
    let handles_match_collection = artifact_line
        .glyphs
        .iter()
        .filter(|glyph| glyph.requires_rasterization)
        .flat_map(|glyph| glyph.font_face.iter().chain(glyph.font_instance.iter()))
        .all(|handle| {
            handle.collection == font_collection.collection_id()
                && handle.generation == font_collection.generation()
        });
    if requires_font_handles
        && (!handles_match_collection || !font_handles.matches_font_collection(&font_collection))
    {
        return None;
    }
    Some(UiResolvedTextGlyphArtifactLine {
        artifact,
        line_index,
        font_collection,
        font_handles,
    })
}

#[cfg(test)]
mod tests;
