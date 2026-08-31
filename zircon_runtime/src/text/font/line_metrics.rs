use std::collections::HashSet;

use super::fallback_cache::{
    fallback_query_identity, fallback_query_identity_for_asset, line_metric_envelope_cache_key,
};
use super::matching::{
    FontFamilyCandidateScope, ScopedFontFamilyCandidate, dedupe_scoped_families,
};
use super::{FontDatabase, font_query_for_text_style};
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
mod tests {
    use std::path::Path;

    use crate::asset::{FontAsset, FontAssetFaceMetrics, FontAssetRenderStrategy};
    use crate::text::font::FontDatabase;
    use crate::text::{
        CompositeFontDescriptor, FontFamilyName, FontScript, SubFontRange, TextStyle,
    };

    use super::{
        SelectedFaceLineExtents, font_chain_line_metric_envelope,
        primary_face_covers_all_hard_line_content, scaled_layout_extents,
    };

    #[test]
    fn primary_only_height_certificate_excludes_fallback_content() {
        let mut database = FontDatabase::default();
        let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts");
        database
            .register_font_file(assets.join("FiraSans-Regular.ttf"), Some("Primary"), 0)
            .expect("register primary font");
        let style = TextStyle {
            font_family: Some("Primary".to_string()),
            ..TextStyle::default()
        };

        assert!(primary_face_covers_all_hard_line_content(
            &database,
            &style,
            "Latin\r\ncontent\u{2028}only"
        ));
        assert!(!primary_face_covers_all_hard_line_content(
            &database,
            &style,
            "\u{4e16}\u{754c}"
        ));
    }

    #[test]
    fn font_chain_metric_envelope_uses_the_requested_font_asset_owner() {
        let mut database = FontDatabase::default();
        let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts");
        database
            .register_font_file(
                assets.join("FiraSans-Regular.ttf"),
                Some("Project Default Sans"),
                0,
            )
            .expect("register project default font");
        assert!(database.set_default_ui_family("Project Default Sans"));
        let owner = "res://fonts/layout-mono.font.toml";
        let asset = FontAsset {
            source: "FiraMono-subset.ttf".to_string(),
            family: Some("Layout Asset Mono".to_string()),
            render_mode: None,
            face_index: 0,
            family_members: Vec::new(),
            variable_instances: Vec::new(),
            fallback_families: Vec::new(),
            composite_font: None,
            render_strategy: FontAssetRenderStrategy::default(),
            metadata: None,
        };
        let registered = database
            .replace_font_asset(owner, &asset, assets.join("FiraMono-subset.ttf"))
            .expect("register layout font asset");
        let asset_face = registered.faces[0];
        let font_size = 20.0;
        let style = TextStyle {
            font: Some(owner.to_string()),
            font_size,
            ..TextStyle::default()
        };

        let envelope = font_chain_line_metric_envelope(&database, &style)
            .expect("font asset chain must produce a line metric envelope");
        let metrics = database
            .face_metrics(asset_face)
            .expect("read asset face metrics")
            .expect("asset face metrics are tracked");
        let (ascent, descent, line_gap) =
            scaled_layout_extents(metrics, font_size).expect("valid asset metrics");

        assert!(
            (envelope.minimum_line_height() - (ascent + descent + line_gap)).abs() < 0.001,
            "the fixed-height certificate must use the FontObject primary face"
        );
    }

    #[test]
    fn unavailable_font_asset_typeface_does_not_change_the_default_metric_envelope() {
        let mut database = FontDatabase::default();
        let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts");
        database
            .register_font_file(
                assets.join("FiraSans-Regular.ttf"),
                Some("Unavailable Owner Typeface"),
                0,
            )
            .expect("global homonym should register");
        let runtime_default = database
            .register_font_file(
                assets.join("FiraMono-subset.ttf"),
                Some("Runtime Metric Default"),
                0,
            )
            .expect("runtime metric default should register");
        assert!(database.set_runtime_default_primary_face(runtime_default));
        let font_size = 20.0;
        let style = TextStyle {
            font: Some("res://fonts/unavailable-metrics.font.toml".to_string()),
            font_family: Some("Unavailable Owner Typeface".to_string()),
            font_size,
            ..TextStyle::default()
        };

        let envelope = font_chain_line_metric_envelope(&database, &style)
            .expect("runtime default must provide the recovery metric envelope");
        let metrics = database
            .face_metrics(runtime_default)
            .expect("read runtime default metrics")
            .expect("runtime default metrics are tracked");
        let (ascent, descent, line_gap) =
            scaled_layout_extents(metrics, font_size).expect("valid runtime default metrics");

        assert!((envelope.minimum_line_height() - (ascent + descent + line_gap)).abs() < 0.001);
    }

    #[test]
    fn font_chain_metric_envelope_includes_the_runtime_last_resort_face() {
        let mut database = FontDatabase::default();
        let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts");
        let primary = database
            .register_font_file(assets.join("FiraSans-Regular.ttf"), Some("Primary"), 0)
            .expect("register primary font");
        let last_resort = database
            .register_font_file(
                assets.join("FiraMono-subset.ttf"),
                Some("Runtime Last Resort"),
                0,
            )
            .expect("register last-resort font");
        assert!(database.set_runtime_last_resort_face(last_resort));
        let style = TextStyle {
            font_family: Some("Primary".to_string()),
            font_size: 20.0,
            ..TextStyle::default()
        };
        let primary_metrics = database
            .face_metrics(primary)
            .expect("read primary metrics")
            .expect("primary metrics are tracked");
        let last_resort_metrics = database
            .face_metrics(last_resort)
            .expect("read last-resort metrics")
            .expect("last-resort metrics are tracked");
        let (primary_ascent, primary_descent, primary_gap) =
            scaled_layout_extents(primary_metrics, style.font_size).expect("primary extents");
        let (last_resort_ascent, last_resort_descent, _) =
            scaled_layout_extents(last_resort_metrics, style.font_size)
                .expect("last-resort extents");

        let envelope = font_chain_line_metric_envelope(&database, &style)
            .expect("last-resort chain must expose a metric envelope");

        let expected = primary_ascent.max(last_resort_ascent)
            + primary_descent.max(last_resort_descent)
            + primary_gap;
        assert!((envelope.minimum_line_height() - expected).abs() < 0.001);
    }

    #[test]
    fn layout_extents_do_not_promote_windows_clip_metrics() {
        let metrics = FontAssetFaceMetrics {
            units_per_em: 1_000,
            ascender: 700,
            descender: -200,
            line_gap: 100,
            uses_typographic_metrics: false,
            windows_ascender: 1_100,
            windows_descender: 450,
            ..FontAssetFaceMetrics::default()
        };

        let (ascent, descent, line_gap) =
            scaled_layout_extents(metrics, 20.0).expect("valid normalized face metrics");

        assert_eq!((ascent, descent, line_gap), (14.0, 4.0, 2.0));
    }

    #[test]
    fn selected_face_content_uses_primary_face_line_gap() {
        let extents = super::SelectedFaceLineExtents {
            ascent: 14.0,
            descent: 4.0,
            selected_face_line_gap: 6.0,
            primary_face_line_gap: Some(2.0),
            has_face_metrics: true,
            has_primary_face_metrics: true,
        };

        let envelope = extents
            .resolve_content_envelope(12.0)
            .expect("selected face metrics must resolve an envelope");

        assert_eq!(envelope.line_height, 20.0);
        assert_eq!(envelope.baseline_from_top, 15.0);
    }

    #[test]
    fn selected_face_extents_exposes_validated_raw_horizontal_metrics() {
        let extents = super::SelectedFaceLineExtents {
            ascent: 14.0,
            descent: 4.0,
            selected_face_line_gap: 6.0,
            primary_face_line_gap: Some(2.0),
            has_face_metrics: true,
            has_primary_face_metrics: true,
        };

        let metrics = extents
            .raw_horizontal_metrics()
            .expect("selected faces provide raw horizontal metrics");

        assert_eq!(metrics.ascent(), 14.0);
        assert_eq!(metrics.descent(), 4.0);
        assert_eq!(metrics.line_spacing_gap(), 2.0);
    }

    #[test]
    fn font_database_envelope_keeps_primary_spacing_when_fallback_supplies_glyphs() {
        let mut database = FontDatabase::default();
        let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts");
        let primary = database
            .register_font_file(assets.join("FiraSans-Regular.ttf"), Some("Primary"), 0)
            .expect("register primary font");
        let fallback = database
            .register_font_file(assets.join("FiraMono-subset.ttf"), Some("Fallback"), 0)
            .expect("register fallback font");
        let primary_metrics = database
            .face_metrics(primary)
            .expect("read primary metrics")
            .expect("primary metrics are tracked");
        let fallback_metrics = database
            .face_metrics(fallback)
            .expect("read fallback metrics")
            .expect("fallback metrics are tracked");
        let font_size = 20.0;
        let requested_line_height = 12.0;
        let (fallback_ascent, fallback_descent, _) =
            scaled_layout_extents(fallback_metrics, font_size).expect("valid fallback metrics");
        let (_, _, primary_gap) =
            scaled_layout_extents(primary_metrics, font_size).expect("valid primary metrics");

        let mut extents = SelectedFaceLineExtents::default();
        let _ = extents.include_face(&database, fallback, font_size);
        extents.include_primary_face(&database, primary, font_size);
        let envelope = extents
            .resolve_content_envelope(requested_line_height)
            .expect("fallback glyph metrics resolve an envelope");

        let expected_height =
            requested_line_height.max(fallback_ascent + fallback_descent + primary_gap);
        let expected_baseline =
            (expected_height - fallback_ascent - fallback_descent).max(0.0) * 0.5 + fallback_ascent;
        assert!((envelope.line_height - expected_height).abs() < 0.001);
        assert!((envelope.baseline_from_top - expected_baseline).abs() < 0.001);
    }

    #[test]
    fn font_chain_height_envelope_includes_eligible_composite_faces_before_coverage() {
        let mut database = FontDatabase::default();
        let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts");
        let primary = database
            .register_font_file(assets.join("FiraSans-Regular.ttf"), Some("Primary"), 0)
            .expect("register primary font");
        let fallback = database
            .register_font_file(
                assets.join("FiraMono-subset.ttf"),
                Some("Composite Fallback"),
                0,
            )
            .expect("register composite fallback font");
        database.set_project_composite_font(Some(CompositeFontDescriptor {
            default_family: FontFamilyName::from("Primary"),
            sub_fonts: vec![SubFontRange {
                family: FontFamilyName::from("Composite Fallback"),
                scripts: vec![FontScript::Han],
                ranges: vec![(0x4E00, 0x9FFF)],
                cultures: Vec::new(),
            }],
        }));
        let style = TextStyle {
            font_family: Some("Primary".to_string()),
            font_size: 20.0,
            ..TextStyle::default()
        };
        let primary_metrics = database
            .face_metrics(primary)
            .expect("read primary metrics")
            .expect("primary metrics are tracked");
        let fallback_metrics = database
            .face_metrics(fallback)
            .expect("read fallback metrics")
            .expect("fallback metrics are tracked");
        let (primary_ascent, primary_descent, primary_gap) =
            scaled_layout_extents(primary_metrics, style.font_size).expect("primary extents");
        let (fallback_ascent, fallback_descent, _) =
            scaled_layout_extents(fallback_metrics, style.font_size).expect("fallback extents");

        let envelope = font_chain_line_metric_envelope(&database, &style)
            .expect("eligible chain must expose a metric envelope");

        let expected_minimum_height = primary_ascent.max(fallback_ascent)
            + primary_descent.max(fallback_descent)
            + primary_gap;
        assert!((envelope.minimum_line_height() - expected_minimum_height).abs() < 0.001);
        assert!(envelope.certifies_uniform_line_height(expected_minimum_height));
        assert!(!envelope.certifies_uniform_line_height(expected_minimum_height - 0.01));

        let report_after_first = database.fallback_cache_report();
        assert_eq!(report_after_first.line_metric_envelope_misses, 1);
        assert_eq!(report_after_first.line_metric_envelope_hits, 0);

        assert_eq!(
            font_chain_line_metric_envelope(&database, &style),
            Some(envelope)
        );
        let report_after_second = database.fallback_cache_report();
        assert_eq!(report_after_second.line_metric_envelope_misses, 1);
        assert_eq!(report_after_second.line_metric_envelope_hits, 1);
        assert_eq!(report_after_second.line_metric_envelope_entry_count, 1);
    }
}
