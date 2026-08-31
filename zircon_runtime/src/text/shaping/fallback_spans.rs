use std::ops::Range;

use crate::text::TextStyle;
use unicode_segmentation::UnicodeSegmentation;

use crate::text::font::{FallbackResolution, FontDatabase, font_query_for_text_style};
use crate::text::model::TextFontResolutionReport;
use crate::text::{BackendShapeRequest, FontFaceId, InstancedFaceId, TextRange};

use super::script_segment::ParagraphTextAnalysis;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FallbackTextSpan {
    pub(crate) range: Range<usize>,
    pub(crate) family: Option<String>,
    /// The query's resolved primary face, retained even when this span shapes with fallback.
    ///
    /// Text03 uses this identity for the eventual composite-line metric policy. It must not be
    /// inferred from the first selected span because a fallback glyph can start the source.
    pub(crate) primary_face: FontFaceId,
    pub(crate) resolution: FallbackResolution,
    pub(crate) instance: Option<InstancedFaceId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FallbackItemizationError {
    PrimaryFaceUnavailable,
}

/// Returns the resolver-selected primary face that is common to one itemized request.
///
/// The selected face of the first span may be a fallback, so callers that form a canonical
/// fragment must use this identity rather than inspect an arbitrary selected span.
pub(crate) fn fallback_primary_face(spans: &[FallbackTextSpan]) -> Option<FontFaceId> {
    let primary_face = spans.first()?.primary_face;
    debug_assert!(
        spans.iter().all(|span| span.primary_face == primary_face),
        "one fallback itemization request must retain exactly one primary face"
    );
    Some(primary_face)
}

pub(crate) fn fallback_text_spans(
    text: &str,
    request: BackendShapeRequest<'_>,
    database: &FontDatabase,
    analysis: &ParagraphTextAnalysis,
) -> Result<Vec<FallbackTextSpan>, FallbackItemizationError> {
    fallback_text_spans_with_report(text, request, database, analysis).map(|(spans, _)| spans)
}

pub(crate) fn fallback_text_spans_with_report(
    text: &str,
    request: BackendShapeRequest<'_>,
    database: &FontDatabase,
    analysis: &ParagraphTextAnalysis,
) -> Result<(Vec<FallbackTextSpan>, TextFontResolutionReport), FallbackItemizationError> {
    debug_assert_eq!(
        analysis.unicode_data_snapshot(),
        request.unicode_data_snapshot(),
        "fallback analysis must use the request-bound Unicode snapshot"
    );
    let query = font_query_for_text_style(request.style);
    let query =
        database.constrain_font_query_to_request_owner(&query, request.style.font.as_deref());
    let default_family = request
        .style
        .font_family
        .as_deref()
        .map(str::trim)
        .filter(|family| !family.is_empty());
    let mut face_resolver = database
        .begin_shaping_face_resolution_for_request(
            query.as_ref(),
            request.style.font.as_deref(),
            request.language_fallback_key(),
        )
        .ok_or(FallbackItemizationError::PrimaryFaceUnavailable)?;
    let primary_face = face_resolver.primary_face();
    if face_resolver.primary_covers_text(text) {
        let face = primary_face;
        let family = database
            .face_family_name(face)
            .map(|family| family.0)
            .or_else(|| default_family.map(str::to_string));
        let instance = database
            .effective_instance_id(
                face,
                TextStyle::normalized_font_weight(request.style.font_weight),
            )
            .ok();
        let spans = vec![FallbackTextSpan {
            range: 0..text.len(),
            family,
            primary_face,
            resolution: face_resolver.primary_resolution(),
            instance,
        }];
        let report = face_resolver.take_resolution_report();
        return Ok((spans, report));
    }
    let mut spans = Vec::<FallbackTextSpan>::new();
    let mut cluster_codepoints = Vec::new();
    for (start, cluster) in text.grapheme_indices(true) {
        let end = start + cluster.len();
        let range = TextRange { start, end };
        cluster_codepoints.clear();
        cluster_codepoints.extend(cluster.chars());
        let resolution =
            face_resolver.resolve(analysis.font_script_for_range(range), &cluster_codepoints);
        let face = resolution.face();
        let instance = database
            .effective_instance_id(
                face,
                TextStyle::normalized_font_weight(request.style.font_weight),
            )
            .ok();
        if let Some(previous) = spans.last_mut() {
            if previous.resolution == resolution
                && previous.instance == instance
                && previous.range.end == start
            {
                previous.range.end = end;
                continue;
            }
        }
        let family = database
            .face_family_name(face)
            .map(|family| family.0)
            .or_else(|| default_family.map(str::to_string));
        spans.push(FallbackTextSpan {
            range: start..end,
            family,
            primary_face,
            resolution,
            instance,
        });
    }
    let report = face_resolver.take_resolution_report();
    Ok((spans, report))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{fallback_primary_face, fallback_text_spans};
    use crate::asset::{FontAsset, FontAssetRenderStrategy};
    use crate::core::framework::text::TextDirection;
    use crate::text::font::FontDatabase;
    use crate::text::shaping::script_segment::ParagraphTextAnalysis;
    use crate::text::{
        BackendShapeRequest, CompositeFontDescriptor, FontFaceDescriptor, FontFamilyName,
        FontScript, SubFontRange, TextRange, TextStyle,
    };

    #[test]
    fn fallback_spans_resolve_font_asset_owner_before_database_defaults() {
        let fonts = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts");
        let mut database = FontDatabase::default();
        let default_face = database
            .register_font_file(
                fonts.join("FiraSans-Regular.ttf"),
                Some("Project Default Sans"),
                0,
            )
            .expect("project default face should register");
        assert!(database.set_default_ui_family("Project Default Sans"));

        let owner = "res://fonts/asset-mono.font.toml";
        let asset = FontAsset {
            source: "FiraMono-subset.ttf".to_string(),
            family: Some("Asset Mono".to_string()),
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
            .replace_font_asset(owner, &asset, fonts.join("FiraMono-subset.ttf"))
            .expect("font asset owner should register");
        let asset_face = registered.faces[0];
        let style = TextStyle {
            font: Some(owner.to_string()),
            ..TextStyle::default()
        };
        let text = "asset owner";
        let request = BackendShapeRequest::horizontal(
            text,
            &style,
            TextDirection::LeftToRight,
            TextRange {
                start: 0,
                end: text.len(),
            },
        )
        .canonicalized()
        .expect("fixture request should canonicalize");
        let request = request.request();
        let analysis = ParagraphTextAnalysis::new(text, request.unicode_data_snapshot());

        let spans = fallback_text_spans(text, request, &database, &analysis)
            .expect("registered font asset should itemize");

        assert_eq!(fallback_primary_face(&spans), Some(asset_face));
        assert_ne!(asset_face, default_face);
    }

    #[test]
    fn unavailable_font_asset_does_not_leak_its_typeface_into_global_matching() {
        let fonts = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts");
        let mut database = FontDatabase::default();
        let global_homonym = database
            .register_font_file(
                fonts.join("FiraSans-Regular.ttf"),
                Some("Owner Local Typeface"),
                0,
            )
            .expect("global homonym should register");
        let runtime_default = database
            .register_font_file(
                fonts.join("FiraMono-subset.ttf"),
                Some("Runtime Recovery Default"),
                0,
            )
            .expect("runtime default should register");
        assert!(database.set_runtime_default_primary_face(runtime_default));
        let style = TextStyle {
            font: Some("res://fonts/unavailable.font.toml".to_string()),
            font_family: Some("Owner Local Typeface".to_string()),
            ..TextStyle::default()
        };
        let text = "A";
        let request = BackendShapeRequest::horizontal(
            text,
            &style,
            TextDirection::LeftToRight,
            TextRange {
                start: 0,
                end: text.len(),
            },
        );
        let analysis = ParagraphTextAnalysis::new(text, request.unicode_data_snapshot());

        let spans = fallback_text_spans(text, request, &database, &analysis)
            .expect("runtime default should recover an unavailable font object");

        assert_eq!(fallback_primary_face(&spans), Some(runtime_default));
        assert_ne!(runtime_default, global_homonym);
    }

    #[test]
    fn fallback_spans_prefer_the_requested_assets_composite_member() {
        let fonts = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts");
        let collection = fonts.join("ZirconDefaultComposite-subset.ttc");
        let mut database = FontDatabase::default();
        let competing_face = database
            .register_font_file(&collection, Some("Zircon Noto Sans CJK SC Proof"), 1)
            .expect("competing global CJK face should register");
        let owner = "res://fonts/scoped-composite.font.toml";
        let asset =
            FontAsset::from_toml_str(include_str!("../../../assets/fonts/default.font.toml"))
                .expect("packaged composite fixture should parse");
        let registered = database
            .replace_font_asset(owner, &asset, &collection)
            .expect("composite font asset should register");
        let asset_cjk_face = registered.faces[1];
        let compiled_at_registration = database.fallback_cache_report().composite_compile_count;
        let style = TextStyle {
            font: Some(owner.to_string()),
            language: Some("zh-Hans".to_string()),
            ..TextStyle::default()
        };
        let text = "界";
        let request = BackendShapeRequest::horizontal(
            text,
            &style,
            TextDirection::LeftToRight,
            TextRange {
                start: 0,
                end: text.len(),
            },
        )
        .canonicalized()
        .expect("fixture language should canonicalize");
        let request = request.request();
        let analysis = ParagraphTextAnalysis::new(text, request.unicode_data_snapshot());

        let spans = fallback_text_spans(text, request, &database, &analysis)
            .expect("asset composite should itemize");

        assert_eq!(spans[0].resolution.face(), asset_cjk_face);
        assert_ne!(asset_cjk_face, competing_face);
        assert_eq!(
            database.fallback_cache_report().composite_compile_count,
            compiled_at_registration,
            "font object composite indexes must be compiled at generation publication, not per request"
        );
    }

    #[test]
    fn fallback_spans_do_not_consume_another_assets_fallback_chain() {
        let fonts = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts");
        let mut database = FontDatabase::default();
        let first_owner = "res://fonts/first-object.font.toml";
        let first_asset = FontAsset {
            source: "FiraSans-Regular.ttf".to_string(),
            family: Some("First Object Sans".to_string()),
            render_mode: None,
            face_index: 0,
            family_members: Vec::new(),
            variable_instances: Vec::new(),
            fallback_families: Vec::new(),
            composite_font: None,
            render_strategy: FontAssetRenderStrategy::default(),
            metadata: None,
        };
        let first = database
            .replace_font_asset(
                first_owner,
                &first_asset,
                fonts.join("FiraSans-Regular.ttf"),
            )
            .expect("first font object should register");
        let second_asset = FontAsset {
            source: "ZirconDefaultComposite-subset.ttc".to_string(),
            family: Some("Second Object CJK".to_string()),
            render_mode: None,
            face_index: 1,
            family_members: Vec::new(),
            variable_instances: Vec::new(),
            fallback_families: vec!["Second Object CJK".to_string()],
            composite_font: None,
            render_strategy: FontAssetRenderStrategy::default(),
            metadata: None,
        };
        let second = database
            .replace_font_asset(
                "res://fonts/second-object.font.toml",
                &second_asset,
                fonts.join("ZirconDefaultComposite-subset.ttc"),
            )
            .expect("second font object should register");
        let style = TextStyle {
            font: Some(first_owner.to_string()),
            ..TextStyle::default()
        };
        let text = "界";
        let request = BackendShapeRequest::horizontal(
            text,
            &style,
            TextDirection::LeftToRight,
            TextRange {
                start: 0,
                end: text.len(),
            },
        );
        let analysis = ParagraphTextAnalysis::new(text, request.unicode_data_snapshot());

        let spans = fallback_text_spans(text, request, &database, &analysis)
            .expect("first font object should retain its primary face as last resort");

        assert_eq!(spans[0].resolution.face(), first.faces[0]);
        assert!(spans[0].resolution.is_missing());
        assert_ne!(spans[0].resolution.face(), second.faces[0]);
    }

    #[test]
    fn missing_owner_local_typeface_does_not_become_an_implicit_global_fallback() {
        let fonts = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts");
        let mut database = FontDatabase::default();
        let global_homonym = database
            .register_test_face_with_coverage(
                FontFaceDescriptor::regular("Missing Owner Typeface"),
                &['界'],
            )
            .expect("global homonym should register");
        let owner = "res://fonts/local-typeface-owner.font.toml";
        let asset = FontAsset {
            source: "FiraMono-subset.ttf".to_string(),
            family: Some("Actual Owner Primary".to_string()),
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
            .replace_font_asset(owner, &asset, fonts.join("FiraMono-subset.ttf"))
            .expect("font object should register");
        let style = TextStyle {
            font: Some(owner.to_string()),
            font_family: Some("Missing Owner Typeface".to_string()),
            ..TextStyle::default()
        };
        let text = "界";
        let request = BackendShapeRequest::horizontal(
            text,
            &style,
            TextDirection::LeftToRight,
            TextRange {
                start: 0,
                end: text.len(),
            },
        );
        let analysis = ParagraphTextAnalysis::new(text, request.unicode_data_snapshot());

        let spans = fallback_text_spans(text, request, &database, &analysis)
            .expect("owner primary should remain the last-resort face");

        assert_eq!(spans[0].resolution.face(), registered.faces[0]);
        assert!(spans[0].resolution.is_missing());
        assert_ne!(spans[0].resolution.face(), global_homonym);
    }

    #[test]
    fn explicit_owner_fallback_family_can_resolve_a_global_face() {
        let fonts = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts");
        let mut database = FontDatabase::default();
        let external = database
            .register_test_face_with_coverage(
                FontFaceDescriptor::regular("Authored External CJK"),
                &['界'],
            )
            .expect("external fallback should register");
        let owner = "res://fonts/external-fallback-owner.font.toml";
        let asset = FontAsset {
            source: "FiraMono-subset.ttf".to_string(),
            family: Some("External Fallback Primary".to_string()),
            render_mode: None,
            face_index: 0,
            family_members: Vec::new(),
            variable_instances: Vec::new(),
            fallback_families: vec!["Authored External CJK".to_string()],
            composite_font: None,
            render_strategy: FontAssetRenderStrategy::default(),
            metadata: None,
        };
        database
            .replace_font_asset(owner, &asset, fonts.join("FiraMono-subset.ttf"))
            .expect("font object should register");
        let style = TextStyle {
            font: Some(owner.to_string()),
            ..TextStyle::default()
        };
        let text = "界";
        let request = BackendShapeRequest::horizontal(
            text,
            &style,
            TextDirection::LeftToRight,
            TextRange {
                start: 0,
                end: text.len(),
            },
        );
        let analysis = ParagraphTextAnalysis::new(text, request.unicode_data_snapshot());

        let spans = fallback_text_spans(text, request, &database, &analysis)
            .expect("authored external fallback should resolve");

        assert_eq!(spans[0].resolution.face(), external);
        assert!(!spans[0].resolution.is_missing());
    }

    #[test]
    fn fallback_spans_keep_primary_coverage_in_one_contiguous_span() {
        let mut database = FontDatabase::default();
        let source =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraSans-Regular.ttf");
        let primary = database
            .register_font_file(source, Some("Layout Primary"), 0)
            .expect("tracked layout font should register");
        let style = TextStyle {
            font_family: Some("Layout Primary".to_string()),
            ..TextStyle::default()
        };
        let text = "Workbench layout label";
        let analysis = ParagraphTextAnalysis::new(text, None);
        let spans = fallback_text_spans(
            text,
            BackendShapeRequest::horizontal(
                text,
                &style,
                TextDirection::LeftToRight,
                TextRange {
                    start: 0,
                    end: text.len(),
                },
            ),
            &database,
            &analysis,
        )
        .expect("primary font must itemize");

        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].range, 0..text.len());
        assert_eq!(spans[0].resolution.face(), primary);
    }

    #[test]
    fn fallback_spans_keep_primary_identity_when_a_fallback_starts_the_text() {
        let mut database = FontDatabase::default();
        let primary = database
            .register_test_face_with_coverage(FontFaceDescriptor::regular("Layout Primary"), &['A'])
            .expect("primary test face must register");
        let fallback = database
            .register_test_face_with_coverage(FontFaceDescriptor::regular("Layout CJK"), &['界'])
            .expect("fallback test face must register");
        assert!(
            database.set_project_composite_font(Some(CompositeFontDescriptor {
                default_family: FontFamilyName::from("Layout Primary"),
                sub_fonts: vec![SubFontRange {
                    family: FontFamilyName::from("Layout CJK"),
                    scripts: vec![FontScript::Han],
                    ranges: vec![(0x4E00, 0x9FFF)],
                    cultures: Vec::new(),
                }],
            }))
        );
        let style = TextStyle {
            font_family: Some("Layout Primary".to_string()),
            ..TextStyle::default()
        };
        let text = "界A";
        let analysis = ParagraphTextAnalysis::new(text, None);

        let spans = fallback_text_spans(
            text,
            BackendShapeRequest::horizontal(
                text,
                &style,
                TextDirection::LeftToRight,
                TextRange {
                    start: 0,
                    end: text.len(),
                },
            ),
            &database,
            &analysis,
        )
        .expect("primary and fallback fonts must itemize");

        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].resolution.face(), fallback);
        assert_eq!(spans[1].resolution.face(), primary);
        assert_eq!(fallback_primary_face(&spans), Some(primary));
        assert!(
            spans.iter().all(|span| span.primary_face == primary),
            "the primary face must survive itemization independently of selected fallback faces"
        );
    }

    #[test]
    fn fallback_itemization_reuses_cluster_codepoint_storage() {
        let source = include_str!("fallback_spans.rs");
        let compact = source
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>();

        assert!(!compact.contains("text.chars().collect::<Vec<_>>()"));
        assert!(compact.contains("cluster_codepoints.clear()"));
    }

    #[test]
    fn fallback_spans_preserve_partial_and_complete_receipt_boundaries() {
        use crate::text::font::FallbackResolutionSource;

        let mut database = FontDatabase::default();
        let primary = database
            .register_test_face_with_coverage(
                FontFaceDescriptor::regular("Partial Primary"),
                &['A', 'B'],
            )
            .expect("partial primary test face must register");
        let style = TextStyle {
            font_family: Some("Partial Primary".to_string()),
            ..TextStyle::default()
        };
        let text = "A\u{0301}B";
        let analysis = ParagraphTextAnalysis::new(text, None);

        let spans = fallback_text_spans(
            text,
            BackendShapeRequest::horizontal(
                text,
                &style,
                TextDirection::LeftToRight,
                TextRange {
                    start: 0,
                    end: text.len(),
                },
            ),
            &database,
            &analysis,
        )
        .expect("partial coverage must retain a real-face itemization receipt");

        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].range, 0.."A\u{0301}".len());
        assert_eq!(spans[0].resolution.face(), primary);
        assert!(spans[0].resolution.is_missing());
        assert_eq!(
            spans[0].resolution.source(),
            FallbackResolutionSource::PartialCoverage
        );
        assert_eq!(spans[1].range, "A\u{0301}".len()..text.len());
        assert_eq!(spans[1].resolution.face(), primary);
        assert!(!spans[1].resolution.is_missing());
        assert_eq!(
            spans[1].resolution.source(),
            FallbackResolutionSource::Primary
        );
    }

    #[test]
    fn fallback_itemization_rejects_a_missing_primary_face() {
        use super::FallbackItemizationError;

        let database = FontDatabase::default();
        let style = TextStyle::default();
        let text = "A";
        let analysis = ParagraphTextAnalysis::new(text, None);

        let result = fallback_text_spans(
            text,
            BackendShapeRequest::horizontal(
                text,
                &style,
                TextDirection::LeftToRight,
                TextRange {
                    start: 0,
                    end: text.len(),
                },
            ),
            &database,
            &analysis,
        );

        assert_eq!(
            result,
            Err(FallbackItemizationError::PrimaryFaceUnavailable)
        );
    }
}
