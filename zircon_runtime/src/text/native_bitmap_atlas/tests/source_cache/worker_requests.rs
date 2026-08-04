use super::*;
use std::path::Path;
use std::sync::Arc;

use crate::text::font::FontDatabase;
use crate::text::parallel::raster_pool::{TextRasterWorkerPool, TextRasterWorkerPoolOptions};
use crate::text::raster::SwashRasterRequest;
use glyphon::FontSystem;
use glyphon::cosmic_text::{CacheKey, CacheKeyFlags, SubpixelBin, Weight, fontdb};
use swash::FontRef;

#[test]
fn native_bitmap_atlas_source_cache_schedules_glyphon_cache_key_worker_request() {
    let source_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraSans-Regular.ttf");
    let mut font_database = FontDatabase::default();
    let registered_face = font_database
        .register_font_file(&source_path, Some("Fira Sans"), 0)
        .expect("test font should register with the text font database");
    let mut font_system = FontSystem::new();
    font_database.sync_font_system(&mut font_system);
    let font_id = font_database
        .backend_face_id(registered_face)
        .expect("registered text font should expose a glyphon backend face");
    let face = font_system
        .db()
        .face(font_id)
        .expect("glyphon font system should contain the registered face");
    let face_index = face.index as usize;
    let parsed_font =
        FontRef::from_index(TEST_FONT_BYTES, face_index).expect("test font face should parse");
    let glyph_id = parsed_font.charmap().map('P');
    let second_glyph_id = parsed_font.charmap().map('Q');
    assert_ne!(glyph_id, 0, "test glyph should exist");
    assert_ne!(second_glyph_id, 0, "second test glyph should exist");
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
    let face_epoch = cache.face_epoch();

    cache.begin_frame();
    assert!(cache.cached_image(cache_key).is_none());
    let work_id = match cache.request_worker_image(
        &mut font_system,
        &font_database,
        Some(&worker_pool),
        face_epoch,
        cache_key,
    ) {
        NativeBitmapAtlasWorkerRequestStatus::Submitted(work_id) => work_id,
        status => panic!("source miss should submit a worker request, got {status:?}"),
    };
    assert_eq!(
        cache.request_worker_image(
            &mut font_system,
            &font_database,
            Some(&worker_pool),
            face_epoch,
            cache_key,
        ),
        NativeBitmapAtlasWorkerRequestStatus::Pending,
        "pending cache key should not enqueue duplicate worker work"
    );
    let second_cache_key = CacheKey {
        glyph_id: second_glyph_id,
        ..cache_key
    };
    let second_work_id = match cache.request_worker_image(
        &mut font_system,
        &font_database,
        Some(&worker_pool),
        face_epoch,
        second_cache_key,
    ) {
        NativeBitmapAtlasWorkerRequestStatus::Submitted(work_id) => work_id,
        status => panic!("second source miss should submit a worker request, got {status:?}"),
    };

    let work = worker_pool
        .try_recv_request_for_test()
        .expect("worker queue should receive source cache request");
    assert_eq!(work.id, work_id);
    assert_eq!(work.face_epoch, face_epoch);
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
        .with_variations(
            font_database
                .effective_instance_variations_shared(
                    registered_face,
                    None,
                    cache_key.font_weight.0,
                )
                .expect("registered face should resolve cached effective variations"),
        )
        .with_font_identity([face_epoch, 1])
    );
    let second_work = worker_pool
        .try_recv_request_for_test()
        .expect("worker queue should receive the second source cache request");
    assert_eq!(second_work.id, second_work_id);
    assert_eq!(second_work.face_epoch, face_epoch);
    assert!(Arc::ptr_eq(&work.font_data, &second_work.font_data));
    assert_eq!(
        second_work.request.font_identity,
        work.request.font_identity
    );
    assert!(Arc::ptr_eq(
        &second_work.request.variations,
        &work.request.variations
    ));
    assert!(worker_pool.try_recv_request_for_test().is_none());
    assert_eq!(
        cache.frame_report(),
        NativeBitmapAtlasSourceCacheFrameReport {
            capacity: 4,
            max_byte_count: TEST_DEFAULT_SOURCE_CACHE_MAX_BYTES,
            miss_count: 1,
            worker_request_submitted_count: 2,
            worker_request_pending_count: 1,
            worker_request_font_copied_byte_count: TEST_FONT_BYTES.len(),
            entry_count: 0,
            pending_worker_count: 2,
            ..NativeBitmapAtlasSourceCacheFrameReport::default()
        }
    );
}

#[test]
fn native_bitmap_atlas_source_cache_defers_full_worker_queue_without_pending_work() {
    let mut font_system =
        FontSystem::new_with_fonts([fontdb::Source::Binary(Arc::new(TEST_FONT_BYTES.to_vec()))]);
    let font_id = font_system
        .db()
        .faces()
        .find(|face| matches!(face.source, fontdb::Source::Binary(_)))
        .expect("test font should register a binary face")
        .id;
    let worker_pool = TextRasterWorkerPool::new_without_workers_for_test(
        TextRasterWorkerPoolOptions::new(1).with_queue_depth(0),
    );
    let mut cache = NativeBitmapAtlasSourceCache::with_capacity(4);
    let font_database = FontDatabase::default();
    let cache_key = CacheKey {
        font_id,
        glyph_id: 1,
        font_size_bits: 16.0f32.to_bits(),
        x_bin: SubpixelBin::Zero,
        y_bin: SubpixelBin::Zero,
        font_weight: Weight(400),
        flags: CacheKeyFlags::empty(),
    };

    cache.begin_frame();
    let face_epoch = cache.face_epoch();
    assert_eq!(
        cache.request_worker_image(
            &mut font_system,
            &font_database,
            Some(&worker_pool),
            face_epoch,
            cache_key,
        ),
        NativeBitmapAtlasWorkerRequestStatus::DeferredByWorkerBackpressure
    );
    assert_eq!(cache.frame_report().worker_request_backpressured_count, 1);
    assert_eq!(cache.frame_report().worker_request_failed_count, 0);
    assert_eq!(cache.frame_report().pending_worker_count, 0);
}

#[test]
fn native_bitmap_atlas_source_cache_reuses_font_bytes_until_face_invalidation() {
    let mut font_system =
        FontSystem::new_with_fonts([fontdb::Source::Binary(Arc::new(TEST_FONT_BYTES.to_vec()))]);
    let font_id = font_system
        .db()
        .faces()
        .find(|face| matches!(face.source, fontdb::Source::Binary(_)))
        .expect("test font should register a binary face")
        .id;
    let worker_pool = TextRasterWorkerPool::new_without_workers_for_test(
        TextRasterWorkerPoolOptions::new(1).with_queue_depth(2),
    );
    let mut cache = NativeBitmapAtlasSourceCache::with_capacity(4);
    let font_database = FontDatabase::default();
    let cache_key = CacheKey {
        font_id,
        glyph_id: 1,
        font_size_bits: 16.0f32.to_bits(),
        x_bin: SubpixelBin::Zero,
        y_bin: SubpixelBin::Zero,
        font_weight: Weight(400),
        flags: CacheKeyFlags::empty(),
    };

    cache.begin_frame();
    let first_face_epoch = cache.face_epoch();
    assert!(matches!(
        cache.request_worker_image(
            &mut font_system,
            &font_database,
            Some(&worker_pool),
            first_face_epoch,
            cache_key,
        ),
        NativeBitmapAtlasWorkerRequestStatus::Submitted(_)
    ));
    let first_work = worker_pool
        .try_recv_request_for_test()
        .expect("first request should enter the worker queue");

    cache.discard_all_for_face_invalidation_with_worker_pool(Some(&worker_pool));
    cache.begin_frame();
    let second_face_epoch = cache.face_epoch();
    assert!(matches!(
        cache.request_worker_image(
            &mut font_system,
            &font_database,
            Some(&worker_pool),
            second_face_epoch,
            cache_key,
        ),
        NativeBitmapAtlasWorkerRequestStatus::Submitted(_)
    ));
    let second_work = worker_pool
        .try_recv_request_for_test()
        .expect("invalidated face should enqueue a new request");

    assert!(!Arc::ptr_eq(&first_work.font_data, &second_work.font_data));
    assert_ne!(
        first_work.request.font_identity,
        second_work.request.font_identity
    );
    assert_eq!(
        cache.frame_report().worker_request_font_copied_byte_count,
        TEST_FONT_BYTES.len()
    );
}
