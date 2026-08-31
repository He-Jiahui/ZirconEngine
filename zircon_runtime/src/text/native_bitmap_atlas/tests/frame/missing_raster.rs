use super::*;

#[test]
fn native_bitmap_atlas_frame_reports_missing_visible_raster_without_replacement() {
    let source = GlyphAtlasBitmapSource {
        raster_key: None,
        format: GlyphAtlasFormat::AlphaMask,
        content_size: UVec2::new(8, 8),
        screen_rect: GlyphAtlasScreenRect::new(4.0, 4.0, 8.0, 8.0),
        foreground_color: [1.0, 1.0, 1.0, 1.0],
        background_color: [0.0, 0.0, 0.0, 1.0],
        source_byte_len: 64,
    };
    let frame = NativeBitmapAtlasFrame {
        missing_raster_image_count: 1,
        visible_missing_raster_image_count: 1,
        ..test_frame(
            test_submission([source]),
            vec![test_source_image(source, vec![255; 64])],
            1,
            0,
            0,
        )
    };
    let report = frame.prepare_report();

    assert!(!frame.supports_native_submission());
    assert_eq!(report.missing_raster_image_count, 1);
    assert_eq!(report.visible_raster_glyph_count, 1);
    assert_eq!(report.source_image_count, 1);
    assert_eq!(
        report.native_degradation_reason,
        Some(NativeBitmapAtlasDegradationReason::MissingRasterImage)
    );
    assert_eq!(
        report.first_frame_degradation,
        Some(NativeBitmapAtlasFirstFrameDegradation::NativeRasterUnavailable)
    );
}

#[test]
fn native_bitmap_atlas_frame_uses_transparent_placeholder_for_pending_worker_raster() {
    let mut submission = test_submission(std::iter::empty::<GlyphAtlasBitmapSource>());
    submission.append_placeholder_glyphs(
        [GlyphAtlasBitmapPlaceholderGlyph {
            source_index: 0,
            format: GlyphAtlasFormat::AlphaMask,
            screen_rect: GlyphAtlasScreenRect::new(12.0, 6.0, 8.0, 16.0),
            retry_frame_index: TEST_BITMAP_ATLAS_FRAME_INDEX.saturating_add(1),
            mode: GlyphAtlasBitmapPlaceholderMode::TransparentQuad,
        }],
        test_clip_rect(),
    );
    let frame = NativeBitmapAtlasFrame {
        missing_raster_image_count: 1,
        visible_missing_raster_image_count: 1,
        ..test_frame(submission, Vec::new(), 0, 0, 0)
    };
    let report = frame.prepare_report();

    assert!(!frame.supports_native_submission());
    assert_eq!(report.submission.visible_placeholder_count, 1);
    assert_eq!(
        native_bitmap_atlas_handoff_for_report(&report),
        NativeBitmapAtlasHandoff::TransparentPlaceholder
    );
    assert_eq!(report.native_degradation_reason, None);
}

#[test]
fn native_bitmap_atlas_frame_schedules_prepared_glyph_miss_as_transparent_placeholder() {
    let (font_database, instance) = test_font_database_with_fira();
    let raster_key = GlyphRasterKey {
        face: instance,
        glyph_id: 47,
        ..test_cache_key(47)
    };
    let worker_pool = TextRasterWorkerPool::new_without_workers_for_test(
        TextRasterWorkerPoolOptions::new(1).with_queue_depth(4),
    );
    let mut source_cache = NativeBitmapAtlasSourceCache::with_capacity(4);
    let mut retry_state = GlyphAtlasBitmapRetryFrameState::new();

    let frame = native_bitmap_atlas_frame(
        &font_database,
        Some(&worker_pool),
        &mut source_cache,
        &mut retry_state,
        GlyphAtlasSet::default(),
        test_viewport_size(),
        TEST_BITMAP_ATLAS_FRAME_INDEX,
        &[test_glyph_run_with_key(raster_key, test_clip_rect())],
    );
    let report = frame.prepare_report();

    assert_eq!(report.missing_raster_image_count, 1);
    assert_eq!(report.source_image_count, 0);
    assert_eq!(report.source_cache.worker_request_submitted_count, 1);
    assert_eq!(report.submission.visible_placeholder_count, 1);
    assert_eq!(
        native_bitmap_atlas_handoff_for_report(&report),
        NativeBitmapAtlasHandoff::TransparentPlaceholder
    );
    assert!(worker_pool.try_recv_request_for_test().is_some());
}

#[test]
fn native_bitmap_atlas_frame_keeps_offscreen_prepared_miss_out_of_the_worker_queue() {
    let (font_database, instance) = test_font_database_with_fira();
    let raster_key = GlyphRasterKey {
        face: instance,
        glyph_id: 47,
        ..test_cache_key(47)
    };
    let worker_pool = TextRasterWorkerPool::new_without_workers_for_test(
        TextRasterWorkerPoolOptions::new(1).with_queue_depth(4),
    );
    let mut source_cache = NativeBitmapAtlasSourceCache::with_capacity(4);
    let mut retry_state = GlyphAtlasBitmapRetryFrameState::new();

    let frame = native_bitmap_atlas_frame(
        &font_database,
        Some(&worker_pool),
        &mut source_cache,
        &mut retry_state,
        GlyphAtlasSet::default(),
        test_viewport_size(),
        TEST_BITMAP_ATLAS_FRAME_INDEX,
        &[test_glyph_run_with_key(
            raster_key,
            GlyphAtlasScreenRect::new(0.0, 0.0, 1.0, 1.0),
        )],
    );
    let report = frame.prepare_report();

    assert_eq!(report.missing_raster_image_count, 1);
    assert_eq!(report.visible_missing_raster_image_count, 0);
    assert_eq!(report.visible_raster_glyph_count, 0);
    assert_eq!(report.source_cache.worker_request_submitted_count, 0);
    assert_eq!(report.submission.visible_placeholder_count, 0);
    assert_eq!(
        native_bitmap_atlas_handoff_for_report(&report),
        NativeBitmapAtlasHandoff::NoVisibleGlyphs
    );
    assert!(worker_pool.try_recv_request_for_test().is_none());
}
