use super::MeasureKey;
use crate::text::cache::{TextLayoutCache, TextLayoutWidthValidity, TextMeasureCache};

#[test]
fn text_measure_cache_reports_resident_bytes_across_update_eviction_and_clear() {
    let width = 80.0;
    let key_a = MeasureKey::new(71, width);
    let key_b = MeasureKey::new(72, width);
    let key_c = MeasureKey::new(73, width);
    let mut cache = TextMeasureCache::with_capacity(2);
    cache.begin_frame(1);

    cache.insert_with_additional_heap_bytes(key_a.clone(), "alpha", 1_u32, 11);
    let one_entry_bytes = cache.report().estimated_bytes;
    assert!(one_entry_bytes >= "alpha".len() + 11);

    cache.insert_with_additional_heap_bytes(key_a.clone(), "alpha", 2_u32, 29);
    let updated_entry_bytes = one_entry_bytes.saturating_sub(11).saturating_add(29);
    assert_eq!(cache.report().estimated_bytes, updated_entry_bytes);

    cache.insert_with_additional_heap_bytes(key_b, "beta", 3_u32, 7);
    let retained_entry_bytes = cache
        .report()
        .estimated_bytes
        .saturating_sub(updated_entry_bytes);
    cache.insert_with_additional_heap_bytes(key_c, "gamma", 4_u32, 5);
    assert!(!cache.contains_exact(&key_a, "alpha"));
    assert_eq!(cache.len(), 2);
    assert_eq!(
        cache.report().estimated_bytes,
        retained_entry_bytes
            .saturating_add(updated_entry_bytes.saturating_sub(29).saturating_add(5))
    );
    assert!(cache.report().peak_estimated_bytes >= cache.report().estimated_bytes);

    cache.clear();
    assert_eq!(cache.report().estimated_bytes, 0);
}

#[test]
fn text_layout_cache_reports_resident_bytes_across_update_eviction_and_clear() {
    let width = TextLayoutWidthValidity::exact(80.0);
    let key_a = MeasureKey::new(81, 80.0);
    let key_b = MeasureKey::new(82, 80.0);
    let key_c = MeasureKey::new(83, 80.0);
    let mut cache = TextLayoutCache::with_capacity(2);
    cache.begin_frame(1);

    cache.insert_with_additional_heap_bytes(key_a.clone(), "alpha", width, 1_u32, 13);
    let one_entry_bytes = cache.report().estimated_bytes;
    assert!(one_entry_bytes >= "alpha".len() + 13);

    cache.insert_with_additional_heap_bytes(key_a.clone(), "alpha", width, 2_u32, 31);
    let updated_entry_bytes = one_entry_bytes.saturating_sub(13).saturating_add(31);
    assert_eq!(cache.report().estimated_bytes, updated_entry_bytes);

    cache.insert_with_additional_heap_bytes(key_b, "beta", width, 3_u32, 9);
    let retained_entry_bytes = cache
        .report()
        .estimated_bytes
        .saturating_sub(updated_entry_bytes);
    cache.insert_with_additional_heap_bytes(key_c, "gamma", width, 4_u32, 5);
    assert!(!cache.contains_exact(&key_a, "alpha", width));
    assert_eq!(cache.len(), 2);
    assert_eq!(
        cache.report().estimated_bytes,
        retained_entry_bytes
            .saturating_add(updated_entry_bytes.saturating_sub(31).saturating_add(5))
    );
    assert!(cache.report().peak_estimated_bytes >= cache.report().estimated_bytes);

    cache.clear();
    assert_eq!(cache.report().estimated_bytes, 0);
}
