use crate::text::language::TextLanguageFallbackKey;
use crate::text::{CompositeFontDescriptor, FontFamilyName, FontQuery};

use super::{
    BoundedCache, FallbackCaches, FallbackCandidateCacheKey, composite_font_identity,
    fallback_query_identity, fallback_query_identity_for_asset,
};

fn query_and_composite() -> (FontQuery, CompositeFontDescriptor) {
    (
        FontQuery::single_family("Primary"),
        CompositeFontDescriptor {
            default_family: FontFamilyName::from("Primary"),
            sub_fonts: Vec::new(),
        },
    )
}

#[test]
fn text_font_fallback_query_identity_is_stable_for_composite_presence() {
    let (query, composite) = query_and_composite();
    let composite = composite_font_identity(&composite);
    let language = TextLanguageFallbackKey::from_language(Some("zh-CN"));

    assert_eq!(
        fallback_query_identity(&query, None, language),
        fallback_query_identity(&query, None, language),
    );
    assert_eq!(
        fallback_query_identity(&query, Some(composite), language),
        fallback_query_identity(&query, Some(composite), language),
    );
}

#[test]
fn text_font_fallback_query_identity_distinguishes_absent_composite() {
    let (query, composite) = query_and_composite();
    let composite = composite_font_identity(&composite);
    let language = TextLanguageFallbackKey::from_language(Some("zh-CN"));

    assert_ne!(
        fallback_query_identity(&query, None, language),
        fallback_query_identity(&query, Some(composite), language),
    );
}

#[test]
fn text_font_fallback_query_identity_distinguishes_font_asset_owners() {
    let (query, _) = query_and_composite();
    let language = TextLanguageFallbackKey::from_language(Some("zh-CN"));

    assert_ne!(
        fallback_query_identity_for_asset(&query, None, language, "res://fonts/first.font.toml",),
        fallback_query_identity_for_asset(&query, None, language, "res://fonts/second.font.toml",),
    );
    assert_ne!(
        fallback_query_identity(&query, None, language),
        fallback_query_identity_for_asset(&query, None, language, "res://fonts/first.font.toml",),
    );
}

#[test]
fn text_font_fallback_query_identity_tracks_only_candidate_affecting_locale_components() {
    let (query, _) = query_and_composite();
    let canonical = TextLanguageFallbackKey::from_language(Some("zh-Hans-CN"));
    let alternate_spelling = TextLanguageFallbackKey::from_language(Some(" ZH_hans_cn "));
    let calendar_extension =
        TextLanguageFallbackKey::from_language(Some("zh-Hans-CN-u-ca-chinese"));
    let alternate_script = TextLanguageFallbackKey::from_language(Some("zh-Hant-CN"));
    let alternate_region = TextLanguageFallbackKey::from_language(Some("zh-Hans-TW"));

    assert_eq!(
        fallback_query_identity(&query, None, canonical),
        fallback_query_identity(&query, None, alternate_spelling),
    );
    assert_eq!(
        fallback_query_identity(&query, None, canonical),
        fallback_query_identity(&query, None, calendar_extension),
    );
    assert_ne!(
        fallback_query_identity(&query, None, canonical),
        fallback_query_identity(&query, None, alternate_script),
    );
    assert_ne!(
        fallback_query_identity(&query, None, canonical),
        fallback_query_identity(&query, None, alternate_region),
    );
}

#[test]
fn text_font_bounded_cache_uses_indexed_lru_eviction() {
    let source = include_str!("../fallback_cache.rs");
    assert!(
        source.contains("lru: BTreeMap<u64, K>"),
        "bounded fallback caches need an indexed LRU order instead of scanning every entry"
    );
    assert!(
        source.contains("self.lru.pop_first()"),
        "bounded fallback caches must evict directly from their indexed LRU order"
    );
    assert!(
        !source.contains(".min_by_key("),
        "bounded fallback caches must not linearly scan entries to find an eviction candidate"
    );

    let mut cache = BoundedCache::new(2, 2);
    cache.insert(1_u8, "first", 1);
    cache.insert(2_u8, "second", 1);
    assert_eq!(cache.get(1), Some("first"));

    cache.insert(3_u8, "third", 1);

    assert_eq!(cache.get(1), Some("first"));
    assert_eq!(cache.get(2), None);
    assert_eq!(cache.get(3), Some("third"));
}

#[test]
fn text_font_bounded_cache_rebases_lru_ticks_without_losing_order() {
    let source = include_str!("../fallback_cache.rs");
    let wrapped_increment = ["self.tick", "wrapping_add(1)"].join(".");
    assert!(
        source.contains("fn rebase_lru_ticks(&mut self)"),
        "the LRU clock must be rebased before its monotonic tick can overflow"
    );
    assert!(
        !source.contains(&wrapped_increment),
        "wrapping the LRU clock would make a recent entry look oldest"
    );

    let mut cache = BoundedCache::new(2, 2);
    cache.insert(1_u8, "first", 1);
    cache.insert(2_u8, "second", 1);
    cache.tick = u64::MAX;

    assert_eq!(cache.get(1), Some("first"));
    cache.insert(3_u8, "third", 1);

    assert_eq!(cache.get(1), Some("first"));
    assert_eq!(cache.get(2), None);
    assert_eq!(cache.get(3), Some("third"));
}

#[test]
fn text_font_fallback_cache_measures_state_lock_without_counting_report_reads() {
    let caches = FallbackCaches::default();
    let key = FallbackCandidateCacheKey([7; 16]);
    let before = caches.report();

    caches.begin_profile_request();
    assert!(caches.candidates(key).is_none());
    let request = caches
        .take_profile_request()
        .expect("an active request must retain its exact lock cost");
    let after_access = caches.report();
    let after_report = caches.report();

    assert_eq!(request.state_lock_acquire_count, 1);
    assert_eq!(
        after_access.state_lock_acquire_count,
        before.state_lock_acquire_count.saturating_add(1),
        "one candidate-cache read must expose exactly one shared-state lock acquisition"
    );
    assert_eq!(
        after_report.state_lock_acquire_count, after_access.state_lock_acquire_count,
        "reading cache diagnostics must not recursively contribute to measured cache work"
    );
}

#[test]
fn text_font_fallback_cache_profile_uses_fixed_lock_cost_names() {
    let source = include_str!("../../shaping/cosmic/fallback_profile.rs");
    for name in [
        "text_font_fallback_cache_state_lock_acquire_count",
        "text_font_fallback_cache_state_lock_wait_nanos",
        "text_font_fallback_cache_state_lock_hold_nanos",
    ] {
        assert!(
            source.contains(name),
            "missing fixed profiler counter: {name}"
        );
    }
    assert!(
        !source.contains("face_id")
            && !source.contains("codepoint")
            && !source.contains("family_name"),
        "fallback cache lock profiling must not introduce data-dependent label dimensions"
    );
}
