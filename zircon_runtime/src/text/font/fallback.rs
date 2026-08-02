use std::collections::HashMap;
use std::sync::Arc;

use crate::text::{CompositeFontDescriptor, FontFaceId, FontQuery, FontScript};
use unicode_normalization::char::canonical_combining_class;

use super::composite_resolve::{CompositeFontIndex, candidate_faces_for_cluster, script_for_char};
use super::database::FontDatabase;
use super::fallback_cache::{
    FallbackQueryIdentity, fallback_candidate_cache_key, fallback_query_identity,
    fallback_resolution_cache_key,
};

pub(super) const DEFAULT_FALLBACK_MAX_DEPTH: u8 = 10;
pub(super) const DEFAULT_MISSING_GLYPH_CAPACITY: usize = 1_024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct FallbackResolution {
    pub face: FontFaceId,
    pub missing: bool,
    pub source: FallbackResolutionSource,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FallbackResolutionSource {
    Primary,
    Fallback,
    PartialCoverage,
    LastResort,
    DepthLimitExceeded,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct MissingGlyphLog {
    entries: Vec<MissingGlyphDiagnostic>,
    // The map owns only bounded lookup state; `entries` retains report insertion order.
    entry_by_key: HashMap<MissingGlyphKey, usize>,
    capacity: usize,
    overflowed: bool,
    dropped_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct MissingGlyphKey {
    face: FontFaceId,
    codepoint: u32,
}

impl From<&MissingGlyphDiagnostic> for MissingGlyphKey {
    fn from(diagnostic: &MissingGlyphDiagnostic) -> Self {
        Self {
            face: diagnostic.face,
            codepoint: diagnostic.codepoint,
        }
    }
}

impl Default for MissingGlyphLog {
    fn default() -> Self {
        Self::with_capacity(DEFAULT_MISSING_GLYPH_CAPACITY)
    }
}

impl MissingGlyphLog {
    pub(super) fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
            entry_by_key: HashMap::with_capacity(capacity),
            capacity,
            overflowed: false,
            dropped_count: 0,
        }
    }

    #[cfg(test)]
    pub(super) fn entries(&self) -> &[MissingGlyphDiagnostic] {
        &self.entries
    }

    #[cfg(test)]
    pub(super) fn overflowed(&self) -> bool {
        self.overflowed
    }

    #[cfg(test)]
    pub(super) fn dropped_count(&self) -> usize {
        self.dropped_count
    }

    pub(super) fn push(&mut self, diagnostic: MissingGlyphDiagnostic) {
        let key = MissingGlyphKey::from(&diagnostic);
        if let Some(&index) = self.entry_by_key.get(&key) {
            let existing = &mut self.entries[index];
            existing.occurrence_count = existing
                .occurrence_count
                .saturating_add(diagnostic.occurrence_count);
            return;
        }
        if self.entries.len() >= self.capacity {
            self.overflowed = true;
            self.dropped_count = self.dropped_count.saturating_add(1);
            return;
        }
        let index = self.entries.len();
        self.entries.push(diagnostic);
        self.entry_by_key.insert(key, index);
    }

    pub(super) fn append(&mut self, other: Self) {
        for entry in other.entries {
            self.push(entry);
        }
        self.overflowed |= other.overflowed;
        self.dropped_count = self.dropped_count.saturating_add(other.dropped_count);
    }

    pub(super) fn take_report(&mut self) -> MissingGlyphDiagnosticsReport {
        let replacement = Self::with_capacity(self.capacity);
        let current = std::mem::replace(self, replacement);
        MissingGlyphDiagnosticsReport {
            entries: current.entries,
            overflowed: current.overflowed,
            dropped_count: current.dropped_count,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MissingGlyphDiagnostic {
    pub face: FontFaceId,
    pub script: FontScript,
    pub codepoint: u32,
    pub reason: MissingGlyphReason,
    pub occurrence_count: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MissingGlyphReason {
    MissingGlyph,
    DepthLimitExceeded,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct MissingGlyphDiagnosticsReport {
    pub(crate) entries: Vec<MissingGlyphDiagnostic>,
    pub(crate) overflowed: bool,
    pub(crate) dropped_count: usize,
}

pub(super) struct FallbackResolver<'a> {
    db: &'a FontDatabase,
    query: &'a FontQuery,
    composite: Option<Arc<CompositeFontIndex>>,
    query_identity: FallbackQueryIdentity,
    language: Option<&'a str>,
    max_depth: u8,
    diagnostics: MissingGlyphLog,
}

impl<'a> FallbackResolver<'a> {
    pub(super) fn new(
        db: &'a FontDatabase,
        query: &'a FontQuery,
        composite: Option<&'a CompositeFontDescriptor>,
        language: Option<&'a str>,
    ) -> Self {
        Self::with_max_depth(db, query, composite, language, DEFAULT_FALLBACK_MAX_DEPTH)
    }

    pub(super) fn with_max_depth(
        db: &'a FontDatabase,
        query: &'a FontQuery,
        composite: Option<&'a CompositeFontDescriptor>,
        language: Option<&'a str>,
        max_depth: u8,
    ) -> Self {
        let composite = db.fallback_composite_index(composite);
        let query_identity = fallback_query_identity(
            query,
            composite.as_ref().map(|(identity, _)| *identity),
            language,
        );
        Self {
            db,
            query,
            composite: composite.map(|(_, index)| index),
            query_identity,
            language,
            max_depth,
            diagnostics: MissingGlyphLog::default(),
        }
    }

    #[cfg(test)]
    pub(super) fn diagnostics(&self) -> &MissingGlyphLog {
        &self.diagnostics
    }

    pub(super) fn resolve(
        &mut self,
        primary: FontFaceId,
        script: FontScript,
        codepoints: &[char],
    ) -> FallbackResolution {
        if codepoints.is_empty() {
            return FallbackResolution {
                face: primary,
                missing: false,
                source: FallbackResolutionSource::Primary,
            };
        }

        let candidate_key = fallback_candidate_cache_key(self.query_identity, script, codepoints);
        let resolution_key = fallback_resolution_cache_key(primary, candidate_key, self.max_depth);
        let resolution = self
            .db
            .cached_fallback_resolution(resolution_key)
            .unwrap_or_else(|| {
                let resolution = self.resolve_uncached(primary, script, codepoints, candidate_key);
                self.db
                    .cache_fallback_resolution(resolution_key, resolution.clone());
                resolution
            });
        if resolution.missing {
            let reason = if resolution.source == FallbackResolutionSource::DepthLimitExceeded {
                MissingGlyphReason::DepthLimitExceeded
            } else {
                MissingGlyphReason::MissingGlyph
            };
            self.record_missing(resolution.face, script, codepoints, reason);
        }
        resolution
    }

    pub(super) fn candidates_for_codepoint(&self, codepoint: char) -> Vec<FontFaceId> {
        self.candidates_for_cluster(script_for_char(codepoint), &[codepoint])
            .to_vec()
    }

    pub(super) fn resolve_codepoint(
        &mut self,
        primary: FontFaceId,
        codepoint: char,
    ) -> FallbackResolution {
        self.resolve(primary, script_for_char(codepoint), &[codepoint])
    }

    pub(super) fn take_diagnostics(&mut self) -> MissingGlyphLog {
        std::mem::take(&mut self.diagnostics)
    }

    fn candidates_for_cluster(&self, script: FontScript, codepoints: &[char]) -> Arc<[FontFaceId]> {
        let key = fallback_candidate_cache_key(self.query_identity, script, codepoints);
        self.candidates_for_cluster_with_key(script, codepoints, key)
    }

    fn candidates_for_cluster_with_key(
        &self,
        script: FontScript,
        codepoints: &[char],
        key: super::fallback_cache::FallbackCandidateCacheKey,
    ) -> Arc<[FontFaceId]> {
        if let Some(candidates) = self.db.cached_fallback_candidates(key) {
            return candidates;
        }
        let candidates = Arc::from(
            candidate_faces_for_cluster(
                self.db,
                self.query,
                self.composite.as_deref(),
                script,
                codepoints,
                self.language,
            )
            .into_boxed_slice(),
        );
        self.db
            .cache_fallback_candidates(key, Arc::clone(&candidates));
        candidates
    }

    fn resolve_uncached(
        &self,
        primary: FontFaceId,
        script: FontScript,
        codepoints: &[char],
        candidate_key: super::fallback_cache::FallbackCandidateCacheKey,
    ) -> FallbackResolution {
        if self.db.face_covers_all(primary, codepoints) {
            return FallbackResolution {
                face: primary,
                missing: false,
                source: FallbackResolutionSource::Primary,
            };
        }
        if self.max_depth == 0 {
            return FallbackResolution {
                face: primary,
                missing: true,
                source: FallbackResolutionSource::DepthLimitExceeded,
            };
        }

        let candidates = self.candidates_for_cluster_with_key(script, codepoints, candidate_key);
        for candidate in candidates.iter().copied() {
            if candidate != primary && self.db.face_covers_all(candidate, codepoints) {
                return FallbackResolution {
                    face: candidate,
                    missing: false,
                    source: FallbackResolutionSource::Fallback,
                };
            }
        }
        if let Some(face) = self.best_partial_coverage_face(primary, &candidates, codepoints) {
            return FallbackResolution {
                face,
                missing: true,
                source: FallbackResolutionSource::PartialCoverage,
            };
        }
        FallbackResolution {
            face: primary,
            missing: true,
            source: FallbackResolutionSource::LastResort,
        }
    }

    fn record_missing(
        &mut self,
        face: FontFaceId,
        script: FontScript,
        codepoints: &[char],
        reason: MissingGlyphReason,
    ) {
        for codepoint in codepoints {
            if self.db.face_covers_codepoint(face, *codepoint) {
                continue;
            }
            self.diagnostics.push(MissingGlyphDiagnostic {
                face,
                script,
                codepoint: *codepoint as u32,
                reason,
                occurrence_count: 1,
            });
        }
    }

    fn best_partial_coverage_face(
        &self,
        primary: FontFaceId,
        candidates: &[FontFaceId],
        codepoints: &[char],
    ) -> Option<FontFaceId> {
        let base = cluster_base_codepoint(codepoints)?;
        let mut best = None;
        for face in std::iter::once(primary)
            .chain(candidates.iter().copied().filter(|face| *face != primary))
            .filter(|face| self.db.face_covers_codepoint(*face, base))
        {
            let covered = self.db.face_coverage_count(face, codepoints);
            if best.is_none_or(|(_, best_covered)| covered > best_covered) {
                best = Some((face, covered));
            }
        }
        best.map(|(face, _)| face)
    }
}

fn cluster_base_codepoint(codepoints: &[char]) -> Option<char> {
    codepoints.iter().copied().find(|codepoint| {
        canonical_combining_class(*codepoint) == 0
            && !matches!(*codepoint, '\u{200D}' | '\u{FE0E}' | '\u{FE0F}')
    })
}

#[cfg(test)]
mod tests;
