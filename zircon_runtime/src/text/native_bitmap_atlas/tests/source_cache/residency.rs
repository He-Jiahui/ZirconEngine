use super::*;

use crate::text::atlas::{
    GlyphHintingMode, GlyphRasterKey, GlyphSmoothingMode, SyntheticGlyphStyle,
};
use crate::text::InstancedFaceId;

#[test]
fn native_bitmap_atlas_idle_frame_keeps_cached_source_images_resident() {
    let mut cache = NativeBitmapAtlasSourceCache::with_capacity(4);
    let first = test_cache_key(11);
    let second = test_cache_key(12);

    cache.begin_frame();
    cache.insert_test_image(first, test_cached_image(11));
    cache.insert_test_image(second, test_cached_image(12));

    let mut retry_state = GlyphAtlasBitmapRetryFrameState::new();
    let mut report = NativeBitmapAtlasPrepareReport::default();
    for _ in 0..300 {
        report = native_bitmap_atlas_idle_prepare_report(&mut cache, &mut retry_state);
        assert_eq!(report.source_cache.entry_count, 2);
        assert_eq!(report.source_cache.pending_worker_count, 0);
        assert_eq!(report.source_cache.worker_request_submitted_count, 0);
        assert_eq!(report.source_cache.worker_completion_unknown_count, 0);
        assert_eq!(report.submission.slot_cache_hit_count, 0);
        assert_eq!(report.submission.slot_cache_miss_count, 0);
        assert_eq!(report.submission.upload_copy_count, 0);
    }

    assert_eq!(
        report.source_cache,
        NativeBitmapAtlasSourceCacheFrameReport {
            capacity: 4,
            max_byte_count: TEST_DEFAULT_SOURCE_CACHE_MAX_BYTES,
            resident_byte_count: 8,
            entry_count: 2,
            ..NativeBitmapAtlasSourceCacheFrameReport::default()
        }
    );
    assert_eq!(report.source_image_count, 0);
    assert_eq!(report.visible_raster_glyph_count, 0);
    assert_eq!(report.retry_state.queued_blocked_glyph_count, 0);
    assert_eq!(report.retry_state.invalidated_blocked_glyph_count, 0);
    assert_eq!(report.submission.visible_glyph_count, 0);
    assert_eq!(
        cache.cached_test_image(first).unwrap().bytes.as_ref(),
        &[11; 4]
    );
    assert_eq!(
        cache.cached_test_image(second).unwrap().bytes.as_ref(),
        &[12; 4]
    );
}

#[test]
fn native_bitmap_atlas_budget_pressure_invalidates_linked_source_slot_and_page() {
    let first_cache_key = test_cache_key(81);
    let second_cache_key = test_cache_key(82);
    let third_cache_key = test_cache_key(83);
    let first_raster_key = test_raster_key(81);
    let second_raster_key = test_raster_key(82);
    let page_size = UVec2::new(32, 32);
    let mut cache = NativeBitmapAtlasSourceCache::with_capacity(2);
    let mut atlas = GlyphAtlasSet::default();

    cache.begin_frame();
    cache.insert_test_image(first_cache_key, test_cached_image(81));
    cache.insert_test_image(second_cache_key, test_cached_image(82));
    assert!(cache.bind_persistent_raster_key(first_cache_key, first_raster_key));
    assert!(cache.bind_persistent_raster_key(second_cache_key, second_raster_key));

    let first_slot = atlas
        .allocate_persistent_bitmap_slot(first_raster_key, UVec2::new(8, 8), page_size, 1, 1, 2)
        .expect("first persistent slot")
        .0;
    let second_slot = atlas
        .allocate_persistent_bitmap_slot(second_raster_key, UVec2::new(8, 8), page_size, 1, 1, 2)
        .expect("second persistent slot")
        .0;
    assert_eq!(first_slot.page_key, second_slot.page_key);

    cache.insert_test_image(third_cache_key, test_cached_image(83));
    let budget_evicted = cache.take_budget_evicted_raster_keys();
    assert_eq!(budget_evicted, vec![first_raster_key]);

    let mut linked_invalidations = atlas.invalidate_bitmap_raster_keys(budget_evicted);
    linked_invalidations.sort_by_key(|key| key.glyph_id);
    assert_eq!(
        linked_invalidations,
        vec![first_raster_key, second_raster_key]
    );
    cache.invalidate_raster_keys(linked_invalidations);

    assert!(cache.cached_test_image(first_cache_key).is_none());
    assert!(cache.cached_test_image(second_cache_key).is_none());
    assert!(cache.cached_test_image(third_cache_key).is_some());
    assert!(atlas
        .persistent_bitmap_slot(first_raster_key, UVec2::new(8, 8), page_size, 2)
        .is_none());
    assert!(atlas
        .persistent_bitmap_slot(second_raster_key, UVec2::new(8, 8), page_size, 2)
        .is_none());
    assert_eq!(
        atlas
            .page(first_slot.page_key.format, first_slot.page_key.page_index)
            .expect("invalidated page remains reusable")
            .generation,
        first_slot.page_generation + 1
    );
    let report = cache.frame_report();
    assert_eq!(report.budget_linked_eviction_count, 1);
    assert_eq!(report.linked_raster_invalidation_count, 1);
}

#[test]
fn native_bitmap_atlas_upload_failure_reports_linked_invalidation_on_next_frame() {
    let cache_key = test_cache_key(84);
    let raster_key = test_raster_key(84);
    let mut cache = NativeBitmapAtlasSourceCache::with_capacity(2);

    cache.begin_frame();
    cache.insert_test_image(cache_key, test_cached_image(84));
    assert!(cache.bind_persistent_raster_key(cache_key, raster_key));
    cache.invalidate_raster_keys_for_next_frame([raster_key]);

    assert!(cache.cached_test_image(cache_key).is_none());
    cache.begin_frame();
    assert_eq!(cache.frame_report().linked_raster_invalidation_count, 1);
    assert_eq!(cache.frame_report().entry_count, 0);
}

#[test]
fn native_bitmap_atlas_source_cache_scale_probes_are_bounded_at_full_capacity() {
    const RESIDENT_ENTRY_COUNT: u16 = 2048;
    for new_glyph_count in [1_u16, 100, 1_000] {
        let mut cache =
            NativeBitmapAtlasSourceCache::with_capacity(usize::from(RESIDENT_ENTRY_COUNT));
        for glyph_id in 1..=RESIDENT_ENTRY_COUNT {
            cache.insert_test_image(test_cache_key(glyph_id), test_cached_image(1));
        }

        cache.begin_frame();
        for glyph_offset in 1..=new_glyph_count {
            assert!(cache
                .approximate_cached_image(test_cache_key(
                    RESIDENT_ENTRY_COUNT.saturating_add(glyph_offset),
                ))
                .is_none());
        }

        let report = cache.frame_report();
        assert_eq!(
            report.approximate_probe_count,
            usize::from(new_glyph_count) * 3
        );
        assert_eq!(report.lru_touch_count, 0);
        assert_eq!(report.evicted_count, 0);
    }
}

#[test]
#[ignore = "manual 31-sample source-cache scale evidence; no machine-time acceptance threshold"]
fn native_bitmap_atlas_source_cache_reports_scale_p50_p95() {
    const RESIDENT_ENTRY_COUNT: u16 = 2048;
    for new_glyph_count in [1_u16, 100, 1_000] {
        let mut cache =
            NativeBitmapAtlasSourceCache::with_capacity(usize::from(RESIDENT_ENTRY_COUNT));
        for glyph_id in 1..=RESIDENT_ENTRY_COUNT {
            cache.insert_test_image(test_cache_key(glyph_id), test_cached_image(1));
        }
        let mut samples_ns = Vec::with_capacity(31);
        for _ in 0..31 {
            cache.begin_frame();
            let started = std::time::Instant::now();
            for glyph_offset in 1..=new_glyph_count {
                let _ = cache.approximate_cached_image(test_cache_key(
                    RESIDENT_ENTRY_COUNT.saturating_add(glyph_offset),
                ));
            }
            samples_ns.push(started.elapsed().as_nanos());
            assert_eq!(
                cache.frame_report().approximate_probe_count,
                usize::from(new_glyph_count) * 3
            );
        }
        samples_ns.sort_unstable();
        let p50_ns = samples_ns[samples_ns.len() / 2];
        let p95_index = (samples_ns.len() * 95).div_ceil(100) - 1;
        let p95_ns = samples_ns[p95_index];

        println!(
            "resident=2048 new_glyphs={new_glyph_count} approximate_probes={} \
             lru_touches=0 evictions=0 p50_ns={p50_ns} p95_ns={p95_ns}",
            usize::from(new_glyph_count) * 3,
        );
    }
}

fn test_raster_key(glyph_id: u32) -> GlyphRasterKey {
    GlyphRasterKey {
        face: InstancedFaceId(17),
        glyph_id,
        px_size_bucket: 16,
        subpixel_bin: 0,
        vertical_subpixel_bin: 0,
        format: GlyphAtlasFormat::AlphaMask,
        hinting: GlyphHintingMode::Full,
        smoothing: GlyphSmoothingMode::Grayscale,
        synthetic: SyntheticGlyphStyle::default(),
    }
}
