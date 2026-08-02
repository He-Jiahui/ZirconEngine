use crate::text::{CompositeFontDescriptor, FontFamilyName, FontQuery};

use super::{BoundedCache, composite_font_identity, fallback_query_identity};

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

    assert_eq!(
        fallback_query_identity(&query, None, Some("zh-CN")),
        fallback_query_identity(&query, None, Some("zh-CN")),
    );
    assert_eq!(
        fallback_query_identity(&query, Some(composite), Some("zh-CN")),
        fallback_query_identity(&query, Some(composite), Some("zh-CN")),
    );
}

#[test]
fn text_font_fallback_query_identity_distinguishes_absent_composite() {
    let (query, composite) = query_and_composite();
    let composite = composite_font_identity(&composite);

    assert_ne!(
        fallback_query_identity(&query, None, Some("zh-CN")),
        fallback_query_identity(&query, Some(composite), Some("zh-CN")),
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
