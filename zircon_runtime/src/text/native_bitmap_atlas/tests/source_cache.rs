use super::*;
use std::sync::Arc;

use crate::core::math::Vec2;
use crate::text::parallel::raster_pool::{
    TextRasterCompletionDrain, TextRasterWorkId, TextRasterWorkResult, TextRasterWorkTarget,
    TextRasterWorkerPool, TextRasterWorkerPoolOptions,
};
use crate::text::raster::{GlyphBitmap, SwashRasterError, SwashRasterRequest};
use glyphon::cosmic_text::{fontdb, CacheKey, CacheKeyFlags, SubpixelBin, Weight};
use glyphon::FontSystem;
use swash::FontRef;

const TEST_FONT_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/fonts/FiraSans-Regular.ttf"
));

#[test]
fn native_bitmap_atlas_source_cache_evicts_least_recently_used_source() {
    let mut cache = NativeBitmapAtlasSourceCache::with_capacity(2);
    let first = test_cache_key(1);
    let second = test_cache_key(2);
    let third = test_cache_key(3);

    cache.begin_frame();
    cache.insert_test_image(first, test_cached_image(1));
    cache.insert_test_image(second, test_cached_image(2));
    assert_eq!(cache.cached_test_image(first).unwrap().bytes, vec![1; 4]);
    cache.insert_test_image(third, test_cached_image(3));

    assert!(cache.cached_test_image(second).is_none());
    assert_eq!(cache.cached_test_image(first).unwrap().bytes, vec![1; 4]);
    assert_eq!(cache.cached_test_image(third).unwrap().bytes, vec![3; 4]);
    assert_eq!(
        cache.frame_report(),
        NativeBitmapAtlasSourceCacheFrameReport {
            hit_count: 0,
            miss_count: 0,
            insert_count: 3,
            evicted_count: 1,
            invalidated_count: 0,
            entry_count: 2,
            ..NativeBitmapAtlasSourceCacheFrameReport::default()
        }
    );
}

#[test]
fn native_bitmap_atlas_idle_frame_discards_cached_source_images() {
    let mut cache = NativeBitmapAtlasSourceCache::with_capacity(4);
    let first = test_cache_key(11);
    let second = test_cache_key(12);

    cache.begin_frame();
    cache.insert_test_image(first, test_cached_image(11));
    cache.insert_test_image(second, test_cached_image(12));

    let mut retry_state = GlyphAtlasBitmapRetryFrameState::new();
    let report = native_bitmap_atlas_idle_prepare_report(&mut cache, &mut retry_state);

    assert_eq!(
        report.source_cache,
        NativeBitmapAtlasSourceCacheFrameReport {
            evicted_count: 2,
            entry_count: 0,
            ..NativeBitmapAtlasSourceCacheFrameReport::default()
        }
    );
    assert_eq!(report.source_image_count, 0);
    assert_eq!(report.visible_raster_glyph_count, 0);
    assert_eq!(report.retry_state.queued_blocked_glyph_count, 0);
    assert_eq!(report.retry_state.invalidated_blocked_glyph_count, 0);
    assert_eq!(report.submission.visible_glyph_count, 0);
    assert!(cache.cached_test_image(first).is_none());
    assert!(cache.cached_test_image(second).is_none());
}

#[test]
fn native_bitmap_atlas_idle_frame_reports_face_invalidated_retry_state() {
    let mut cache = NativeBitmapAtlasSourceCache::with_capacity(4);
    let invalidated_source = GlyphAtlasBitmapSource {
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
        ..TextRasterCompletionDrain::default()
    });

    assert_eq!(report.worker_completion_insert_count, 1);
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
    assert_eq!(image.bytes, vec![9; 8]);
}

#[test]
fn native_bitmap_atlas_source_cache_normalizes_horizontal_subpixel_bucket_for_lookup() {
    let mut cache = NativeBitmapAtlasSourceCache::with_capacity(4);
    let requested_key = test_cache_key(35);
    let mut same_pixel_key = requested_key;
    same_pixel_key.x_bin = SubpixelBin::Three;

    cache.begin_frame();
    cache.insert_test_image(same_pixel_key, test_cached_image(35));

    let image = cache
        .cached_image(requested_key)
        .expect("horizontal subpixel buckets should share the same stable editor glyph image");

    assert_eq!(image.bytes, vec![35; 4]);
    assert_eq!(
        cache.frame_report(),
        NativeBitmapAtlasSourceCacheFrameReport {
            hit_count: 1,
            approximate_hit_count: 0,
            miss_count: 0,
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
    neighboring_key.y_bin = SubpixelBin::One;

    cache.begin_frame();
    cache.insert_test_image(neighboring_key, test_cached_image(36));

    assert!(cache.cached_image(requested_key).is_none());
    let image = cache
        .approximate_cached_image(requested_key)
        .expect("same glyph in a neighboring vertical bucket should remain reusable");

    assert_eq!(image.bytes, vec![36; 4]);
    assert_eq!(
        cache.frame_report(),
        NativeBitmapAtlasSourceCacheFrameReport {
            hit_count: 1,
            approximate_hit_count: 1,
            miss_count: 1,
            insert_count: 1,
            entry_count: 1,
            ..NativeBitmapAtlasSourceCacheFrameReport::default()
        }
    );
}

#[test]
fn native_bitmap_atlas_source_cache_schedules_glyphon_cache_key_worker_request() {
    let mut font_system =
        FontSystem::new_with_fonts([fontdb::Source::Binary(Arc::new(TEST_FONT_BYTES.to_vec()))]);
    let face = font_system
        .db()
        .faces()
        .find(|face| matches!(face.source, fontdb::Source::Binary(_)))
        .expect("test font should register a binary face");
    let face_index = face.index as usize;
    let font_id = face.id;
    let glyph_id = FontRef::from_index(TEST_FONT_BYTES, face_index)
        .expect("test font face should parse")
        .charmap()
        .map('P');
    assert_ne!(glyph_id, 0, "test glyph should exist");
    let cache_key = CacheKey {
        font_id,
        glyph_id,
        font_size_bits: 17.25f32.to_bits(),
        x_bin: SubpixelBin::Three,
        y_bin: SubpixelBin::One,
        font_weight: Weight(500),
        flags: CacheKeyFlags::FAKE_ITALIC,
    };
    let worker_pool = TextRasterWorkerPool::new_without_workers_for_test(
        TextRasterWorkerPoolOptions::new(1).with_queue_depth(4),
    );
    let mut cache = NativeBitmapAtlasSourceCache::with_capacity(4);
    let target = TextRasterWorkTarget::new(TEST_BITMAP_ATLAS_FRAME_INDEX, cache.face_epoch());

    cache.begin_frame();
    assert!(cache.cached_image(cache_key).is_none());
    let work_id = cache
        .request_worker_image(
            &mut font_system,
            &FontDatabase::default(),
            Some(&worker_pool),
            target,
            cache_key,
        )
        .expect("source miss should submit a worker request");
    assert!(
        cache
            .request_worker_image(
                &mut font_system,
                &FontDatabase::default(),
                Some(&worker_pool),
                target,
                cache_key,
            )
            .is_none(),
        "pending cache key should not enqueue duplicate worker work"
    );

    let work = worker_pool
        .try_recv_request_for_test()
        .expect("worker queue should receive source cache request");
    assert_eq!(work.id, work_id);
    assert_eq!(work.target, target);
    assert_eq!(work.font_data.as_ref(), TEST_FONT_BYTES);
    assert_eq!(
        work.request,
        SwashRasterRequest::glyphon_cache_key(
            face_index,
            CacheKey {
                x_bin: SubpixelBin::Zero,
                ..cache_key
            }
        )
    );
    assert!(worker_pool.try_recv_request_for_test().is_none());
    assert_eq!(
        cache.frame_report(),
        NativeBitmapAtlasSourceCacheFrameReport {
            miss_count: 1,
            worker_request_submitted_count: 1,
            worker_request_pending_count: 1,
            entry_count: 0,
            pending_worker_count: 1,
            ..NativeBitmapAtlasSourceCacheFrameReport::default()
        }
    );
}

#[test]
fn native_bitmap_atlas_source_cache_rejects_worker_completion_edges_without_cache_pollution() {
    let mut cache = NativeBitmapAtlasSourceCache::with_capacity(4);
    let failed_key = test_cache_key(41);
    let stale_key = test_cache_key(42);
    let invalidated_key = test_cache_key(43);
    let failed_id = TextRasterWorkId::new(41);
    let stale_id = TextRasterWorkId::new(42);
    let invalidated_id = TextRasterWorkId::new(43);
    let unknown_id = TextRasterWorkId::new(44);
    let unknown_bitmap =
        GlyphBitmap::alpha_mask(UVec2::new(1, 1), Vec2::new(0.0, 1.0), 16.0, vec![255])
            .expect("test alpha bitmap should be valid");

    cache.begin_frame();
    cache.register_worker_request(failed_id, failed_key);
    cache.register_worker_request(stale_id, stale_key);
    cache.register_worker_request(invalidated_id, invalidated_key);

    let report = cache.apply_worker_completion_drain(TextRasterCompletionDrain {
        accepted: vec![
            worker_result(failed_id, Err(SwashRasterError::InvalidPxSize)),
            worker_result(unknown_id, Ok(unknown_bitmap)),
        ],
        stale_page_generation_ids: vec![stale_id],
        face_invalidated_ids: vec![invalidated_id],
        stale_page_generation_count: 1,
        face_invalidated_count: 1,
    });

    assert_eq!(report.worker_completion_failed_count, 1);
    assert_eq!(report.worker_completion_unknown_count, 1);
    assert_eq!(report.worker_completion_stale_page_generation_count, 1);
    assert_eq!(report.worker_completion_face_invalidated_count, 1);
    assert_eq!(report.worker_completion_insert_count, 0);
    assert_eq!(report.pending_worker_count, 0);
    assert!(cache.cached_test_image(failed_key).is_none());
    assert!(cache.cached_test_image(stale_key).is_none());
    assert!(cache.cached_test_image(invalidated_key).is_none());
}

fn worker_result(
    id: TextRasterWorkId,
    result: Result<GlyphBitmap, SwashRasterError>,
) -> TextRasterWorkResult {
    TextRasterWorkResult {
        id,
        target: TextRasterWorkTarget::new(TEST_BITMAP_ATLAS_FRAME_INDEX, 0),
        result,
    }
}
