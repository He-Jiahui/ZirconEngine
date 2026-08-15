use super::*;
use crate::text::{ShapedGlyph, ShapedTextLine};

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
            .canonicalized();
    let second_request =
        BackendShapeRequest::horizontal(text, &style, TextDirection::LeftToRight, source_range)
            .with_features(&second_features)
            .canonicalized();
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
    .canonicalized();
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
    lines.push(ShapedTextLine {
        line_index: 0,
        source_range: request.source_range,
        visual_range: request.source_range,
        measured_width: 1.0,
        baseline: 1.0,
        line_height: 1.0,
        glyphs,
    });
    let run = ShapedGlyphRun {
        source_text: Arc::from(text),
        source_range: request.source_range,
        direction: request.base_direction,
        orientation: request.orientation,
        vertical_mode: request.vertical_mode,
        include_kerning: request.include_kerning,
        measured_width: 1.0,
        measured_height: 1.0,
        lines,
    };
    let capacity_payload_floor = run
        .lines
        .capacity()
        .saturating_mul(size_of::<ShapedTextLine>())
        .saturating_add(
            run.lines[0]
                .glyphs
                .capacity()
                .saturating_mul(size_of::<ShapedGlyph>()),
        )
        .saturating_add(key.font_family.as_ref().map_or(0, String::capacity))
        .saturating_add(key.language.as_ref().map_or(0, String::capacity))
        .saturating_add(
            key.features
                .len()
                .saturating_mul(size_of::<OpenTypeFeature>()),
        );

    assert!(estimated_entry_bytes(&key, &run) > capacity_payload_floor);
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
    assert!(cache
        .get(&ShapedRunCacheKey::from_request(&resolved), text)
        .is_some());
    assert!(cache
        .get(&ShapedRunCacheKey::from_request(&forced_rtl), text)
        .is_none());
    assert!(cache
        .get(&ShapedRunCacheKey::from_request(&different_mode), text)
        .is_none());
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
            .canonicalized();
    let changed_request =
        BackendShapeRequest::horizontal(text, &style, TextDirection::LeftToRight, source_range)
            .with_features(&changed_features)
            .canonicalized();
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
        direction,
        orientation: request.orientation,
        vertical_mode: request.vertical_mode,
        include_kerning: request.include_kerning,
        measured_width: 1.0,
        measured_height: 1.0,
        lines: Vec::new(),
    }
}
