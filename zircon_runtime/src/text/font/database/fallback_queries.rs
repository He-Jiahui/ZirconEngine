use std::borrow::Cow;
use std::sync::Arc;

use crate::text::language::TextLanguageFallbackKey;
use crate::text::model::TextFontResolutionReport;
use crate::text::{CompositeFontDescriptor, FontFaceId, FontQuery, FontScript};

use super::super::composite_resolve::CompositeFontIndex;
use super::super::composite_resolve::script_for_char;
use super::super::fallback::{FallbackResolution, FallbackResolver, MissingGlyphDiagnosticsReport};
#[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
use super::super::fallback_cache::FallbackCacheRequestProfile;
use super::super::fallback_cache::{
    CompositeFontIdentity, FallbackCacheReport, FallbackCandidateCacheKey,
    FallbackResolutionCacheKey, LineMetricEnvelopeCacheKey,
};
use super::super::line_metrics::FontChainLineMetricEnvelope;
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

    pub(crate) const fn primary_resolution(&self) -> FallbackResolution {
        FallbackResolution::primary(self.primary)
    }

    pub(crate) fn primary_covers_text(&mut self, text: &str) -> bool {
        self.fallback.record_primary_text_request();
        for codepoint in text.chars() {
            if super::codepoint_requires_font_coverage(codepoint) {
                self.fallback.record_decision_coverage_call();
            }
            if !self.database.face_covers_codepoint(self.primary, codepoint) {
                self.fallback.record_primary_coverage_rejection();
                return false;
            }
        }
        self.fallback.record_primary_text_fast_path();
        true
    }

    pub(crate) fn resolve(
        &mut self,
        script: FontScript,
        codepoints: &[char],
    ) -> FallbackResolution {
        self.fallback.resolve(self.primary, script, codepoints)
    }

    pub(crate) fn take_resolution_report(&mut self) -> TextFontResolutionReport {
        self.fallback.take_resolution_report()
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
    pub(crate) fn constrain_font_query_to_request_owner<'a>(
        &self,
        query: &'a FontQuery,
        font_asset_owner: Option<&str>,
    ) -> Cow<'a, FontQuery> {
        let owner_is_unavailable = font_asset_owner
            .filter(|owner| !owner.is_empty())
            .is_some_and(|owner| !self.asset_owners.contains_key(owner));
        if !owner_is_unavailable || query.families.is_empty() {
            return Cow::Borrowed(query);
        }
        let mut constrained = query.clone();
        constrained.families.clear();
        Cow::Owned(constrained)
    }

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
        resolution.face()
    }

    pub(crate) fn resolve_shaping_face_for_cluster(
        &self,
        script: FontScript,
        codepoints: &[char],
        query: &FontQuery,
        language: Option<&str>,
    ) -> Option<FontFaceId> {
        let mut resolver = self.begin_shaping_face_resolution(query, language)?;
        Some(resolver.resolve(script, codepoints).face())
    }

    pub(crate) fn resolve_shaping_face_for_request_codepoint(
        &self,
        codepoint: char,
        query: &FontQuery,
        font_asset_owner: Option<&str>,
        language: Option<&str>,
    ) -> Option<FontFaceId> {
        let query = self.constrain_font_query_to_request_owner(query, font_asset_owner);
        let mut resolver = self.begin_shaping_face_resolution_for_request(
            query.as_ref(),
            font_asset_owner,
            TextLanguageFallbackKey::from_language(language),
        )?;
        Some(
            resolver
                .resolve(script_for_char(codepoint), &[codepoint])
                .face(),
        )
    }

    pub(crate) fn begin_shaping_face_resolution<'a>(
        &'a self,
        query: &'a FontQuery,
        language: Option<&'a str>,
    ) -> Option<FontShapingFaceResolver<'a>> {
        self.begin_shaping_face_resolution_with_language_key(
            query,
            TextLanguageFallbackKey::from_language(language),
        )
    }

    pub(crate) fn begin_shaping_face_resolution_for_request<'a>(
        &'a self,
        query: &'a FontQuery,
        font_asset_owner: Option<&'a str>,
        language: Option<TextLanguageFallbackKey>,
    ) -> Option<FontShapingFaceResolver<'a>> {
        if let Some(owner) = font_asset_owner
            .filter(|owner| !owner.is_empty())
            .filter(|owner| self.asset_owners.contains_key(*owner))
        {
            return self.begin_font_asset_shaping_face_resolution(query, owner, language);
        }
        self.begin_shaping_face_resolution_with_language_key(query, language)
    }

    fn begin_font_asset_shaping_face_resolution<'a>(
        &'a self,
        query: &'a FontQuery,
        owner: &'a str,
        language: Option<TextLanguageFallbackKey>,
    ) -> Option<FontShapingFaceResolver<'a>> {
        self.asset_owners.get(owner)?;
        let primary = self.match_font_asset_face(owner, query)?.face;
        Some(FontShapingFaceResolver {
            database: self,
            primary,
            fallback: FallbackResolver::new_for_font_asset(self, query, language, owner),
        })
    }

    fn begin_shaping_face_resolution_with_language_key<'a>(
        &'a self,
        query: &'a FontQuery,
        language: Option<TextLanguageFallbackKey>,
    ) -> Option<FontShapingFaceResolver<'a>> {
        let primary = self.match_face(query)?.face;
        Some(FontShapingFaceResolver {
            database: self,
            primary,
            fallback: FallbackResolver::new_with_language_key(self, query, None, language),
        })
    }

    pub(crate) fn take_missing_glyph_diagnostics(&self) -> MissingGlyphDiagnosticsReport {
        self.missing_glyph_log().take_report()
    }

    pub(crate) fn fallback_cache_report(&self) -> FallbackCacheReport {
        self.fallback_caches.report()
    }

    #[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
    pub(crate) fn begin_fallback_cache_profile_request(&self) {
        self.fallback_caches.begin_profile_request();
    }

    #[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
    pub(crate) fn take_fallback_cache_profile_request(
        &self,
    ) -> Option<FallbackCacheRequestProfile> {
        self.fallback_caches.take_profile_request()
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
            .or_else(|| {
                self.runtime_default_composite_index
                    .as_ref()
                    .map(|(identity, index)| (*identity, Arc::clone(index)))
            })
    }

    pub(in crate::text::font) fn fallback_font_asset_composite_index(
        &self,
        owner: &str,
    ) -> Option<(CompositeFontIdentity, Arc<CompositeFontIndex>)> {
        self.asset_composite_indexes
            .get(owner)
            .map(|(identity, index)| (*identity, Arc::clone(index)))
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

    pub(in crate::text::font) fn cached_line_metric_envelope(
        &self,
        key: LineMetricEnvelopeCacheKey,
    ) -> Option<Option<FontChainLineMetricEnvelope>> {
        self.fallback_caches.line_metric_envelope(key)
    }

    pub(in crate::text::font) fn cache_line_metric_envelope(
        &self,
        key: LineMetricEnvelopeCacheKey,
        envelope: Option<FontChainLineMetricEnvelope>,
    ) {
        self.fallback_caches
            .insert_line_metric_envelope(key, envelope);
    }
}
