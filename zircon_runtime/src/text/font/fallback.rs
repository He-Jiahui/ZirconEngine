use std::collections::HashMap;
use std::sync::Arc;

use crate::text::language::TextLanguageFallbackKey;
use crate::text::model::TextFontResolutionReport;
use crate::text::{CompositeFontDescriptor, FontFaceId, FontQuery, FontScript};
use unicode_normalization::char::canonical_combining_class;

use super::composite_resolve::{CompositeFontIndex, candidate_faces_for_cluster, script_for_char};
use super::database::{FontDatabase, codepoint_requires_font_coverage};
use super::fallback_cache::{
    CompositeFontIdentity, FallbackQueryIdentity, fallback_candidate_cache_key,
    fallback_query_identity, fallback_query_identity_for_asset, fallback_resolution_cache_key,
};

pub(super) const DEFAULT_FALLBACK_MAX_DEPTH: u8 = 10;
pub(super) const DEFAULT_MISSING_GLYPH_CAPACITY: usize = 1_024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FallbackResolution {
    face: FontFaceId,
    missing: bool,
    source: FallbackResolutionSource,
}

impl FallbackResolution {
    pub(in crate::text::font) const fn primary(face: FontFaceId) -> Self {
        Self {
            face,
            missing: false,
            source: FallbackResolutionSource::Primary,
        }
    }

    pub(crate) const fn face(self) -> FontFaceId {
        self.face
    }

    #[cfg(test)]
    pub(crate) const fn is_missing(self) -> bool {
        self.missing
    }

    #[cfg(test)]
    pub(crate) const fn source(self) -> FallbackResolutionSource {
        self.source
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FallbackResolutionSource {
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
    font_asset_owner: Option<&'a str>,
    query_identity: FallbackQueryIdentity,
    language: Option<TextLanguageFallbackKey>,
    max_depth: u8,
    diagnostics: MissingGlyphLog,
    resolution_report: TextFontResolutionReport,
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

    pub(super) fn new_with_language_key(
        db: &'a FontDatabase,
        query: &'a FontQuery,
        composite: Option<&'a CompositeFontDescriptor>,
        language: Option<TextLanguageFallbackKey>,
    ) -> Self {
        Self::with_max_depth_and_language_key(
            db,
            query,
            composite,
            language,
            DEFAULT_FALLBACK_MAX_DEPTH,
        )
    }

    pub(super) fn new_for_font_asset(
        db: &'a FontDatabase,
        query: &'a FontQuery,
        language: Option<TextLanguageFallbackKey>,
        font_asset_owner: &'a str,
    ) -> Self {
        Self::with_max_depth_and_asset_scope(
            db,
            query,
            language,
            DEFAULT_FALLBACK_MAX_DEPTH,
            font_asset_owner,
        )
    }

    pub(super) fn with_max_depth(
        db: &'a FontDatabase,
        query: &'a FontQuery,
        composite: Option<&'a CompositeFontDescriptor>,
        language: Option<&'a str>,
        max_depth: u8,
    ) -> Self {
        Self::with_max_depth_and_language_key(
            db,
            query,
            composite,
            TextLanguageFallbackKey::from_language(language),
            max_depth,
        )
    }

    fn with_max_depth_and_language_key(
        db: &'a FontDatabase,
        query: &'a FontQuery,
        composite: Option<&'a CompositeFontDescriptor>,
        language: Option<TextLanguageFallbackKey>,
        max_depth: u8,
    ) -> Self {
        let composite = db.fallback_composite_index(composite);
        Self::with_compiled_scope(db, query, composite, language, max_depth, None)
    }

    fn with_max_depth_and_asset_scope(
        db: &'a FontDatabase,
        query: &'a FontQuery,
        language: Option<TextLanguageFallbackKey>,
        max_depth: u8,
        font_asset_owner: &'a str,
    ) -> Self {
        let composite = db.fallback_font_asset_composite_index(font_asset_owner);
        Self::with_compiled_scope(
            db,
            query,
            composite,
            language,
            max_depth,
            Some(font_asset_owner),
        )
    }

    fn with_compiled_scope(
        db: &'a FontDatabase,
        query: &'a FontQuery,
        composite: Option<(CompositeFontIdentity, Arc<CompositeFontIndex>)>,
        language: Option<TextLanguageFallbackKey>,
        max_depth: u8,
        font_asset_owner: Option<&'a str>,
    ) -> Self {
        let composite_identity = composite.as_ref().map(|(identity, _)| *identity);
        let query_identity = match font_asset_owner {
            Some(owner) => {
                fallback_query_identity_for_asset(query, composite_identity, language, owner)
            }
            None => fallback_query_identity(query, composite_identity, language),
        };
        Self {
            db,
            query,
            composite: composite.map(|(_, index)| index),
            font_asset_owner,
            query_identity,
            language,
            max_depth,
            diagnostics: MissingGlyphLog::default(),
            resolution_report: TextFontResolutionReport::default(),
        }
    }

    #[cfg(test)]
    pub(super) fn diagnostics(&self) -> &MissingGlyphLog {
        &self.diagnostics
    }

    pub(super) fn take_resolution_report(&mut self) -> TextFontResolutionReport {
        std::mem::take(&mut self.resolution_report)
    }

    pub(super) fn record_primary_text_request(&mut self) {
        self.resolution_report.primary_text_request_count = self
            .resolution_report
            .primary_text_request_count
            .saturating_add(1);
    }

    pub(super) fn record_primary_text_fast_path(&mut self) {
        self.resolution_report.primary_text_fast_path_count = self
            .resolution_report
            .primary_text_fast_path_count
            .saturating_add(1);
        self.resolution_report.primary_selection_count = self
            .resolution_report
            .primary_selection_count
            .saturating_add(1);
    }

    pub(super) fn record_decision_coverage_call(&mut self) {
        self.resolution_report.decision_coverage_call_count = self
            .resolution_report
            .decision_coverage_call_count
            .saturating_add(1);
    }

    pub(super) fn record_primary_coverage_rejection(&mut self) {
        self.resolution_report.primary_coverage_rejection_count = self
            .resolution_report
            .primary_coverage_rejection_count
            .saturating_add(1);
    }

    pub(super) fn resolve(
        &mut self,
        primary: FontFaceId,
        script: FontScript,
        codepoints: &[char],
    ) -> FallbackResolution {
        self.resolution_report.resolution_request_count = self
            .resolution_report
            .resolution_request_count
            .saturating_add(1);
        if codepoints.is_empty() {
            let resolution = FallbackResolution {
                face: primary,
                missing: false,
                source: FallbackResolutionSource::Primary,
            };
            self.record_resolution_source(resolution.source);
            return resolution;
        }

        let candidate_key = fallback_candidate_cache_key(self.query_identity, script, codepoints);
        let resolution_key = fallback_resolution_cache_key(primary, candidate_key, self.max_depth);
        let resolution =
            if let Some(resolution) = self.db.cached_fallback_resolution(resolution_key) {
                self.resolution_report.resolution_cache_hit_count = self
                    .resolution_report
                    .resolution_cache_hit_count
                    .saturating_add(1);
                resolution
            } else {
                self.resolution_report.resolution_cache_miss_count = self
                    .resolution_report
                    .resolution_cache_miss_count
                    .saturating_add(1);
                let resolution = self.resolve_uncached(primary, script, codepoints, candidate_key);
                self.db
                    .cache_fallback_resolution(resolution_key, resolution);
                resolution
            };
        self.record_resolution_source(resolution.source);
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
        let candidates = candidate_faces_for_cluster(
            self.db,
            self.query,
            self.composite.as_deref(),
            self.font_asset_owner,
            script,
            codepoints,
            self.language,
        );
        let candidates = Arc::from(candidates.faces.into_boxed_slice());
        self.db
            .cache_fallback_candidates(key, Arc::clone(&candidates));
        candidates
    }

    fn candidates_for_cluster_with_key_recorded(
        &mut self,
        script: FontScript,
        codepoints: &[char],
        key: super::fallback_cache::FallbackCandidateCacheKey,
    ) -> Arc<[FontFaceId]> {
        if let Some(candidates) = self.db.cached_fallback_candidates(key) {
            self.resolution_report.candidate_cache_hit_count = self
                .resolution_report
                .candidate_cache_hit_count
                .saturating_add(1);
            return candidates;
        }
        self.resolution_report.candidate_cache_miss_count = self
            .resolution_report
            .candidate_cache_miss_count
            .saturating_add(1);
        let candidates = candidate_faces_for_cluster(
            self.db,
            self.query,
            self.composite.as_deref(),
            self.font_asset_owner,
            script,
            codepoints,
            self.language,
        );
        self.resolution_report.decision_coverage_call_count = self
            .resolution_report
            .decision_coverage_call_count
            .saturating_add(candidates.coverage_probe_count as u64);
        let candidates = Arc::from(candidates.faces.into_boxed_slice());
        self.db
            .cache_fallback_candidates(key, Arc::clone(&candidates));
        candidates
    }

    fn resolve_uncached(
        &mut self,
        primary: FontFaceId,
        script: FontScript,
        codepoints: &[char],
        candidate_key: super::fallback_cache::FallbackCandidateCacheKey,
    ) -> FallbackResolution {
        if self.face_covers_all_recorded(primary, codepoints) {
            return FallbackResolution {
                face: primary,
                missing: false,
                source: FallbackResolutionSource::Primary,
            };
        }
        self.record_primary_coverage_rejection();
        if self.max_depth == 0 {
            return FallbackResolution {
                face: primary,
                missing: true,
                source: FallbackResolutionSource::DepthLimitExceeded,
            };
        }

        let candidates =
            self.candidates_for_cluster_with_key_recorded(script, codepoints, candidate_key);
        for candidate in candidates.iter().copied() {
            self.resolution_report.complete_candidate_visit_count = self
                .resolution_report
                .complete_candidate_visit_count
                .saturating_add(1);
            if candidate == primary {
                continue;
            }
            if self.face_covers_all_recorded(candidate, codepoints) {
                return FallbackResolution {
                    face: candidate,
                    missing: false,
                    source: FallbackResolutionSource::Fallback,
                };
            }
            self.resolution_report.complete_candidate_rejection_count = self
                .resolution_report
                .complete_candidate_rejection_count
                .saturating_add(1);
        }
        if let Some(face) = self.best_partial_coverage_face(primary, &candidates, codepoints) {
            return FallbackResolution {
                face,
                missing: true,
                source: FallbackResolutionSource::PartialCoverage,
            };
        }
        FallbackResolution {
            face: self.db.runtime_last_resort_face().unwrap_or(primary),
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
            if codepoint_requires_font_coverage(*codepoint) {
                self.record_decision_coverage_call();
            }
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
        &mut self,
        primary: FontFaceId,
        candidates: &[FontFaceId],
        codepoints: &[char],
    ) -> Option<FontFaceId> {
        let base = cluster_base_codepoint(codepoints)?;
        let mut best = None;
        for face in std::iter::once(primary)
            .chain(candidates.iter().copied().filter(|face| *face != primary))
        {
            self.resolution_report.partial_candidate_visit_count = self
                .resolution_report
                .partial_candidate_visit_count
                .saturating_add(1);
            if codepoint_requires_font_coverage(base) {
                self.record_decision_coverage_call();
            }
            if !self.db.face_covers_codepoint(face, base) {
                continue;
            }
            let covered = self.face_coverage_count_recorded(face, codepoints);
            if best.is_none_or(|(_, best_covered)| covered > best_covered) {
                best = Some((face, covered));
            }
        }
        best.map(|(face, _)| face)
    }

    fn record_resolution_source(&mut self, source: FallbackResolutionSource) {
        let count = match source {
            FallbackResolutionSource::Primary => {
                &mut self.resolution_report.primary_selection_count
            }
            FallbackResolutionSource::Fallback => {
                &mut self.resolution_report.fallback_selection_count
            }
            FallbackResolutionSource::PartialCoverage => {
                &mut self.resolution_report.partial_coverage_selection_count
            }
            FallbackResolutionSource::LastResort => {
                &mut self.resolution_report.last_resort_selection_count
            }
            FallbackResolutionSource::DepthLimitExceeded => {
                &mut self.resolution_report.depth_limit_selection_count
            }
        };
        *count = count.saturating_add(1);
    }

    fn face_covers_all_recorded(&mut self, face: FontFaceId, codepoints: &[char]) -> bool {
        for codepoint in codepoints {
            if codepoint_requires_font_coverage(*codepoint) {
                self.record_decision_coverage_call();
            }
            if !self.db.face_covers_codepoint(face, *codepoint) {
                return false;
            }
        }
        true
    }

    fn face_coverage_count_recorded(&mut self, face: FontFaceId, codepoints: &[char]) -> usize {
        codepoints
            .iter()
            .filter(|codepoint| codepoint_requires_font_coverage(**codepoint))
            .filter(|codepoint| {
                self.record_decision_coverage_call();
                self.db.face_covers_codepoint(face, **codepoint)
            })
            .count()
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
