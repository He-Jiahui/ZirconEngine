use super::*;
use std::path::Path;
use std::sync::Arc;

use crate::core::math::{UVec2, Vec2};
use crate::text::font::FontDatabase;
use crate::text::parallel::raster_pool::{
    TextRasterCompletionDrain, TextRasterWorkId, TextRasterWorkerPool, TextRasterWorkerPoolOptions,
};
use crate::text::raster::GlyphBitmap;

#[test]
fn native_bitmap_atlas_source_cache_requests_exact_instance_once_per_glyph() {
    let (font_database, instance) = test_font_database_with_fira();
    let cache_key = GlyphRasterKey {
        face: instance,
        glyph_id: 47,
        subpixel_bin: 2,
        vertical_subpixel_bin: 1,
        synthetic: SyntheticGlyphStyle {
            bold: false,
            oblique: true,
        },
        ..test_cache_key(47)
    };
    let second_cache_key = GlyphRasterKey {
        glyph_id: 48,
        ..cache_key
    };
    let worker_pool = TextRasterWorkerPool::new_without_workers_for_test(
        TextRasterWorkerPoolOptions::new(1).with_queue_depth(4),
    );
    let mut cache = NativeBitmapAtlasSourceCache::with_capacity(4);
    let face_epoch = cache.face_epoch();

    cache.begin_frame();
    assert!(cache.cached_image(cache_key).is_none());
    let work_id =
        match cache.request_worker_image(&font_database, Some(&worker_pool), face_epoch, cache_key)
        {
            NativeBitmapAtlasWorkerRequestStatus::Submitted(work_id) => work_id,
            status => panic!("source miss should submit a worker request, got {status:?}"),
        };
    assert_eq!(
        cache.request_worker_image(&font_database, Some(&worker_pool), face_epoch, cache_key,),
        NativeBitmapAtlasWorkerRequestStatus::Pending,
        "a prepared glyph key must not enqueue duplicate work"
    );
    let second_work_id = match cache.request_worker_image(
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
        .expect("worker queue should receive the first source request");
    assert_eq!(work.id, work_id);
    assert_eq!(work.face_epoch, face_epoch);
    assert_eq!(work.request.glyph_id, 47);
    assert_eq!(work.request.offset.x, 2.0 / 3.0);
    assert_eq!(work.request.offset.y, 0.25);
    assert!(work.request.fake_italic);
    assert_eq!(work.request.font_identity, Some([face_epoch, 1]));
    assert_eq!(
        work.request.variations.as_ref(),
        &font_database
            .font_instance(instance)
            .expect("registered instance should remain resolvable")
            .variations
    );

    let second_work = worker_pool
        .try_recv_request_for_test()
        .expect("worker queue should receive the second source request");
    assert_eq!(second_work.id, second_work_id);
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
    assert_eq!(cache.frame_report().worker_request_submitted_count, 2);
    assert_eq!(cache.frame_report().worker_request_pending_count, 1);
    assert_eq!(cache.frame_report().pending_worker_count, 2);
}

#[test]
fn native_bitmap_atlas_worker_completion_binds_direct_persistent_raster_key() {
    let (font_database, instance) = test_font_database_with_fira();
    let cache_key = GlyphRasterKey {
        face: instance,
        glyph_id: 36,
        ..test_cache_key(36)
    };
    let bitmap = GlyphBitmap::alpha_mask(UVec2::new(2, 1), Vec2::new(0.0, 7.0), 18.0, vec![255; 2])
        .expect("test alpha bitmap should be valid");
    let mut cache = NativeBitmapAtlasSourceCache::with_capacity(4);

    cache.begin_frame();
    cache.register_worker_request(TextRasterWorkId::new(1), cache_key);
    let report = cache.apply_worker_completion_drain(TextRasterCompletionDrain {
        accepted: vec![worker_result(TextRasterWorkId::new(1), Ok(bitmap))],
        ..TextRasterCompletionDrain::default()
    });

    assert_eq!(report.worker_completion_insert_count, 1);
    assert_eq!(report.persistent_raster_key_count, 1);
    assert_eq!(
        cache
            .cached_image(cache_key)
            .expect("completed direct raster should be cached")
            .content,
        SwashContent::Mask
    );
    assert!(font_database.font_instance(instance).is_some());
}

#[test]
fn native_bitmap_atlas_source_cache_reports_backpressure_without_marking_work_pending() {
    let (font_database, instance) = test_font_database_with_fira();
    let worker_pool = TextRasterWorkerPool::new_without_workers_for_test(
        TextRasterWorkerPoolOptions::new(1).with_queue_depth(0),
    );
    let mut cache = NativeBitmapAtlasSourceCache::with_capacity(4);
    let cache_key = GlyphRasterKey {
        face: instance,
        glyph_id: 36,
        ..test_cache_key(36)
    };

    cache.begin_frame();
    assert_eq!(
        cache.request_worker_image(
            &font_database,
            Some(&worker_pool),
            cache.face_epoch(),
            cache_key,
        ),
        NativeBitmapAtlasWorkerRequestStatus::DeferredByWorkerBackpressure
    );
    assert_eq!(cache.frame_report().worker_request_backpressured_count, 1);
    assert_eq!(cache.frame_report().pending_worker_count, 0);
}

#[test]
fn native_bitmap_atlas_source_cache_reloads_font_bytes_after_face_invalidation() {
    let (font_database, instance) = test_font_database_with_fira();
    let worker_pool = TextRasterWorkerPool::new_without_workers_for_test(
        TextRasterWorkerPoolOptions::new(1).with_queue_depth(2),
    );
    let cache_key = GlyphRasterKey {
        face: instance,
        glyph_id: 36,
        ..test_cache_key(36)
    };
    let mut cache = NativeBitmapAtlasSourceCache::with_capacity(4);

    cache.begin_frame();
    assert!(matches!(
        cache.request_worker_image(
            &font_database,
            Some(&worker_pool),
            cache.face_epoch(),
            cache_key,
        ),
        NativeBitmapAtlasWorkerRequestStatus::Submitted(_)
    ));
    let first_work = worker_pool
        .try_recv_request_for_test()
        .expect("first direct request should enter the worker queue");
    cache.discard_all_for_face_invalidation_with_worker_pool(Some(&worker_pool));

    cache.begin_frame();
    assert!(matches!(
        cache.request_worker_image(
            &font_database,
            Some(&worker_pool),
            cache.face_epoch(),
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
    assert_eq!(cache.frame_report().worker_raster_font_entry_count, 1);
}

#[test]
fn native_bitmap_atlas_source_cache_tracks_distinct_registered_font_snapshots() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts");
    let mut font_database = FontDatabase::default();
    let fira = font_database
        .register_font_file(
            source_root.join("FiraSans-Regular.ttf"),
            Some("Fira Sans"),
            0,
        )
        .expect("first font should register");
    let mono = font_database
        .register_font_file(
            source_root.join("FiraMono-subset.ttf"),
            Some("Fira Mono"),
            0,
        )
        .expect("second font should register");
    let fira_instance = font_database
        .effective_instance_id(fira, 400)
        .expect("first instance should resolve");
    let mono_instance = font_database
        .effective_instance_id(mono, 400)
        .expect("second instance should resolve");
    let worker_pool = TextRasterWorkerPool::new_without_workers_for_test(
        TextRasterWorkerPoolOptions::new(1).with_queue_depth(2),
    );
    let mut cache = NativeBitmapAtlasSourceCache::with_capacity(4);

    cache.begin_frame();
    for (glyph_id, face) in [(36, fira_instance), (37, mono_instance)] {
        assert!(matches!(
            cache.request_worker_image(
                &font_database,
                Some(&worker_pool),
                cache.face_epoch(),
                GlyphRasterKey {
                    face,
                    glyph_id,
                    ..test_cache_key(glyph_id as u16)
                },
            ),
            NativeBitmapAtlasWorkerRequestStatus::Submitted(_)
        ));
    }

    assert_eq!(cache.frame_report().worker_raster_font_entry_count, 2);
    assert!(cache.frame_report().worker_raster_font_resident_byte_count > 0);
}
