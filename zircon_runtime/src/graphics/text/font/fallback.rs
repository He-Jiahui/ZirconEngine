use crate::core::framework::render::{CompositeFontDescriptor, FontFaceId, FontQuery, FontScript};
use unicode_normalization::char::canonical_combining_class;

use super::composite_resolve::{candidate_faces_for_cluster, script_for_char};
use super::database::FontDatabase;

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
    capacity: usize,
    overflowed: bool,
    dropped_count: usize,
}

impl Default for MissingGlyphLog {
    fn default() -> Self {
        Self::with_capacity(DEFAULT_MISSING_GLYPH_CAPACITY)
    }
}

impl MissingGlyphLog {
    pub(super) fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::new(),
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
        if let Some(existing) = self.entries.iter_mut().find(|existing| {
            existing.face == diagnostic.face && existing.codepoint == diagnostic.codepoint
        }) {
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
        self.entries.push(diagnostic);
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
    composite: Option<&'a CompositeFontDescriptor>,
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
        Self {
            db,
            query,
            composite,
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
        if codepoints.is_empty() || self.db.face_covers_all(primary, codepoints) {
            return FallbackResolution {
                face: primary,
                missing: false,
                source: FallbackResolutionSource::Primary,
            };
        }

        if self.max_depth == 0 {
            self.record_missing(
                primary,
                script,
                codepoints,
                MissingGlyphReason::DepthLimitExceeded,
            );
            return FallbackResolution {
                face: primary,
                missing: true,
                source: FallbackResolutionSource::DepthLimitExceeded,
            };
        }

        let candidates = self.candidates_for_cluster(script, codepoints);
        for candidate in &candidates {
            let candidate = *candidate;
            if candidate != primary && self.db.face_covers_all(candidate, codepoints) {
                return FallbackResolution {
                    face: candidate,
                    missing: false,
                    source: FallbackResolutionSource::Fallback,
                };
            }
        }

        if let Some(face) = self.best_partial_coverage_face(primary, &candidates, codepoints) {
            self.record_missing(face, script, codepoints, MissingGlyphReason::MissingGlyph);
            return FallbackResolution {
                face,
                missing: true,
                source: FallbackResolutionSource::PartialCoverage,
            };
        }

        self.record_missing(
            primary,
            script,
            codepoints,
            MissingGlyphReason::MissingGlyph,
        );
        FallbackResolution {
            face: primary,
            missing: true,
            source: FallbackResolutionSource::LastResort,
        }
    }

    pub(super) fn candidates_for_codepoint(&self, codepoint: char) -> Vec<FontFaceId> {
        self.candidates_for_cluster(script_for_char(codepoint), &[codepoint])
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

    fn candidates_for_cluster(&self, script: FontScript, codepoints: &[char]) -> Vec<FontFaceId> {
        candidate_faces_for_cluster(
            self.db,
            self.query,
            self.composite,
            script,
            codepoints,
            self.language,
        )
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
