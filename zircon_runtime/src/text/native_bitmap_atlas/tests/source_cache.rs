use super::*;
use std::sync::Arc;

use crate::core::math::Vec2;
use crate::text::parallel::raster_pool::{
    TextRasterCompletionDrain, TextRasterCompletionDrainBudget, TextRasterWorkId,
    TextRasterWorkItem, TextRasterWorkResult, TextRasterWorkerPool, TextRasterWorkerPoolOptions,
};
use crate::text::raster::{GlyphBitmap, SwashRasterError, SwashRasterRequest};

const TEST_FONT_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/fonts/FiraSans-Regular.ttf"
));
const TEST_DEFAULT_SOURCE_CACHE_MAX_BYTES: usize = 8 * 1024 * 1024;

#[path = "source_cache/readiness_generation.rs"]
mod readiness_generation_tests;
#[path = "source_cache/residency.rs"]
mod residency_tests;
#[path = "source_cache/worker_requests.rs"]
mod worker_request_tests;

#[test]
fn native_bitmap_atlas_source_cache_uses_indexed_lru_eviction() {
    let source = include_str!("../source_cache.rs");
    let lru_source = include_str!("../source_cache/lru.rs");

    assert!(
        source.contains("mod lru;"),
        "source cache LRU ownership must stay in its leaf module"
    );
    assert!(
        lru_source.contains("head: Option<GlyphRasterKey>"),
        "source cache eviction must keep a direct link to its oldest live image"
    );
    assert!(
        lru_source.contains("detach_or_repair("),
        "source cache eviction must detach the oldest glyph without scanning all entries"
    );
    assert!(
        !lru_source.contains(".min_by_key("),
        "source cache must not linearly scan every cached glyph to evict one entry"
    );
    assert!(
        source.contains("cache_keys_by_raster_key"),
        "source/slot budget invalidation must use a direct raster-key reverse index"
    );
    assert!(
        !source.contains("entries.iter().find"),
        "linked raster invalidation must not scan all resident source images"
    );
    assert!(
        !lru_source.contains(".expect("),
        "recoverable source-cache LRU state must not panic in production"
    );
    assert!(
        source.contains("approximate_vertical_bin_candidates"),
        "approximate source reuse must derive its bounded candidate keys directly"
    );
}

#[test]
fn native_bitmap_atlas_source_cache_repairs_dangling_lru_tail_before_eviction() {
    let mut cache = NativeBitmapAtlasSourceCache::with_capacity(2);
    let first = test_cache_key(71);
    let second = test_cache_key(72);
    let third = test_cache_key(73);
    let missing = test_cache_key(74);

    cache.begin_frame();
    cache.insert_test_image(first, test_cached_image(71));
    cache.insert_test_image(second, test_cached_image(72));
    cache.corrupt_lru_tail_for_test(missing);

    assert_eq!(
        cache.cached_test_image(first).unwrap().bytes.as_ref(),
        &[71; 4]
    );
    cache.insert_test_image(third, test_cached_image(73));

    assert_eq!(
        cache.cached_test_image(first).unwrap().bytes.as_ref(),
        &[71; 4]
    );
    assert!(cache.cached_test_image(second).is_none());
    assert_eq!(
        cache.cached_test_image(third).unwrap().bytes.as_ref(),
        &[73; 4]
    );
    assert_eq!(cache.frame_report().lru_repair_count, 1);
}

#[test]
fn native_bitmap_atlas_source_cache_evicts_least_recently_used_source() {
    let mut cache = NativeBitmapAtlasSourceCache::with_capacity(2);
    let first = test_cache_key(1);
    let second = test_cache_key(2);
    let third = test_cache_key(3);

    cache.begin_frame();
    cache.insert_test_image(first, test_cached_image(1));
    cache.insert_test_image(second, test_cached_image(2));
    assert_eq!(
        cache.cached_test_image(first).unwrap().bytes.as_ref(),
        &[1; 4]
    );
    cache.insert_test_image(third, test_cached_image(3));

    assert!(cache.cached_test_image(second).is_none());
    assert_eq!(
        cache.cached_test_image(first).unwrap().bytes.as_ref(),
        &[1; 4]
    );
    assert_eq!(
        cache.cached_test_image(third).unwrap().bytes.as_ref(),
        &[3; 4]
    );
    assert_eq!(
        cache.frame_report(),
        NativeBitmapAtlasSourceCacheFrameReport {
            capacity: 2,
            max_byte_count: TEST_DEFAULT_SOURCE_CACHE_MAX_BYTES,
            resident_byte_count: 8,
            hit_count: 0,
            miss_count: 0,
            insert_count: 3,
            lru_touch_count: 3,
            evicted_count: 1,
            evicted_byte_count: 4,
            invalidated_count: 0,
            entry_count: 2,
            ..NativeBitmapAtlasSourceCacheFrameReport::default()
        }
    );
}

#[test]
fn native_bitmap_atlas_source_cache_shares_cached_pixels_across_hits() {
    let mut cache = NativeBitmapAtlasSourceCache::with_capacity(1);
    let key = test_cache_key(101);
    let bytes = Arc::<[u8]>::from(vec![101; 4]);
    cache.insert_test_image(
        key,
        super::super::source_cache::NativeBitmapAtlasCachedGlyphImage {
            content: SwashContent::Mask,
            top: 0,
            left: 0,
            width: 2,
            height: 2,
            bytes: Arc::clone(&bytes),
        },
    );

    let first_hit = cache
        .cached_test_image(key)
        .expect("cached glyph should be available");
    let second_hit = cache
        .cached_test_image(key)
        .expect("cached glyph should remain available");

    assert!(Arc::ptr_eq(&bytes, &first_hit.bytes));
    assert!(Arc::ptr_eq(&first_hit.bytes, &second_hit.bytes));
}

#[test]
fn native_bitmap_atlas_source_cache_keeps_lru_order_after_repeated_touches() {
    let mut cache = NativeBitmapAtlasSourceCache::with_capacity(2);
    let first = test_cache_key(4);
    let second = test_cache_key(5);
    let third = test_cache_key(6);

    cache.begin_frame();
    cache.insert_test_image(first, test_cached_image(4));
    cache.insert_test_image(second, test_cached_image(5));

    assert_eq!(
        cache.cached_test_image(first).unwrap().bytes.as_ref(),
        &[4; 4]
    );
    cache.insert_test_image(third, test_cached_image(6));

    assert_eq!(
        cache.cached_test_image(first).unwrap().bytes.as_ref(),
        &[4; 4]
    );
    assert!(cache.cached_test_image(second).is_none());
    assert_eq!(
        cache.cached_test_image(third).unwrap().bytes.as_ref(),
        &[6; 4]
    );
}

#[test]
fn native_bitmap_atlas_source_cache_evicts_lru_entries_to_hold_its_cpu_byte_cap() {
    let mut cache = NativeBitmapAtlasSourceCache::with_limits(4, 8);
    let first = test_cache_key(61);
    let second = test_cache_key(62);
    let third = test_cache_key(63);

    cache.begin_frame();
    cache.insert_test_image(first, test_cached_image(61));
    cache.insert_test_image(second, test_cached_image(62));
    assert!(cache.cached_test_image(first).is_some());
    cache.insert_test_image(third, test_cached_image(63));

    assert!(cache.cached_test_image(first).is_some());
    assert!(cache.cached_test_image(second).is_none());
    assert!(cache.cached_test_image(third).is_some());
    let report = cache.frame_report();
    assert_eq!(report.resident_byte_count, 8);
    assert_eq!(report.max_byte_count, 8);
    assert_eq!(report.evicted_byte_count, 4);
    assert_eq!(report.evicted_count, 1);
}

#[test]
fn native_bitmap_atlas_source_cache_rejects_an_image_larger_than_its_cpu_byte_cap() {
    let mut cache = NativeBitmapAtlasSourceCache::with_limits(4, 3);
    let key = test_cache_key(64);

    cache.begin_frame();
    cache.insert_test_image(key, test_cached_image(64));

    assert!(cache.cached_test_image(key).is_none());
    let report = cache.frame_report();
    assert_eq!(report.entry_count, 0);
    assert_eq!(report.resident_byte_count, 0);
    assert_eq!(report.rejected_byte_budget_count, 1);
}

#[test]
fn native_bitmap_atlas_source_cache_replacement_keeps_only_the_live_lru_tick() {
    let mut cache = NativeBitmapAtlasSourceCache::with_capacity(2);
    let first = test_cache_key(7);
    let second = test_cache_key(8);
    let third = test_cache_key(9);

    cache.begin_frame();
    cache.insert_test_image(first, test_cached_image(7));
    cache.insert_test_image(second, test_cached_image(8));
    cache.insert_test_image(first, test_cached_image(70));
    cache.insert_test_image(third, test_cached_image(9));

    assert_eq!(
        cache.cached_test_image(first).unwrap().bytes.as_ref(),
        &[70; 4]
    );
    assert!(cache.cached_test_image(second).is_none());
    assert_eq!(
        cache.cached_test_image(third).unwrap().bytes.as_ref(),
        &[9; 4]
    );
}

#[test]
fn native_bitmap_atlas_idle_frame_reports_face_invalidated_retry_state() {
    let mut cache = NativeBitmapAtlasSourceCache::with_capacity(4);
    let invalidated_source = GlyphAtlasBitmapSource {
        raster_key: None,
        format: GlyphAtlasFormat::AlphaMask,
        content_size: UVec2::new(8, 8),
        screen_rect: GlyphAtlasScreenRect::new(8.0, 4.0, 8.0, 8.0),
        foreground_color: [0.8, 0.8, 0.8, 1.0],
        background_color: [0.0, 0.0, 0.0, 1.0],
        source_byte_len: 64,
    };
    let mut retry_state = GlyphAtlasBitmapRetryFrameState::with_blocked_glyphs([
        crate::text::atlas::GlyphAtlasBitmapQueuedGlyph {
            source_index: 0,
            source: invalidated_source,
            retry_frame_index: TEST_BITMAP_ATLAS_FRAME_INDEX,
        },
    ]);
    retry_state.discard_all_for_face_invalidation();

    let report = native_bitmap_atlas_idle_prepare_report(&mut cache, &mut retry_state);

    assert_eq!(report.retry_state.queued_blocked_glyph_count, 0);
    assert_eq!(report.retry_state.invalidated_blocked_glyph_count, 1);
    assert_eq!(retry_state.report().invalidated_blocked_glyph_count, 0);
}

#[test]
fn native_bitmap_atlas_source_cache_reports_face_invalidation_on_next_frame() {
    let mut cache = NativeBitmapAtlasSourceCache::with_capacity(4);
    let first = test_cache_key(21);
    let second = test_cache_key(22);

    assert_eq!(cache.face_epoch(), 0);
    cache.begin_frame();
    cache.insert_test_image(first, test_cached_image(21));
    cache.insert_test_image(second, test_cached_image(22));

    cache.discard_all_for_face_invalidation();

    assert!(cache.cached_test_image(first).is_none());
    assert!(cache.cached_test_image(second).is_none());
    assert_eq!(cache.face_epoch(), 1);

    cache.begin_frame();

    assert_eq!(
        cache.frame_report(),
        NativeBitmapAtlasSourceCacheFrameReport {
            capacity: 4,
            max_byte_count: TEST_DEFAULT_SOURCE_CACHE_MAX_BYTES,
            evicted_count: 2,
            invalidated_count: 2,
            entry_count: 0,
            ..NativeBitmapAtlasSourceCacheFrameReport::default()
        }
    );
}

#[test]
fn native_bitmap_atlas_source_cache_advances_face_epoch_per_invalidation() {
    let mut cache = NativeBitmapAtlasSourceCache::with_capacity(1);

    assert_eq!(cache.face_epoch(), 0);

    cache.discard_all_for_face_invalidation();
    assert_eq!(cache.face_epoch(), 1);

    cache.discard_all_for_face_invalidation();
    assert_eq!(cache.face_epoch(), 2);
}

#[test]
fn native_bitmap_atlas_source_cache_cancels_pending_worker_work_on_face_invalidation() {
    let worker_pool = TextRasterWorkerPool::new_without_workers_for_test(
        TextRasterWorkerPoolOptions::new(1).with_queue_depth(1),
    );
    let mut cache = NativeBitmapAtlasSourceCache::with_capacity(4);
    let work_id = TextRasterWorkId::new(24);

    worker_pool
        .request(TextRasterWorkItem::new(
            work_id,
            cache.face_epoch(),
            Arc::<[u8]>::from(TEST_FONT_BYTES),
            SwashRasterRequest::alpha_outline(0, 1, 16.0, true),
        ))
        .expect("test worker queue should accept the pending request");
    cache.register_worker_request(work_id, test_cache_key(24));
    cache.discard_all_for_face_invalidation_with_worker_pool(Some(&worker_pool));

    cache.begin_frame();
    assert_eq!(cache.frame_report().worker_request_cancelled_count, 1);
    assert!(worker_pool.process_next_request_for_test());
    assert!(
        worker_pool
            .drain_completed_for_face_epoch(
                cache.face_epoch(),
                TextRasterCompletionDrainBudget::new(1, usize::MAX),
            )
            .accepted
            .is_empty()
    );
    assert_eq!(worker_pool.diagnostics().cancelled, 1);
}

#[test]
fn native_bitmap_atlas_source_cache_inserts_registered_worker_completion() {
    let mut cache = NativeBitmapAtlasSourceCache::with_capacity(4);
    let key = test_cache_key(31);
    let work_id = TextRasterWorkId::new(31);
    let bitmap =
        GlyphBitmap::subpixel_mask(UVec2::new(2, 1), Vec2::new(-1.0, 7.0), 16.0, vec![9; 8])
            .expect("test subpixel bitmap should be valid");

    cache.begin_frame();
    cache.register_worker_request(work_id, key);

    let report = cache.apply_worker_completion_drain(TextRasterCompletionDrain {
        accepted: vec![worker_result(work_id, Ok(bitmap))],
        drained_bytes: 8,
        byte_budget_deferred_count: 1,
        oversized_accepted_count: 1,
        ..TextRasterCompletionDrain::default()
    });

    assert_eq!(report.worker_completion_insert_count, 1);
    assert_eq!(report.worker_completion_applied_byte_count, 8);
    assert_eq!(report.worker_completion_drained_byte_count, 8);
    assert_eq!(report.worker_completion_byte_budget_deferred_count, 1);
    assert_eq!(report.worker_completion_oversized_accepted_count, 1);
    assert_eq!(report.insert_count, 1);
    assert_eq!(report.pending_worker_count, 0);

    let image = cache
        .cached_test_image(key)
        .expect("worker completion should populate source cache");
    assert_eq!(image.content, SwashContent::SubpixelMask);
    assert_eq!(image.left, -1);
    assert_eq!(image.top, 7);
    assert_eq!(image.width, 2);
    assert_eq!(image.height, 1);
    assert_eq!(image.bytes.as_ref(), &[9; 8]);
}

#[test]
fn native_bitmap_atlas_source_cache_does_not_count_rejected_completion_bytes_as_applied() {
    let mut cache = NativeBitmapAtlasSourceCache::with_limits(4, 4);
    let key = test_cache_key(32);
    let work_id = TextRasterWorkId::new(32);
    let bitmap =
        GlyphBitmap::subpixel_mask(UVec2::new(2, 1), Vec2::new(-1.0, 7.0), 16.0, vec![9; 8])
            .expect("test subpixel bitmap should be valid");

    cache.begin_frame();
    cache.register_worker_request(work_id, key);

    let report = cache.apply_worker_completion_drain(TextRasterCompletionDrain {
        accepted: vec![worker_result(work_id, Ok(bitmap))],
        drained_bytes: 8,
        ..TextRasterCompletionDrain::default()
    });

    assert_eq!(report.worker_completion_insert_count, 0);
    assert_eq!(report.worker_completion_applied_byte_count, 0);
    assert_eq!(report.worker_completion_drained_byte_count, 8);
    assert_eq!(report.rejected_byte_budget_count, 1);
    assert_eq!(report.pending_worker_count, 0);
    assert!(cache.cached_test_image(key).is_none());
}

#[test]
fn native_bitmap_atlas_source_cache_keeps_horizontal_subpixel_bucket_exact() {
    let mut cache = NativeBitmapAtlasSourceCache::with_capacity(4);
    let requested_key = test_cache_key(35);
    let mut same_pixel_key = requested_key;
    same_pixel_key.subpixel_bin = 2;

    cache.begin_frame();
    cache.insert_test_image(same_pixel_key, test_cached_image(35));

    assert!(cache.cached_image(requested_key).is_none());
    assert_eq!(
        cache.frame_report(),
        NativeBitmapAtlasSourceCacheFrameReport {
            capacity: 4,
            max_byte_count: TEST_DEFAULT_SOURCE_CACHE_MAX_BYTES,
            resident_byte_count: 4,
            hit_count: 0,
            approximate_hit_count: 0,
            lru_touch_count: 0,
            miss_count: 1,
            insert_count: 1,
            entry_count: 1,
            ..NativeBitmapAtlasSourceCacheFrameReport::default()
        }
    );
}

#[test]
fn native_bitmap_atlas_source_cache_reuses_neighboring_vertical_bucket_as_approximate_image() {
    let mut cache = NativeBitmapAtlasSourceCache::with_capacity(4);
    let requested_key = test_cache_key(36);
    let mut neighboring_key = requested_key;
    neighboring_key.vertical_subpixel_bin = 1;

    cache.begin_frame();
    cache.insert_test_image(neighboring_key, test_cached_image(36));

    assert!(cache.cached_image(requested_key).is_none());
    let image = cache
        .approximate_cached_image(requested_key)
        .expect("same glyph in a neighboring vertical bucket should remain reusable");

    assert_eq!(image.bytes.as_ref(), &[36; 4]);
    assert_eq!(
        cache.frame_report(),
        NativeBitmapAtlasSourceCacheFrameReport {
            capacity: 4,
            max_byte_count: TEST_DEFAULT_SOURCE_CACHE_MAX_BYTES,
            resident_byte_count: 4,
            hit_count: 1,
            approximate_hit_count: 1,
            approximate_probe_count: 1,
            lru_touch_count: 1,
            miss_count: 1,
            insert_count: 1,
            entry_count: 1,
            ..NativeBitmapAtlasSourceCacheFrameReport::default()
        }
    );
}

#[test]
fn native_bitmap_atlas_source_cache_bounds_approximate_probes_at_full_capacity() {
    const RESIDENT_ENTRY_COUNT: u16 = 2048;
    let mut cache = NativeBitmapAtlasSourceCache::with_capacity(usize::from(RESIDENT_ENTRY_COUNT));

    cache.begin_frame();
    for glyph_id in 1..=RESIDENT_ENTRY_COUNT {
        cache.insert_test_image(test_cache_key(glyph_id), test_cached_image(1));
    }

    assert!(
        cache
            .approximate_cached_image(test_cache_key(RESIDENT_ENTRY_COUNT + 1))
            .is_none()
    );
    assert_eq!(cache.frame_report().approximate_probe_count, 3);
}

#[test]
fn native_bitmap_atlas_source_cache_bounds_new_raster_requests_per_frame() {
    let (font_database, instance) = test_font_database_with_fira();
    let worker_pool = TextRasterWorkerPool::new_without_workers_for_test(
        TextRasterWorkerPoolOptions::new(1)
            .with_queue_depth(
                super::super::source_cache::NATIVE_BITMAP_ATLAS_MAX_RASTER_REQUESTS_PER_FRAME + 1,
            )
            .with_request_byte_budget(usize::MAX),
    );
    let mut cache = NativeBitmapAtlasSourceCache::with_capacity(4);
    let face_epoch = cache.face_epoch();
    let cache_key = |glyph_id| GlyphRasterKey {
        face: instance,
        glyph_id,
        ..test_cache_key(glyph_id as u16)
    };

    cache.begin_frame();
    for glyph_id in
        1..=super::super::source_cache::NATIVE_BITMAP_ATLAS_MAX_RASTER_REQUESTS_PER_FRAME
    {
        assert!(matches!(
            cache.request_worker_image(
                &font_database,
                Some(&worker_pool),
                face_epoch,
                cache_key(glyph_id as u32),
            ),
            NativeBitmapAtlasWorkerRequestStatus::Submitted(_)
        ));
    }
    let deferred_key = cache_key(
        super::super::source_cache::NATIVE_BITMAP_ATLAS_MAX_RASTER_REQUESTS_PER_FRAME as u32 + 1,
    );
    assert_eq!(
        cache.request_worker_image(&font_database, Some(&worker_pool), face_epoch, deferred_key,),
        NativeBitmapAtlasWorkerRequestStatus::DeferredByFrameBudget
    );
    assert_eq!(
        cache.frame_report().worker_request_submitted_count,
        super::super::source_cache::NATIVE_BITMAP_ATLAS_MAX_RASTER_REQUESTS_PER_FRAME
    );
    assert_eq!(cache.frame_report().worker_request_deferred_count, 1);

    cache.begin_frame();
    assert!(matches!(
        cache.request_worker_image(&font_database, Some(&worker_pool), face_epoch, deferred_key,),
        NativeBitmapAtlasWorkerRequestStatus::Submitted(_)
    ));
    assert_eq!(cache.frame_report().worker_request_submitted_count, 1);
    assert_eq!(cache.frame_report().worker_request_deferred_count, 0);
}

#[test]
fn native_bitmap_atlas_source_cache_rejects_worker_completion_edges_without_cache_pollution() {
    let mut cache = NativeBitmapAtlasSourceCache::with_capacity(4);
    let failed_key = test_cache_key(41);
    let invalidated_key = test_cache_key(42);
    let failed_id = TextRasterWorkId::new(41);
    let invalidated_id = TextRasterWorkId::new(42);
    let unknown_id = TextRasterWorkId::new(43);
    let unknown_bitmap =
        GlyphBitmap::alpha_mask(UVec2::new(1, 1), Vec2::new(0.0, 1.0), 16.0, vec![255])
            .expect("test alpha bitmap should be valid");

    cache.begin_frame();
    cache.register_worker_request(failed_id, failed_key);
    cache.register_worker_request(invalidated_id, invalidated_key);

    let report = cache.apply_worker_completion_drain(TextRasterCompletionDrain {
        accepted: vec![
            worker_result(failed_id, Err(SwashRasterError::InvalidPxSize)),
            worker_result(unknown_id, Ok(unknown_bitmap)),
        ],
        face_invalidated_ids: vec![invalidated_id],
        face_invalidated_count: 1,
        ..TextRasterCompletionDrain::default()
    });

    assert_eq!(report.worker_completion_failed_count, 1);
    assert_eq!(report.worker_completion_unknown_count, 1);
    assert_eq!(report.worker_completion_face_invalidated_count, 1);
    assert_eq!(report.worker_completion_insert_count, 0);
    assert_eq!(report.pending_worker_count, 0);
    assert!(cache.cached_test_image(failed_key).is_none());
    assert!(cache.cached_test_image(invalidated_key).is_none());
}

fn worker_result(
    id: TextRasterWorkId,
    result: Result<GlyphBitmap, SwashRasterError>,
) -> TextRasterWorkResult {
    TextRasterWorkResult {
        id,
        face_epoch: 0,
        result,
    }
}
