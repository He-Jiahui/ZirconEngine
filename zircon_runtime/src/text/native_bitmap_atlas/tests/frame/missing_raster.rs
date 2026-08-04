use super::*;

#[test]
fn native_bitmap_atlas_frame_keeps_glyphon_when_raster_image_is_missing() {
    let source = GlyphAtlasBitmapSource {
        raster_key: None,
        format: GlyphAtlasFormat::AlphaMask,
        content_size: UVec2::new(8, 8),
        screen_rect: GlyphAtlasScreenRect::new(4.0, 4.0, 8.0, 8.0),
        foreground_color: [1.0, 1.0, 1.0, 1.0],
        background_color: [0.0, 0.0, 0.0, 1.0],
        source_byte_len: 64,
    };
    let submission = test_submission([source]);
    let frame = NativeBitmapAtlasFrame {
        missing_raster_image_count: 1,
        visible_missing_raster_image_count: 1,
        ..test_frame(
            submission,
            vec![test_source_image(source, vec![255; 64])],
            1,
            0,
            0,
        )
    };
    let report = frame.prepare_report();

    assert!(!frame.replaces_glyphon());
    assert_eq!(report.missing_raster_image_count, 1);
    assert_eq!(report.visible_raster_glyph_count, 1);
    assert_eq!(report.source_image_count, 1);
    assert_eq!(
        report.glyphon_fallback_reason,
        Some(NativeBitmapAtlasGlyphonFallbackReason::MissingRasterImage)
    );
    assert_eq!(
        report.first_frame_degradation,
        Some(NativeBitmapAtlasFirstFrameDegradation::GlyphonFallback)
    );
    assert!(!report.replaces_glyphon);
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

    assert!(!frame.replaces_glyphon());
    assert_eq!(report.missing_raster_image_count, 1);
    assert_eq!(report.source_image_count, 0);
    assert_eq!(report.submission.visible_placeholder_count, 1);
    assert_eq!(
        native_bitmap_atlas_handoff_for_report(&report),
        NativeBitmapAtlasHandoff::TransparentPlaceholder
    );
    assert_eq!(report.glyphon_fallback_reason, None);
    assert_eq!(
        report.first_frame_degradation,
        Some(NativeBitmapAtlasFirstFrameDegradation::TransparentPlaceholder)
    );
    assert!(!report.replaces_glyphon);
}

#[test]
fn native_bitmap_atlas_frame_schedules_worker_miss_as_transparent_placeholder() {
    let mut font_system = FontSystem::new_with_fonts([fontdb::Source::Binary(
        std::sync::Arc::new(TEST_FRAME_FONT_BYTES.to_vec()),
    )]);
    let mut buffer = Buffer::new(&mut font_system, Metrics::new(16.0, 20.0));
    buffer.set_size(&mut font_system, Some(128.0), Some(32.0));
    buffer.set_text(
        &mut font_system,
        "P",
        &Attrs::new(),
        Shaping::Advanced,
        None,
    );
    buffer.shape_until_scroll(&mut font_system, false);
    let text_area = TextArea {
        buffer: &buffer,
        left: 4.0,
        top: 6.0,
        scale: 1.0,
        bounds: TextBounds {
            left: 0,
            top: 0,
            right: 128,
            bottom: 64,
        },
        default_color: Color::rgba(255, 255, 255, 255),
        custom_glyphs: &[],
    };
    let bitmap_text_area = NativeBitmapAtlasTextArea::new(&text_area, None);
    let worker_pool = TextRasterWorkerPool::new_without_workers_for_test(
        TextRasterWorkerPoolOptions::new(1).with_queue_depth(4),
    );
    let mut source_cache = NativeBitmapAtlasSourceCache::with_capacity(4);
    let mut retry_state = GlyphAtlasBitmapRetryFrameState::new();

    let frame = native_bitmap_atlas_frame(
        &mut font_system,
        &FontDatabase::default(),
        Some(&worker_pool),
        &mut source_cache,
        &mut retry_state,
        GlyphAtlasSet::default(),
        test_viewport_size(),
        TEST_BITMAP_ATLAS_FRAME_INDEX,
        &[bitmap_text_area],
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
    assert_eq!(
        report.first_frame_degradation,
        Some(NativeBitmapAtlasFirstFrameDegradation::TransparentPlaceholder)
    );
    assert!(worker_pool.try_recv_request_for_test().is_some());
}

#[test]
fn native_bitmap_atlas_frame_falls_back_when_visible_worker_raster_is_unavailable() {
    let mut font_system = FontSystem::new_with_fonts([fontdb::Source::Binary(
        std::sync::Arc::new(TEST_FRAME_FONT_BYTES.to_vec()),
    )]);
    let mut buffer = Buffer::new(&mut font_system, Metrics::new(16.0, 20.0));
    buffer.set_size(&mut font_system, Some(128.0), Some(32.0));
    buffer.set_text(
        &mut font_system,
        "P",
        &Attrs::new(),
        Shaping::Advanced,
        None,
    );
    buffer.shape_until_scroll(&mut font_system, false);
    let text_area = TextArea {
        buffer: &buffer,
        left: 4.0,
        top: 6.0,
        scale: 1.0,
        bounds: TextBounds {
            left: 0,
            top: 0,
            right: 128,
            bottom: 64,
        },
        default_color: Color::rgba(255, 255, 255, 255),
        custom_glyphs: &[],
    };
    let bitmap_text_area = NativeBitmapAtlasTextArea::new(&text_area, None);
    let mut source_cache = NativeBitmapAtlasSourceCache::with_capacity(4);
    let mut retry_state = GlyphAtlasBitmapRetryFrameState::new();

    let frame = native_bitmap_atlas_frame(
        &mut font_system,
        &FontDatabase::default(),
        None,
        &mut source_cache,
        &mut retry_state,
        GlyphAtlasSet::default(),
        test_viewport_size(),
        TEST_BITMAP_ATLAS_FRAME_INDEX,
        &[bitmap_text_area],
    );
    let report = frame.prepare_report();

    assert_eq!(report.missing_raster_image_count, 1);
    assert_eq!(report.visible_missing_raster_image_count, 1);
    assert_eq!(report.source_image_count, 0);
    assert_eq!(report.source_cache.worker_request_submitted_count, 0);
    assert_eq!(report.source_cache.worker_request_unavailable_count, 1);
    assert_eq!(report.submission.visible_placeholder_count, 0);
    assert_eq!(
        native_bitmap_atlas_handoff_for_report(&report),
        NativeBitmapAtlasHandoff::GlyphonFallback
    );
    assert_eq!(
        report.glyphon_fallback_reason,
        Some(NativeBitmapAtlasGlyphonFallbackReason::MissingRasterImage)
    );
    assert_eq!(
        report.first_frame_degradation,
        Some(NativeBitmapAtlasFirstFrameDegradation::GlyphonFallback)
    );
}

#[test]
fn native_bitmap_atlas_pending_placeholder_respects_text_area_bounds() {
    let mut font_system = FontSystem::new_with_fonts([fontdb::Source::Binary(
        std::sync::Arc::new(TEST_FRAME_FONT_BYTES.to_vec()),
    )]);
    let mut buffer = Buffer::new(&mut font_system, Metrics::new(16.0, 20.0));
    buffer.set_size(&mut font_system, Some(128.0), Some(32.0));
    buffer.set_text(
        &mut font_system,
        "P",
        &Attrs::new(),
        Shaping::Advanced,
        None,
    );
    buffer.shape_until_scroll(&mut font_system, false);
    let text_area = TextArea {
        buffer: &buffer,
        left: 32.0,
        top: 16.0,
        scale: 1.0,
        bounds: TextBounds {
            left: 0,
            top: 0,
            right: 1,
            bottom: 1,
        },
        default_color: Color::rgba(255, 255, 255, 255),
        custom_glyphs: &[],
    };
    let bitmap_text_area = NativeBitmapAtlasTextArea::new(&text_area, None);
    let worker_pool = TextRasterWorkerPool::new_without_workers_for_test(
        TextRasterWorkerPoolOptions::new(1).with_queue_depth(4),
    );
    let mut source_cache = NativeBitmapAtlasSourceCache::with_capacity(4);
    let mut retry_state = GlyphAtlasBitmapRetryFrameState::new();

    let frame = native_bitmap_atlas_frame(
        &mut font_system,
        &FontDatabase::default(),
        Some(&worker_pool),
        &mut source_cache,
        &mut retry_state,
        GlyphAtlasSet::default(),
        test_viewport_size(),
        TEST_BITMAP_ATLAS_FRAME_INDEX,
        &[bitmap_text_area],
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
    assert_eq!(report.glyphon_fallback_reason, None);
    assert_eq!(report.first_frame_degradation, None);
    assert!(worker_pool.try_recv_request_for_test().is_none());
}
