use std::sync::Arc;

use crate::text::{CompositeFontDescriptor, FontFaceId, FontQuery, FontScript};

use super::super::composite_resolve::CompositeFontIndex;
use super::super::fallback::{FallbackResolution, FallbackResolver, MissingGlyphDiagnosticsReport};
use super::super::fallback_cache::{
    CompositeFontIdentity, FallbackCacheReport, FallbackCandidateCacheKey,
    FallbackResolutionCacheKey,
};
use super::FontDatabase;

pub(crate) struct FontShapingFaceResolver<'a> {
    database: &'a FontDatabase,
    primary: FontFaceId,
    fallback: FallbackResolver<'a>,
}

impl FontShapingFaceResolver<'_> {
    pub(crate) const fn primary_face(&self) -> FontFaceId {
        self.primary
    }

    pub(crate) fn primary_covers_text(&self, text: &str) -> bool {
        text.chars()
            .all(|codepoint| self.database.face_covers_codepoint(self.primary, codepoint))
    }

    pub(crate) fn resolve(&mut self, script: FontScript, codepoints: &[char]) -> FontFaceId {
        self.fallback.resolve(self.primary, script, codepoints).face
    }
}

impl Drop for FontShapingFaceResolver<'_> {
    fn drop(&mut self) {
        self.database
            .missing_glyph_log()
            .append(self.fallback.take_diagnostics());
    }
}

impl FontDatabase {
    pub(crate) fn fallback_candidates_for_codepoint(
        &self,
        codepoint: char,
        query: &FontQuery,
        composite: Option<&CompositeFontDescriptor>,
        language: Option<&str>,
    ) -> Vec<FontFaceId> {
        FallbackResolver::new(self, query, composite, language).candidates_for_codepoint(codepoint)
    }

    pub(crate) fn resolve_fallback_face_for_codepoint(
        &self,
        primary: FontFaceId,
        codepoint: char,
        query: &FontQuery,
        composite: Option<&CompositeFontDescriptor>,
        language: Option<&str>,
    ) -> FontFaceId {
        let mut resolver = FallbackResolver::new(self, query, composite, language);
        let resolution = resolver.resolve_codepoint(primary, codepoint);
        self.missing_glyph_log().append(resolver.take_diagnostics());
        resolution.face
    }

    pub(crate) fn resolve_shaping_face_for_cluster(
        &self,
        script: FontScript,
        codepoints: &[char],
        query: &FontQuery,
        language: Option<&str>,
    ) -> Option<FontFaceId> {
        let mut resolver = self.begin_shaping_face_resolution(query, language)?;
        Some(resolver.resolve(script, codepoints))
    }

    pub(crate) fn begin_shaping_face_resolution<'a>(
        &'a self,
        query: &'a FontQuery,
        language: Option<&'a str>,
    ) -> Option<FontShapingFaceResolver<'a>> {
        let primary = self.match_face(query)?.face;
        Some(FontShapingFaceResolver {
            database: self,
            primary,
            fallback: FallbackResolver::new(self, query, None, language),
        })
    }

    pub(crate) fn take_missing_glyph_diagnostics(&self) -> MissingGlyphDiagnosticsReport {
        self.missing_glyph_log().take_report()
    }

    pub(crate) fn fallback_cache_report(&self) -> FallbackCacheReport {
        self.fallback_caches.report()
    }

    pub(in crate::text::font) fn fallback_composite_index(
        &self,
        composite: Option<&CompositeFontDescriptor>,
    ) -> Option<(CompositeFontIdentity, Arc<CompositeFontIndex>)> {
        composite
            .map(|descriptor| self.fallback_caches.composite_index(descriptor))
            .or_else(|| {
                self.project_composite_index
                    .as_ref()
                    .map(|(identity, index)| (*identity, Arc::clone(index)))
            })
    }

    pub(in crate::text::font) fn cached_fallback_candidates(
        &self,
        key: FallbackCandidateCacheKey,
    ) -> Option<Arc<[FontFaceId]>> {
        self.fallback_caches.candidates(key)
    }

    pub(in crate::text::font) fn record_fallback_family_visits(&self, count: usize) {
        self.fallback_caches.record_family_visits(count);
    }

    pub(in crate::text::font) fn record_fallback_coverage_probe(&self) {
        self.fallback_caches.record_coverage_probe();
    }

    pub(in crate::text::font) fn cache_fallback_candidates(
        &self,
        key: FallbackCandidateCacheKey,
        candidates: Arc<[FontFaceId]>,
    ) {
        self.fallback_caches.insert_candidates(key, candidates);
    }

    pub(in crate::text::font) fn cached_fallback_resolution(
        &self,
        key: FallbackResolutionCacheKey,
    ) -> Option<FallbackResolution> {
        self.fallback_caches.resolution(key)
    }

    pub(in crate::text::font) fn cache_fallback_resolution(
        &self,
        key: FallbackResolutionCacheKey,
        resolution: FallbackResolution,
    ) {
        self.fallback_caches.insert_resolution(key, resolution);
    }
}
