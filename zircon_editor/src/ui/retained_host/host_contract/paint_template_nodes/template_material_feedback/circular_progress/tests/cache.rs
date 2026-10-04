use std::sync::Arc;

use super::{
    CircularProgressRasterCache, CircularProgressRasterKey,
    MAX_CIRCULAR_PROGRESS_RASTER_CACHE_ENTRIES,
};

fn key(index: u32) -> CircularProgressRasterKey {
    CircularProgressRasterKey::new(index, 0.5, [1; 4], [2; 4])
}

fn pixels(index: u32) -> Arc<[u8]> {
    vec![index as u8].into()
}

#[test]
fn raster_cache_evicts_the_least_recently_used_variant() {
    let mut cache = CircularProgressRasterCache::new();
    for index in 0..MAX_CIRCULAR_PROGRESS_RASTER_CACHE_ENTRIES {
        cache.insert(
            key(index as u32),
            format!("progress-{index}"),
            pixels(index as u32),
        );
    }
    assert!(cache.get(key(0)).is_some());

    cache.insert(key(u32::MAX), "progress-new".to_string(), pixels(u32::MAX));

    assert_eq!(
        cache.entries.len(),
        MAX_CIRCULAR_PROGRESS_RASTER_CACHE_ENTRIES
    );
    assert!(cache.get(key(0)).is_some());
    assert!(!cache.entries.contains_key(&key(1)));
    assert!(cache.get(key(u32::MAX)).is_some());
}
