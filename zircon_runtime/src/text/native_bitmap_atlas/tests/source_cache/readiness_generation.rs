use super::*;

#[test]
fn source_cache_readiness_generation_tracks_effective_image_and_binding_changes() {
    let mut cache = NativeBitmapAtlasSourceCache::with_capacity(2);
    let cache_key = test_cache_key(301);
    let persistent_key = GlyphRasterKey {
        format: GlyphAtlasFormat::Color,
        smoothing: GlyphSmoothingMode::None,
        ..cache_key
    };
    let initial = cache.readiness_generation();

    cache.begin_frame();
    assert_eq!(cache.readiness_generation(), initial);

    cache.insert_test_image(cache_key, test_cached_image(31));
    let inserted = cache.readiness_generation();
    assert!(inserted > initial);

    assert!(cache.cached_test_image(cache_key).is_some());
    assert_eq!(cache.readiness_generation(), inserted);

    assert!(cache.bind_persistent_raster_key(cache_key, persistent_key));
    let bound = cache.readiness_generation();
    assert!(bound > inserted);

    assert!(cache.bind_persistent_raster_key(cache_key, persistent_key));
    assert_eq!(cache.readiness_generation(), bound);

    cache.invalidate_raster_keys([persistent_key]);
    assert!(cache.readiness_generation() > bound);
}

#[test]
fn source_cache_readiness_generation_advances_for_eviction_and_face_invalidation() {
    let mut cache = NativeBitmapAtlasSourceCache::with_capacity(1);
    let first = test_cache_key(302);
    let second = test_cache_key(303);

    cache.insert_test_image(first, test_cached_image(32));
    let first_generation = cache.readiness_generation();
    cache.insert_test_image(second, test_cached_image(33));
    let eviction_generation = cache.readiness_generation();
    assert!(eviction_generation > first_generation);

    cache.discard_all_for_face_invalidation();
    let face_generation = cache.readiness_generation();
    assert!(face_generation > eviction_generation);

    cache.discard_all_for_face_invalidation();
    assert!(cache.readiness_generation() > face_generation);
}

#[test]
fn source_cache_readiness_receipt_expands_vertical_approximation_dependencies() {
    let mut cache = NativeBitmapAtlasSourceCache::with_capacity(2);
    let cache_key = GlyphRasterKey {
        vertical_subpixel_bin: 2,
        ..test_cache_key(304)
    };

    cache.insert_test_image(cache_key, test_cached_image(34));
    let receipt = cache.take_readiness_changes();
    let changed_keys = receipt.changed_keys();

    assert_eq!(receipt.generation(), cache.readiness_generation());
    assert!(!receipt.full_invalidation());
    assert_eq!(changed_keys.len(), 4);
    for vertical_subpixel_bin in 0..=3 {
        assert!(changed_keys.contains(&GlyphRasterKey {
            vertical_subpixel_bin,
            ..cache_key
        }));
    }

    assert!(cache.cached_test_image(cache_key).is_some());
    let touch_receipt = cache.take_readiness_changes();
    assert_eq!(touch_receipt.generation(), receipt.generation());
    assert!(!touch_receipt.full_invalidation());
    assert!(touch_receipt.changed_keys().is_empty());
}

#[test]
fn source_cache_readiness_receipt_reports_evicted_and_inserted_dependencies() {
    let mut cache = NativeBitmapAtlasSourceCache::with_capacity(1);
    let first = test_cache_key(305);
    let second = test_cache_key(306);

    cache.insert_test_image(first, test_cached_image(35));
    let _ = cache.take_readiness_changes();
    cache.insert_test_image(second, test_cached_image(36));
    let receipt = cache.take_readiness_changes();
    let changed_keys = receipt.changed_keys();

    assert!(!receipt.full_invalidation());
    for cache_key in [first, second] {
        for vertical_subpixel_bin in 0..=3 {
            assert!(changed_keys.contains(&GlyphRasterKey {
                vertical_subpixel_bin,
                ..cache_key
            }));
        }
    }
}

#[test]
fn source_cache_readiness_receipt_marks_face_invalidation_as_full() {
    let mut cache = NativeBitmapAtlasSourceCache::with_capacity(1);
    cache.insert_test_image(test_cache_key(307), test_cached_image(37));
    let _ = cache.take_readiness_changes();

    cache.discard_all_for_face_invalidation();
    let receipt = cache.take_readiness_changes();

    assert!(receipt.full_invalidation());
    assert_eq!(receipt.generation(), cache.readiness_generation());
    assert!(receipt.changed_keys().is_empty());
}
