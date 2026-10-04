use std::collections::HashSet;

use super::fallback_cache::{
    fallback_query_identity, fallback_query_identity_for_asset, line_metric_envelope_cache_key,
};
use super::matching::{
    dedupe_scoped_families, FontFamilyCandidateScope, ScopedFontFamilyCandidate,
};
use super::{font_query_for_text_style, FontDatabase};
use crate::asset::FontAssetFaceMetrics;
use crate::text::language::TextLanguageFallbackKey;
use crate::text::{FontFaceId, HorizontalLineRawMetrics, TextStyle};

/// A generation-local upper bound for the faces an arbitrary line can resolve.
///
/// This is admission evidence for a fixed-height shortcut only. It is not a
/// shaped-line baseline and does not replace per-line selected-face metrics.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct FontChainLineMetricEnvelope {
    max_ascent: f32,
    max_descent: f32,
    primary_line_gap: f32,
}

impl FontChainLineMetricEnvelope {
    pub(crate) fn minimum_line_height(self) -> f32 {
        self.max_ascent + self.max_descent + self.primary_line_gap
    }

    pub(crate) fn certifies_uniform_line_height(self, requested_line_height: f32) -> bool {
        requested_line_height.is_finite() && requested_line_height >= self.minimum_line_height()
    }
}

/// Bounds every face that the active composite/query/fallback chain may select.
///
/// The candidate set deliberately avoids source codepoints and the resolver's
/// codepoint-keyed caches. A constant-height result is safe only when the caller's
/// requested line height covers this complete envelope.
pub(crate) fn font_chain_line_metric_envelope(
    database: &FontDatabase,
    style: &TextStyle,
) -> Option<FontChainLineMetricEnvelope> {
    let font_size = style.font_size.max(1.0);
    if !font_size.is_finite() {
        return None;
    }
    let query = font_query_for_text_style(style);
    let query = database.constrain_font_query_to_request_owner(&query, style.font.as_deref());
    let query = query.as_ref();
    let font_asset_owner = style
        .font
        .as_deref()
        .filter(|owner| database.has_font_asset_owner(owner));
    let composite = match font_asset_owner {
        Some(owner) => database.fallback_font_asset_composite_index(owner),
        None => database.fallback_composite_index(None),
    };
    let language = TextLanguageFallbackKey::from_language(style.language.as_deref());
    let composite_identity = composite.as_ref().map(|(identity, _)| *identity);
    let query_identity = match font_asset_owner {
        Some(owner) => {
            fallback_query_identity_for_asset(query, composite_identity, language, owner)
        }
        None => fallback_query_identity(query, composite_identity, language),
    };
    let cache_key = line_metric_envelope_cache_key(query_identity, font_size);
    // 认证结果按 asset owner、query/composite/language 与字号隔离，避免复用另一条候选字体链的高度上界。
    if let Some(cached) = database.cached_line_metric_envelope(cache_key) {
        return cached;
    }

    let primary = match font_asset_owner {
        Some(owner) => database.match_font_asset_face(owner, query),
        None => database.match_face(query),
    };
    let envelope = primary.and_then(|primary| {
        let mut extents = SelectedFaceLineExtents::default();
        let mut faces = HashSet::new();
        extents.include_primary_face(database, primary.face, font_size);
        if faces.insert(primary.face) {
            let _ = extents.include_face(database, primary.face, font_size);
        }
        if let Some(last_resort) = database.runtime_last_resort_face() {
            if faces.insert(last_resort) {
                let _ = extents.include_face(database, last_resort, font_size);
            }
        }
        for candidate in font_chain_metric_families(
            composite.as_ref().map(|(_, composite)| composite.as_ref()),
            query,
            database,
            font_asset_owner,
            language,
        ) {
            let family_faces = match font_asset_owner {
                Some(owner) => database.font_asset_family_candidates_for_line_metrics(
                    owner,
                    &candidate.family,
                    query,
                    candidate.scope,
                ),
                None => database
                    .family_candidates_for_line_metrics(&candidate.family, query)
                    .to_vec(),
            };
            for face in family_faces {
                if faces.insert(face) {
                    let _ = extents.include_face(database, face, font_size);
                }
            }
        }
        extents.font_chain_metric_envelope()
    });
    database.cache_line_metric_envelope(cache_key, envelope);
    envelope
}

fn font_chain_metric_families(
    composite: Option<&super::composite_resolve::CompositeFontIndex>,
    query: &crate::text::FontQuery,
    database: &FontDatabase,
    font_asset_owner: Option<&str>,
    language: Option<TextLanguageFallbackKey>,
) -> Vec<ScopedFontFamilyCandidate> {
    let external = FontFamilyCandidateScope::OwnerThenGlobal;
    let query_scope = if font_asset_owner.is_some() {
        FontFamilyCandidateScope::OwnerLocalOnly
    } else {
        external
    };
    let mut families = composite
        .map_or_else(Vec::new, |composite| {
            composite.line_metric_envelope_families(language)
        })
        .into_iter()
        .map(|family| (family, external))
        .collect::<Vec<_>>();
    families.extend(
        query
            .families
            .iter()
            .cloned()
            .map(|family| (family, query_scope)),
    );
    if let Some(owner) = font_asset_owner {
        if let Some(asset_fallbacks) = database.font_asset_fallback_families(owner) {
            families.extend(
                asset_fallbacks
                    .iter()
                    .cloned()
                    .map(|family| (family, external)),
            );
        }
        families.extend(
            database
                .font_asset_base_fallback_families()
                .iter()
                .cloned()
                .map(|family| (family, external)),
        );
    } else {
        families.extend(
            database
                .fallback_families()
                .iter()
                .cloned()
                .map(|family| (family, external)),
        );
    }
    dedupe_scoped_families(families)
}

/// Certifies the narrow primary-only case for a fixed-height hard-line shortcut.
///
/// This deliberately does not approximate a fallback chain. A caller must use complete
/// measurement when even one hard-line content grapheme needs another face.
pub(crate) fn primary_face_covers_all_hard_line_content(
    database: &FontDatabase,
    style: &TextStyle,
    text: &str,
) -> bool {
    let query = font_query_for_text_style(style);
    let query = database.constrain_font_query_to_request_owner(&query, style.font.as_deref());
    let Some(mut resolver) = database.begin_shaping_face_resolution_for_request(
        query.as_ref(),
        style.font.as_deref(),
        TextLanguageFallbackKey::from_language(style.language.as_deref()),
    ) else {
        return false;
    };
    crate::text::hard_lines(text).into_iter().all(|line| {
        text.get(line.content)
            .is_some_and(|content| resolver.primary_covers_text(content))
    })
}

/// Aggregates the raw content extents of faces selected for one text fragment.
///
/// This is input to line-box policy and glyph-origin projection. It does not establish the
/// public composite baseline for a UI line: that decision also needs the participating runs and
/// matching glyph-origin adjustments.
#[derive(Default)]
pub(crate) struct SelectedFaceLineExtents {
    ascent: f32,
    descent: f32,
    selected_face_line_gap: f32,
    primary_face_line_gap: Option<f32>,
    has_face_metrics: bool,
    has_primary_face_metrics: bool,
}

/// Raw face-content envelope positioned within its requested line height.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SelectedFaceLineEnvelope {
    pub(crate) baseline_from_top: f32,
    pub(crate) line_height: f32,
}

impl SelectedFaceLineExtents {
    pub(crate) fn include_face(
        &mut self,
        database: &FontDatabase,
        face: FontFaceId,
        font_size: f32,
    ) -> Option<HorizontalLineRawMetrics> {
        let Some(metrics) = database.face_metrics(face).ok().flatten() else {
            return None;
        };
        if metrics.units_per_em == 0 {
            return None;
        }
        let Some((ascent, descent, line_gap)) = scaled_layout_extents(metrics, font_size) else {
            return None;
        };
        self.ascent = self.ascent.max(ascent);
        self.descent = self.descent.max(descent);
        self.selected_face_line_gap = self.selected_face_line_gap.max(line_gap);
        self.has_face_metrics = true;
        HorizontalLineRawMetrics::new(ascent, descent, line_gap)
    }

    /// Uses the resolver-selected primary face for typography spacing while
    /// retaining all selected faces for the glyph-content envelope.
    ///
    /// A fallback can be the first face that supplies a glyph. It must not
    /// therefore silently replace the collection's primary line-gap policy.
    pub(crate) fn include_primary_face(
        &mut self,
        database: &FontDatabase,
        face: FontFaceId,
        font_size: f32,
    ) {
        let Some(metrics) = database.face_metrics(face).ok().flatten() else {
            return;
        };
        let Some((_, _, line_gap)) = scaled_layout_extents(metrics, font_size) else {
            return;
        };
        self.primary_face_line_gap = Some(line_gap);
        self.has_primary_face_metrics = true;
    }

    pub(crate) fn resolve_content_envelope(
        &self,
        requested_line_height: f32,
    ) -> Option<SelectedFaceLineEnvelope> {
        self.has_face_metrics.then(|| {
            let content_height = self.ascent + self.descent;
            let line_gap = self
                .primary_face_line_gap
                .unwrap_or(self.selected_face_line_gap);
            let line_height = requested_line_height.max(content_height + line_gap);
            let leading = (line_height - content_height).max(0.0) * 0.5;
            SelectedFaceLineEnvelope {
                baseline_from_top: leading + self.ascent,
                line_height,
            }
        })
    }

    pub(crate) fn raw_horizontal_metrics(&self) -> Option<HorizontalLineRawMetrics> {
        self.has_face_metrics.then(|| {
            HorizontalLineRawMetrics::new(
                self.ascent,
                self.descent,
                self.primary_face_line_gap
                    .unwrap_or(self.selected_face_line_gap),
            )
        })?
    }

    fn font_chain_metric_envelope(&self) -> Option<FontChainLineMetricEnvelope> {
        (self.has_face_metrics && self.has_primary_face_metrics).then(|| {
            FontChainLineMetricEnvelope {
                max_ascent: self.ascent,
                max_descent: self.descent,
                primary_line_gap: self.primary_face_line_gap.unwrap_or_default(),
            }
        })
    }
}

fn scaled_layout_extents(metrics: FontAssetFaceMetrics, font_size: f32) -> Option<(f32, f32, f32)> {
    (metrics.units_per_em > 0).then(|| {
        let scale = font_size.max(1.0) / f32::from(metrics.units_per_em);
        // `FontFaceMetadata` receives these fields from ttf-parser's normalized face metrics:
        // USE_TYPO_METRICS first, hhea second, and Windows only as the parser's last-resort
        // fallback. Do not re-promote the raw Windows clipping bounds here.
        (
            f32::from(metrics.ascender.max(0)) * scale,
            f32::from(metrics.descender.saturating_neg().max(0)) * scale,
            f32::from(metrics.line_gap.max(0)) * scale,
        )
    })
}

#[cfg(test)]
#[path = "tests/line_metrics.rs"]
mod tests;
