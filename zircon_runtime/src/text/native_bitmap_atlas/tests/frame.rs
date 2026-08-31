use super::*;

#[path = "frame/missing_raster.rs"]
mod missing_raster;

#[test]
fn native_bitmap_atlas_frame_supports_native_submission_only_with_alpha_source_coverage() {
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
    let frame = test_frame(
        submission.clone(),
        vec![test_source_image(source, vec![255; 64])],
        1,
        0,
        0,
    );

    assert!(frame.supports_native_submission());
    assert_eq!(frame.atlas_layer_count(), 1);
    assert_eq!(
        frame.atlas_storage_format(),
        Some(GlyphAtlasStorageFormat::R8Unorm)
    );
    assert_eq!(frame.source_bytes().next().unwrap().bytes.len(), 64);
    assert_eq!(
        frame.prepare_report(),
        NativeBitmapAtlasPrepareReport {
            frame_index: TEST_BITMAP_ATLAS_FRAME_INDEX,
            visible_raster_glyph_count: 1,
            source_image_count: 1,
            missing_raster_image_count: 0,
            visible_missing_raster_image_count: 0,
            approximate_raster_image_count: 0,
            unsupported_glyph_count: 0,
            clipped_glyph_count: 0,
            atlas_storage_format: Some(GlyphAtlasStorageFormat::R8Unorm),
            mixed_atlas_storage_format: false,
            storage_submission_count: 1,
            storage_submission_visible_glyph_count: 1,
            mixed_storage_replacement_ready: false,
            requires_background_composite: false,
            background_composite_replacement_ready: false,
            background_composite_glyph_count: 0,
            missing_background_composite_glyph_count: 0,
            source_cache: NativeBitmapAtlasSourceCacheFrameReport::default(),
            retry_submission: GlyphAtlasBitmapRetryFrameSubmissionReport::default(),
            retry_state: GlyphAtlasBitmapRetryFrameStateReport::default(),
            discarded_stale_retry_glyph_count: 0,
            native_degradation_reason: None,
            first_frame_degradation: None,
            native_submission_ready: true,
            submission: frame.submission.submission_report(),
        }
    );

    let unsupported = NativeBitmapAtlasFrame {
        unsupported_glyph_count: 1,
        ..frame
    };
    assert!(!unsupported.supports_native_submission());
}

#[test]
fn native_bitmap_atlas_frame_does_not_schedule_offscreen_approximate_worker() {
    let (font_database, instance) = test_font_database_with_fira();
    let requested_key = GlyphRasterKey {
        face: instance,
        glyph_id: 47,
        vertical_subpixel_bin: 0,
        ..test_cache_key(47)
    };
    let approximate_key = GlyphRasterKey {
        vertical_subpixel_bin: 1,
        ..requested_key
    };
    let worker_pool = TextRasterWorkerPool::new_without_workers_for_test(
        TextRasterWorkerPoolOptions::new(1).with_queue_depth(4),
    );
    let mut source_cache = NativeBitmapAtlasSourceCache::with_capacity(4);
    source_cache.insert_test_image(approximate_key, test_cached_image(7));
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
            requested_key,
            GlyphAtlasScreenRect::new(0.0, 0.0, 1.0, 1.0),
        )],
    );
    let report = frame.prepare_report();

    assert_eq!(report.missing_raster_image_count, 0);
    assert_eq!(report.approximate_raster_image_count, 0);
    assert_eq!(report.visible_raster_glyph_count, 0);
    assert_eq!(report.source_image_count, 0);
    assert_eq!(report.source_cache.approximate_hit_count, 1);
    assert_eq!(report.source_cache.worker_request_submitted_count, 0);
    assert_eq!(report.submission.visible_placeholder_count, 0);
    assert_eq!(
        native_bitmap_atlas_handoff_for_report(&report),
        NativeBitmapAtlasHandoff::NoVisibleGlyphs
    );
    assert_eq!(report.first_frame_degradation, None);
    assert!(worker_pool.try_recv_request_for_test().is_none());
}

#[test]
fn native_bitmap_atlas_frame_reuses_approximate_bucket_and_requests_the_exact_instance() {
    let (font_database, instance) = test_font_database_with_fira();
    let requested_key = GlyphRasterKey {
        face: instance,
        glyph_id: 47,
        vertical_subpixel_bin: 0,
        ..test_cache_key(47)
    };
    let approximate_key = GlyphRasterKey {
        vertical_subpixel_bin: 1,
        ..requested_key
    };
    let worker_pool = TextRasterWorkerPool::new_without_workers_for_test(
        TextRasterWorkerPoolOptions::new(1).with_queue_depth(4),
    );
    let mut source_cache = NativeBitmapAtlasSourceCache::with_capacity(4);
    source_cache.insert_test_image(approximate_key, test_cached_image(7));
    let mut retry_state = GlyphAtlasBitmapRetryFrameState::new();

    let frame = native_bitmap_atlas_frame(
        &font_database,
        Some(&worker_pool),
        &mut source_cache,
        &mut retry_state,
        GlyphAtlasSet::default(),
        test_viewport_size(),
        TEST_BITMAP_ATLAS_FRAME_INDEX,
        &[test_glyph_run_with_key(requested_key, test_clip_rect())],
    );
    let report = frame.prepare_report();

    assert_eq!(report.missing_raster_image_count, 0);
    assert_eq!(report.approximate_raster_image_count, 1);
    assert_eq!(report.source_image_count, 1);
    assert_eq!(report.source_cache.approximate_hit_count, 1);
    assert_eq!(report.source_cache.worker_request_submitted_count, 1);
    assert_eq!(report.submission.visible_placeholder_count, 0);
    assert_eq!(
        native_bitmap_atlas_handoff_for_report(&report),
        NativeBitmapAtlasHandoff::SingleStorageReplacement
    );
    assert_eq!(
        report.first_frame_degradation,
        Some(NativeBitmapAtlasFirstFrameDegradation::ApproximateBucketReplacement)
    );
    let work = worker_pool
        .try_recv_request_for_test()
        .expect("approximate bucket should still enqueue the exact glyph worker");
    assert_eq!(work.request.glyph_id, requested_key.glyph_id as u16);
    assert_eq!(work.request.offset.y, 0.0);
    assert_eq!(
        work.request.font_identity,
        Some([source_cache.face_epoch(), 1])
    );
    assert_eq!(
        work.request.variations.as_ref(),
        &font_database
            .font_instance(instance)
            .expect("exact text instance must remain registered")
            .variations
    );
}

#[test]
fn native_bitmap_atlas_prepare_report_carries_frame_index() {
    let source = GlyphAtlasBitmapSource {
        raster_key: None,
        format: GlyphAtlasFormat::AlphaMask,
        content_size: UVec2::new(8, 8),
        screen_rect: GlyphAtlasScreenRect::new(4.0, 4.0, 8.0, 8.0),
        foreground_color: [1.0, 1.0, 1.0, 1.0],
        background_color: [0.0, 0.0, 0.0, 1.0],
        source_byte_len: 64,
    };
    let submission = glyph_atlas_bitmap_render_submission_plan(
        [source],
        UVec2::new(64, 64),
        91,
        GLYPH_ATLAS_DEFAULT_MAX_PAGES_PER_FORMAT,
        test_viewport_size(),
        test_clip_rect(),
    );
    let frame = NativeBitmapAtlasFrame {
        submission,
        source_images: vec![test_source_image(source, vec![255; 64])],
        frame_index: 91,
        viewport_size: test_viewport_size(),
        clip_rect: test_clip_rect(),
        visible_raster_glyph_count: 1,
        missing_raster_image_count: 0,
        visible_missing_raster_image_count: 0,
        approximate_raster_image_count: 0,
        unsupported_glyph_count: 0,
        clipped_glyph_count: 0,
        background_composite_glyph_count: 0,
        missing_background_composite_glyph_count: 0,
        source_cache: NativeBitmapAtlasSourceCacheFrameReport::default(),
        retry_submission: GlyphAtlasBitmapRetryFrameSubmissionReport::default(),
        retry_state: GlyphAtlasBitmapRetryFrameStateReport::default(),
        discarded_stale_retry_glyph_count: 0,
        face_epoch: 0,
    };

    assert_eq!(frame.prepare_report().frame_index, 91);
}

#[test]
fn native_bitmap_atlas_frame_reports_face_validity_from_source_epochs() {
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
    let mut frame = test_frame(
        submission,
        vec![test_source_image(source, vec![255; 64])],
        1,
        0,
        0,
    );

    assert_eq!(frame.source_bytes().next().unwrap().face_epoch, 0);
    assert_eq!(frame.face_validity(), GlyphAtlasBitmapFaceValidity::Valid);

    frame.face_epoch = 1;

    assert_eq!(
        frame.face_validity(),
        GlyphAtlasBitmapFaceValidity::Invalidated
    );
}

#[test]
fn native_bitmap_atlas_frame_marks_contiguous_mixed_storage_ready_for_renderer_handoff() {
    let alpha = GlyphAtlasBitmapSource {
        raster_key: None,
        format: GlyphAtlasFormat::AlphaMask,
        content_size: UVec2::new(8, 8),
        screen_rect: GlyphAtlasScreenRect::new(4.0, 4.0, 8.0, 8.0),
        foreground_color: [1.0, 1.0, 1.0, 1.0],
        background_color: [0.0, 0.0, 0.0, 1.0],
        source_byte_len: 64,
    };
    let color = GlyphAtlasBitmapSource {
        raster_key: None,
        format: GlyphAtlasFormat::Color,
        content_size: UVec2::new(8, 8),
        screen_rect: GlyphAtlasScreenRect::new(18.0, 4.0, 8.0, 8.0),
        foreground_color: [1.0, 1.0, 1.0, 1.0],
        background_color: [0.0, 0.0, 0.0, 1.0],
        source_byte_len: 256,
    };
    let submission = test_submission([alpha, color]);
    let frame = test_frame(
        submission,
        vec![
            test_source_image(alpha, vec![255; 64]),
            test_source_image(color, vec![255; 256]),
        ],
        2,
        0,
        0,
    );

    let report = frame.prepare_report();

    assert!(!frame.supports_native_submission());
    assert_eq!(frame.atlas_storage_format(), None);
    assert!(report.mixed_atlas_storage_format);
    assert_eq!(frame.canonical_frame_plan_count(), 1);
    assert_eq!(frame.storage_resource_count(), 2);
    assert_eq!(frame.ordered_draw_segment_count(), 2);
    assert_eq!(report.storage_submission_count, 1);
    assert_eq!(report.storage_submission_visible_glyph_count, 2);
    assert!(report.mixed_storage_replacement_ready);
    assert!(!report.requires_background_composite);
    assert!(!report.native_submission_ready);
    assert_eq!(report.submission.visible_glyph_count, 2);
    assert_eq!(frame.source_bytes().count(), 2);
    assert_eq!(frame.submission.gpu_draw.visible_glyph_count, 2);
}

#[test]
fn native_bitmap_atlas_frame_separates_same_storage_format_atlas_layers() {
    let subpixel = GlyphAtlasBitmapSource {
        raster_key: None,
        format: GlyphAtlasFormat::SubpixelMask,
        content_size: UVec2::new(8, 8),
        screen_rect: GlyphAtlasScreenRect::new(4.0, 4.0, 8.0, 8.0),
        foreground_color: [1.0, 1.0, 1.0, 1.0],
        background_color: [0.0, 0.0, 0.0, 1.0],
        source_byte_len: 256,
    };
    let color = GlyphAtlasBitmapSource {
        raster_key: None,
        format: GlyphAtlasFormat::Color,
        content_size: UVec2::new(8, 8),
        screen_rect: GlyphAtlasScreenRect::new(18.0, 4.0, 8.0, 8.0),
        foreground_color: [1.0, 1.0, 1.0, 1.0],
        background_color: [0.0, 0.0, 0.0, 1.0],
        source_byte_len: 256,
    };
    let mut frame = test_frame(
        test_submission([subpixel, color]),
        vec![
            test_source_image(subpixel, vec![255; 256]),
            test_source_image(color, vec![127; 256]),
        ],
        2,
        0,
        0,
    );
    frame.background_composite_glyph_count = 1;

    let report = frame.prepare_report();

    assert_eq!(
        frame.atlas_storage_format(),
        Some(GlyphAtlasStorageFormat::Rgba8Unorm)
    );
    assert_eq!(frame.atlas_format(), None);
    assert!(!frame.supports_native_submission());
    assert!(!report.mixed_atlas_storage_format);
    assert!(report.mixed_storage_replacement_ready);
    assert_eq!(
        native_bitmap_atlas_handoff_for_report(&report),
        NativeBitmapAtlasHandoff::MixedStorageReplacement
    );
    assert_eq!(frame.canonical_frame_plan_count(), 1);
    assert_eq!(frame.storage_resource_count(), 2);
    assert_eq!(frame.ordered_draw_segment_count(), 2);
}

#[test]
fn native_bitmap_atlas_canonical_frame_plan_inherits_persistent_frame_atlas() {
    let alpha = GlyphAtlasBitmapSource {
        raster_key: None,
        format: GlyphAtlasFormat::AlphaMask,
        content_size: UVec2::new(8, 8),
        screen_rect: GlyphAtlasScreenRect::new(4.0, 4.0, 8.0, 8.0),
        foreground_color: [1.0, 1.0, 1.0, 1.0],
        background_color: [0.0, 0.0, 0.0, 1.0],
        source_byte_len: 64,
    };
    let color = GlyphAtlasBitmapSource {
        raster_key: None,
        format: GlyphAtlasFormat::Color,
        content_size: UVec2::new(8, 8),
        screen_rect: GlyphAtlasScreenRect::new(18.0, 4.0, 8.0, 8.0),
        foreground_color: [1.0, 1.0, 1.0, 1.0],
        background_color: [0.0, 0.0, 0.0, 1.0],
        source_byte_len: 256,
    };
    let alpha_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0);
    let color_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::Color, 0);
    let persistent_atlas = GlyphAtlasSet::from_page(
        GlyphAtlasPageSpec::new(alpha_key, bitmap_atlas_page_size()).with_generation(11),
    )
    .with_page(GlyphAtlasPageSpec::new(color_key, bitmap_atlas_page_size()).with_generation(23));
    let submission = glyph_atlas_bitmap_render_submission_plan_with_atlas(
        persistent_atlas,
        [alpha, color],
        bitmap_atlas_page_size(),
        TEST_BITMAP_ATLAS_FRAME_INDEX,
        GLYPH_ATLAS_DEFAULT_MAX_PAGES_PER_FORMAT,
        test_viewport_size(),
        test_clip_rect(),
    );
    let frame = test_frame(
        submission,
        vec![
            test_source_image(alpha, vec![255; 64]),
            test_source_image(color, vec![255; 256]),
        ],
        2,
        0,
        0,
    );
    let frame_alpha_generation = frame
        .submission
        .run
        .atlas
        .page(GlyphAtlasFormat::AlphaMask, 0)
        .map(|page| page.generation)
        .unwrap();
    let frame_color_generation = frame
        .submission
        .run
        .atlas
        .page(GlyphAtlasFormat::Color, 0)
        .map(|page| page.generation)
        .unwrap();

    assert_eq!(frame_alpha_generation, 12);
    assert_eq!(frame_color_generation, 24);
    assert_eq!(
        frame
            .submission
            .run
            .atlas
            .page(GlyphAtlasFormat::AlphaMask, 0)
            .map(|page| page.generation),
        Some(frame_alpha_generation)
    );
    assert_eq!(
        frame
            .submission
            .run
            .atlas
            .page(GlyphAtlasFormat::Color, 0)
            .map(|page| page.generation),
        Some(frame_color_generation)
    );
    assert_eq!(frame.canonical_frame_plan_count(), 1);
    assert_eq!(frame.submission.run.upload_copies.len(), 2);
}

#[test]
fn native_bitmap_atlas_frame_supports_native_submission_for_single_color_storage() {
    let color = GlyphAtlasBitmapSource {
        raster_key: None,
        format: GlyphAtlasFormat::Color,
        content_size: UVec2::new(8, 8),
        screen_rect: GlyphAtlasScreenRect::new(18.0, 4.0, 8.0, 8.0),
        foreground_color: [1.0, 1.0, 1.0, 1.0],
        background_color: [0.0, 0.0, 0.0, 1.0],
        source_byte_len: 256,
    };
    let submission = test_submission([color]);
    let frame = test_frame(
        submission,
        vec![test_source_image(color, vec![255; 256])],
        1,
        0,
        0,
    );
    let report = frame.prepare_report();

    assert!(frame.supports_native_submission());
    assert_eq!(
        frame.atlas_storage_format(),
        Some(GlyphAtlasStorageFormat::Rgba8Unorm)
    );
    assert!(!report.mixed_atlas_storage_format);
    assert_eq!(report.storage_submission_count, 1);
    assert_eq!(report.storage_submission_visible_glyph_count, 1);
    assert!(!report.mixed_storage_replacement_ready);
    assert!(!report.requires_background_composite);
    assert!(report.native_submission_ready);
}

#[test]
fn native_bitmap_atlas_frame_degrades_for_missing_subpixel_background_composite() {
    let subpixel = GlyphAtlasBitmapSource {
        raster_key: None,
        format: GlyphAtlasFormat::SubpixelMask,
        content_size: UVec2::new(8, 8),
        screen_rect: GlyphAtlasScreenRect::new(18.0, 4.0, 8.0, 8.0),
        foreground_color: [1.0, 1.0, 1.0, 1.0],
        background_color: [0.0, 0.0, 0.0, 1.0],
        source_byte_len: 256,
    };
    let submission = test_submission([subpixel]);
    let mut frame = test_frame(
        submission,
        vec![test_source_image(subpixel, vec![255; 256])],
        1,
        0,
        0,
    );
    frame.background_composite_glyph_count = 1;
    frame.missing_background_composite_glyph_count = 1;
    let report = frame.prepare_report();

    assert!(!frame.supports_native_submission());
    assert_eq!(
        frame.atlas_storage_format(),
        Some(GlyphAtlasStorageFormat::Rgba8Unorm)
    );
    assert!(report.requires_background_composite);
    assert!(!report.background_composite_replacement_ready);
    assert_eq!(
        report.native_degradation_reason,
        Some(NativeBitmapAtlasDegradationReason::MissingBackgroundCompositeInput)
    );
    assert_eq!(report.background_composite_glyph_count, 1);
    assert_eq!(report.missing_background_composite_glyph_count, 1);
    assert!(!report.native_submission_ready);
    assert_eq!(report.storage_submission_count, 1);
    assert_eq!(report.storage_submission_visible_glyph_count, 1);
    assert!(!report.mixed_storage_replacement_ready);
    assert_eq!(report.source_image_count, 1);
}

#[test]
fn native_bitmap_atlas_frame_supports_native_submission_for_known_subpixel_background_composite() {
    let subpixel = GlyphAtlasBitmapSource {
        raster_key: None,
        format: GlyphAtlasFormat::SubpixelMask,
        content_size: UVec2::new(8, 8),
        screen_rect: GlyphAtlasScreenRect::new(18.0, 4.0, 8.0, 8.0),
        foreground_color: [1.0, 1.0, 1.0, 1.0],
        background_color: [0.18, 0.2, 0.22, 1.0],
        source_byte_len: 256,
    };
    let submission = test_submission([subpixel]);
    let mut frame = test_frame(
        submission,
        vec![test_source_image(subpixel, vec![255; 256])],
        1,
        0,
        0,
    );
    frame.background_composite_glyph_count = 1;
    let report = frame.prepare_report();

    assert!(frame.supports_native_submission());
    assert_eq!(
        frame.atlas_storage_format(),
        Some(GlyphAtlasStorageFormat::Rgba8Unorm)
    );
    assert!(report.requires_background_composite);
    assert!(report.background_composite_replacement_ready);
    assert_eq!(report.background_composite_glyph_count, 1);
    assert_eq!(report.missing_background_composite_glyph_count, 0);
    assert!(report.native_submission_ready);
    assert_eq!(report.storage_submission_count, 1);
    assert_eq!(report.storage_submission_visible_glyph_count, 1);
}
