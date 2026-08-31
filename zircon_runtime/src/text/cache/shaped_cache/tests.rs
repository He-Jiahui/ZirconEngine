use super::*;
use crate::core::framework::text::TextFontCollectionHandle;
use crate::text::{
    HorizontalGlyphMetricSpan, ShapedGlyph, ShapedHardLine, TextHorizontalCompositionReceipt,
    TextShapingFailureCode, TextShapingFailureDependency, TextShapingFailureDisposition,
    TextShapingFailurePhase, TextShapingFailureReceipt,
};

#[test]
fn shaped_run_cache_identity_includes_the_font_collection() {
    let text = "collection identity";
    let style = TextStyle::default();
    let request = BackendShapeRequest::horizontal(
        text,
        &style,
        TextDirection::LeftToRight,
        TextRange {
            start: 0,
            end: text.len(),
        },
    );
    let first = ShapedRunCacheLookupKey::from_request_in_font_collection(
        &request,
        TextFontCollectionHandle::new(41),
        1,
    );
    let second = ShapedRunCacheLookupKey::from_request_in_font_collection(
        &request,
        TextFontCollectionHandle::new(42),
        1,
    );

    assert_ne!(first.exact_fingerprint(), second.exact_fingerprint());
    assert!(!ShapedRunCacheKey::from_lookup(&first).matches_lookup(&second));
}

#[test]
fn shaped_run_cache_reuses_single_paragraph_auto_result_for_resolved_ltr() {
    let text = "editor base.zui";
    let style = TextStyle::default();
    let source_range = TextRange {
        start: 0,
        end: text.len(),
    };
    let auto = BackendShapeRequest::horizontal(text, &style, TextDirection::Auto, source_range);
    let explicit =
        BackendShapeRequest::horizontal(text, &style, TextDirection::LeftToRight, source_range);
    let auto_key = ShapedRunCacheKey::from_request(&auto);
    let explicit_key = ShapedRunCacheKey::from_request(&explicit);
    let mut cache = ShapedRunCache::with_capacity(4);

    assert!(cache.get(&auto_key, text).is_none());
    cache.insert(auto_key, cached_run(&auto, TextDirection::LeftToRight));
    assert!(cache.get(&explicit_key, text).is_some());
    assert_eq!(cache.report().miss_count, 1);
    assert_eq!(cache.report().hit_count, 1);
}

#[test]
fn shaped_run_cache_uses_request_level_canonical_language_identity() {
    let text = "locale";
    let first_style = TextStyle {
        language: Some("ZH_hans_cn".to_string()),
        ..TextStyle::default()
    };
    let second_style = TextStyle {
        language: Some("zh-Hans-CN".to_string()),
        ..TextStyle::default()
    };
    let source_range = TextRange {
        start: 0,
        end: text.len(),
    };
    let first = BackendShapeRequest::horizontal(
        text,
        &first_style,
        TextDirection::LeftToRight,
        source_range,
    )
    .canonicalized()
    .expect("valid legacy separator form");
    let second = BackendShapeRequest::horizontal(
        text,
        &second_style,
        TextDirection::LeftToRight,
        source_range,
    )
    .canonicalized()
    .expect("valid canonical language form");

    assert_eq!(first.request().language, Some("zh-Hans-CN"));
    assert_eq!(
        ShapedRunCacheKey::from_request(&first.request()),
        ShapedRunCacheKey::from_request(&second.request())
    );
}

#[test]
fn shaped_run_cache_separates_italic_and_inherits_style_features() {
    let text = "office";
    let source_range = TextRange {
        start: 0,
        end: text.len(),
    };
    let plain_style = TextStyle::default();
    let styled = TextStyle {
        italic: true,
        features: Arc::from([
            OpenTypeFeature::new(*b"ss01", 1),
            OpenTypeFeature::new(*b"liga", 0),
        ]),
        ..TextStyle::default()
    };
    let plain = BackendShapeRequest::horizontal(
        text,
        &plain_style,
        TextDirection::LeftToRight,
        source_range,
    );
    let styled =
        BackendShapeRequest::horizontal(text, &styled, TextDirection::LeftToRight, source_range)
            .canonicalized()
            .expect("style features are valid");

    assert_ne!(
        ShapedRunCacheKey::from_request(&plain),
        ShapedRunCacheKey::from_request(&styled.request())
    );
    assert_eq!(
        styled.request().features(),
        &[
            OpenTypeFeature::new(*b"liga", 0),
            OpenTypeFeature::new(*b"ss01", 1),
        ]
    );
}

#[test]
fn shaped_run_cache_separates_font_assets_that_share_a_typeface_name() {
    let text = "asset identity";
    let first_style = TextStyle {
        font: Some("res://fonts/first.font.toml".to_string()),
        font_family: Some("Regular".to_string()),
        ..TextStyle::default()
    };
    let second_style = TextStyle {
        font: Some("res://fonts/second.font.toml".to_string()),
        font_family: Some("Regular".to_string()),
        ..TextStyle::default()
    };
    let source_range = TextRange {
        start: 0,
        end: text.len(),
    };
    let first = BackendShapeRequest::horizontal(
        text,
        &first_style,
        TextDirection::LeftToRight,
        source_range,
    );
    let second = BackendShapeRequest::horizontal(
        text,
        &second_style,
        TextDirection::LeftToRight,
        source_range,
    );
    let padded_style = TextStyle {
        font: Some(" res://fonts/first.font.toml ".to_string()),
        font_family: Some("Regular".to_string()),
        ..TextStyle::default()
    };
    let padded = BackendShapeRequest::horizontal(
        text,
        &padded_style,
        TextDirection::LeftToRight,
        source_range,
    );

    assert_ne!(
        ShapedRunCacheKey::from_request(&first),
        ShapedRunCacheKey::from_request(&second)
    );
    assert_ne!(
        ShapedRunCacheKey::from_request(&first),
        ShapedRunCacheKey::from_request(&padded),
        "font object identity is exact and must not inherit family-name trimming"
    );
}

#[test]
fn shaped_run_cache_separates_unicode_data_generations() {
    let text = "unicode generation";
    let style = TextStyle::default();
    let source_range = TextRange {
        start: 0,
        end: text.len(),
    };
    let current_snapshot = crate::text::compiled_unicode_data_snapshot_id();
    let next_snapshot =
        current_snapshot.with_generation_for_test(current_snapshot.generation() + 1);
    let current =
        BackendShapeRequest::horizontal(text, &style, TextDirection::LeftToRight, source_range)
            .with_unicode_data_snapshot_for_test(current_snapshot);
    let next =
        BackendShapeRequest::horizontal(text, &style, TextDirection::LeftToRight, source_range)
            .with_unicode_data_snapshot_for_test(next_snapshot);

    assert_ne!(
        ShapedRunCacheKey::from_request(&current),
        ShapedRunCacheKey::from_request(&next)
    );
    assert_eq!(
        cached_run(&next, TextDirection::LeftToRight).unicode_data_snapshot,
        next_snapshot
    );
}

#[test]
fn shaped_run_cache_does_not_alias_multi_paragraph_auto_with_forced_direction() {
    let style = TextStyle::default();
    for separator in [
        '\n', '\r', '\u{001c}', '\u{001d}', '\u{001e}', '\u{0085}', '\u{2029}',
    ] {
        let text = format!("abc{separator}\u{645}\u{631}\u{62d}\u{628}\u{627}");
        let source_range = TextRange {
            start: 0,
            end: text.len(),
        };
        let auto =
            BackendShapeRequest::horizontal(&text, &style, TextDirection::Auto, source_range);
        let explicit = BackendShapeRequest::horizontal(
            &text,
            &style,
            TextDirection::LeftToRight,
            source_range,
        );
        let auto_key = ShapedRunCacheKey::from_request(&auto);
        let explicit_key = ShapedRunCacheKey::from_request(&explicit);
        let mut cache = ShapedRunCache::with_capacity(4);

        cache.insert(auto_key, cached_run(&auto, TextDirection::LeftToRight));
        assert!(
            cache.get(&explicit_key, &text).is_none(),
            "paragraph separator U+{:04X} must prevent direction aliasing",
            separator as u32
        );
    }
}

#[test]
fn shaped_run_cache_estimate_counts_the_shared_source_once() {
    let text = "single source";
    let style = TextStyle::default();
    let request = BackendShapeRequest::horizontal(
        text,
        &style,
        TextDirection::LeftToRight,
        TextRange {
            start: 0,
            end: text.len(),
        },
    );
    let run = cached_run(&request, TextDirection::LeftToRight);
    let key = ShapedRunCacheKey::from_request(&request);
    let payload_floor = size_of::<ShapedRunCacheEntry>()
        .saturating_add(size_of::<ShapedGlyphRun>())
        .saturating_add(text.len());

    assert!(estimated_entry_bytes(&key, &run) > payload_floor);
}

#[test]
fn shaped_run_cache_bucket_removal_does_not_scan_candidates() {
    let source = include_str!("../shaped_cache.rs");
    let linear_search = concat!("candidates.iter()", ".position");

    assert!(
        !source.contains(linear_search),
        "shaped-cache eviction must remove auxiliary bucket slots through indexed positions"
    );
}

#[test]
fn shaped_run_cache_keeps_a_surviving_exact_lookup_candidate_after_eviction() {
    let text = "0123";
    let style = TextStyle::default();
    let source_range = TextRange {
        start: 0,
        end: text.len(),
    };
    let first_features = [OpenTypeFeature::new(*b"liga", 0)];
    let second_features = [OpenTypeFeature::new(*b"liga", 1)];
    let first_request =
        BackendShapeRequest::horizontal(text, &style, TextDirection::LeftToRight, source_range)
            .with_features(&first_features)
            .canonicalized()
            .expect("valid language and feature fixture");
    let second_request =
        BackendShapeRequest::horizontal(text, &style, TextDirection::LeftToRight, source_range)
            .with_features(&second_features)
            .canonicalized()
            .expect("valid language and feature fixture");
    let first_request = first_request.request();
    let second_request = second_request.request();
    let first_key = ShapedRunCacheKey::from_request(&first_request);
    let mut second_key = ShapedRunCacheKey::from_request(&second_request);
    second_key.features_hash = first_key.features_hash;
    let filler = BackendShapeRequest::horizontal(
        "filler",
        &style,
        TextDirection::LeftToRight,
        TextRange {
            start: 0,
            end: "filler".len(),
        },
    );
    let filler_key = ShapedRunCacheKey::from_request(&filler);
    let mut cache = ShapedRunCache::with_capacity(2);

    cache.insert(
        first_key,
        cached_run(&first_request, TextDirection::LeftToRight),
    );
    cache.insert(
        second_key.clone(),
        cached_run(&second_request, TextDirection::LeftToRight),
    );
    cache.insert(filler_key, cached_run(&filler, TextDirection::LeftToRight));

    assert!(cache.get(&second_key, text).is_some());
}

#[test]
fn shaped_run_cache_estimate_accounts_for_capacity_key_and_index_residency() {
    let text = "resident estimate";
    let style = TextStyle {
        font_family: Some("Layout Variable Family".to_string()),
        language: Some("sr-Latn".to_string()),
        ..TextStyle::default()
    };
    let features = [OpenTypeFeature::new(*b"liga", 0)];
    let request = BackendShapeRequest::horizontal(
        text,
        &style,
        TextDirection::LeftToRight,
        TextRange {
            start: 0,
            end: text.len(),
        },
    )
    .with_features(&features)
    .canonicalized()
    .expect("valid language and feature fixture");
    let request = request.request();
    let key = ShapedRunCacheKey::from_request(&request);
    let mut glyphs = Vec::with_capacity(16);
    glyphs.push(crate::text::ShapedGlyph {
        glyph_id: 1,
        font_id: None,
        font_instance_id: None,
        source_range: TextRange {
            start: 0,
            end: text.len(),
        },
        visual_range: TextRange {
            start: 0,
            end: text.len(),
        },
        advance: 1.0,
        x: 0.0,
        y: 0.0,
        offset_x: 0.0,
        offset_y: 0.0,
        direction: TextDirection::LeftToRight,
        bidi_level: 0,
        cluster_flags: crate::text::ShapedGlyphClusterFlags::default(),
        rotation: crate::text::ShapedGlyphRotation::None,
        script: crate::text::ShapedGlyphScript::default(),
    });
    let mut lines = Vec::with_capacity(8);
    lines.push(ShapedHardLine {
        line_index: 0,
        source_range: request.source_range,
        visual_range: request.source_range,
        measured_width: 1.0,
        baseline: 1.0,
        line_height: 1.0,
        glyphs,
    });
    let mut horizontal_line_raw_metrics = Vec::with_capacity(8);
    horizontal_line_raw_metrics.push(None);
    let mut horizontal_glyph_metric_spans = Vec::with_capacity(8);
    horizontal_glyph_metric_spans.push(HorizontalGlyphMetricSpan {
        line_index: 0,
        glyph_start: 0,
        glyph_end: 1,
        metrics: crate::text::HorizontalLineRawMetrics::new(1.0, 0.0, 0.0)
            .expect("finite metric span fixture"),
    });
    let mut run = ShapedGlyphRun {
        source_text: Arc::from(text),
        source_range: request.source_range,
        unicode_data_snapshot: request.unicode_data_snapshot(),
        primary_face_id: None,
        direction: request.base_direction,
        orientation: request.orientation,
        vertical_mode: request.vertical_mode,
        include_kerning: request.include_kerning,
        measured_width: 1.0,
        measured_height: 1.0,
        horizontal_composition_receipt: None,
        horizontal_line_raw_metrics,
        horizontal_glyph_metric_spans,
        lines,
    };
    let capacity_payload_floor = run
        .lines
        .capacity()
        .saturating_mul(size_of::<ShapedHardLine>())
        .saturating_add(
            run.lines[0]
                .glyphs
                .capacity()
                .saturating_mul(size_of::<ShapedGlyph>()),
        )
        .saturating_add(
            run.horizontal_line_raw_metrics
                .capacity()
                .saturating_mul(size_of::<Option<crate::text::HorizontalLineRawMetrics>>()),
        )
        .saturating_add(
            run.horizontal_glyph_metric_spans
                .capacity()
                .saturating_mul(size_of::<HorizontalGlyphMetricSpan> > ()),
        )
        .saturating_add(key.font_family.as_ref().map_or(0, String::capacity))
        .saturating_add(key.language.as_ref().map_or(0, String::capacity))
        .saturating_add(
            key.features
                .len()
                .saturating_mul(size_of::<OpenTypeFeature>()),
        );

    let without_composition_receipt = estimated_entry_bytes(&key, &run);
    assert!(without_composition_receipt > capacity_payload_floor);

    let mut alternate_ranges = Vec::with_capacity(8);
    alternate_ranges.push(TextRange { start: 0, end: 1 });
    run.horizontal_composition_receipt = Some(Box::new(TextHorizontalCompositionReceipt {
        alternate_ranges,
        first_failure: TextShapingFailureReceipt {
            code: TextShapingFailureCode::BackendFaceParse,
            phase: TextShapingFailurePhase::FontLoad,
            source_range: Some(TextRange { start: 0, end: 1 }),
            face: None,
            dependency: TextShapingFailureDependency::FontFace,
            disposition: TextShapingFailureDisposition::AlternateBackend,
            budget: None,
        },
    }));
    let with_composition_receipt = estimated_entry_bytes(&key, &run);
    let receipt_residency =
        size_of::<TextHorizontalCompositionReceipt>() + 8 * size_of::<TextRange>();

    assert!(
        with_composition_receipt >= without_composition_receipt + receipt_residency,
        "hybrid provenance must participate in shaped-cache byte admission"
    );
}

#[test]
fn shaped_run_cache_preserves_forced_direction_and_vertical_mode_boundaries() {
    let text = "vertical";
    let style = TextStyle::default();
    let source_range = TextRange {
        start: 0,
        end: text.len(),
    };
    let mixed = BackendShapeRequest::vertical(
        text,
        &style,
        TextDirection::Mixed,
        source_range,
        VerticalMode::Upright,
    );
    let resolved = BackendShapeRequest::vertical(
        text,
        &style,
        TextDirection::LeftToRight,
        source_range,
        VerticalMode::Upright,
    );
    let forced_rtl = BackendShapeRequest::vertical(
        text,
        &style,
        TextDirection::RightToLeft,
        source_range,
        VerticalMode::Upright,
    );
    let different_mode = BackendShapeRequest::vertical(
        text,
        &style,
        TextDirection::LeftToRight,
        source_range,
        VerticalMode::Sideways,
    );
    let mut cache = ShapedRunCache::with_capacity(4);

    cache.insert(
        ShapedRunCacheKey::from_request(&mixed),
        cached_run(&mixed, TextDirection::LeftToRight),
    );
    assert!(
        cache
            .get(&ShapedRunCacheKey::from_request(&resolved), text)
            .is_some()
    );
    assert!(
        cache
            .get(&ShapedRunCacheKey::from_request(&forced_rtl), text)
            .is_none()
    );
    assert!(
        cache
            .get(&ShapedRunCacheKey::from_request(&different_mode), text)
            .is_none()
    );
}

#[test]
fn shaped_run_cache_exact_match_rejects_a_feature_hash_collision() {
    let text = "0123";
    let style = TextStyle::default();
    let source_range = TextRange {
        start: 0,
        end: text.len(),
    };
    let first_features = [OpenTypeFeature::new(*b"liga", 0)];
    let changed_features = [OpenTypeFeature::new(*b"liga", 1)];
    let first_request =
        BackendShapeRequest::horizontal(text, &style, TextDirection::LeftToRight, source_range)
            .with_features(&first_features)
            .canonicalized()
            .expect("valid language and feature fixture");
    let changed_request =
        BackendShapeRequest::horizontal(text, &style, TextDirection::LeftToRight, source_range)
            .with_features(&changed_features)
            .canonicalized()
            .expect("valid language and feature fixture");
    let first_request = first_request.request();
    let changed_request = changed_request.request();
    let key = ShapedRunCacheKey::from_request(&first_request);
    let mut collided_lookup = ShapedRunCacheLookupKey::from_request(&changed_request);
    collided_lookup.features_hash = key.features_hash;

    assert!(!key.matches_lookup(&collided_lookup));
}

fn cached_run(request: &BackendShapeRequest<'_>, direction: TextDirection) -> ShapedGlyphRun {
    ShapedGlyphRun {
        source_text: Arc::from(request.text),
        source_range: request.source_range,
        unicode_data_snapshot: request.unicode_data_snapshot(),
        primary_face_id: None,
        direction,
        orientation: request.orientation,
        vertical_mode: request.vertical_mode,
        include_kerning: request.include_kerning,
        measured_width: 1.0,
        measured_height: 1.0,
        horizontal_composition_receipt: None,
        horizontal_line_raw_metrics: Vec::new(),
        horizontal_glyph_metric_spans: Vec::new(),
        lines: Vec::new(),
    }
}
