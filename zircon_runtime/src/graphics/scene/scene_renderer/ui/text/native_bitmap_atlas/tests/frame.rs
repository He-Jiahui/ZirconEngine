use super::*;
use crate::graphics::text::raster::SwashRasterRequest;

const TEST_FRAME_FONT_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/fonts/FiraSans-Regular.ttf"
));

#[test]
fn native_bitmap_atlas_frame_replaces_glyphon_only_when_alpha_sources_cover_visible_glyphs() {
    let source = GlyphAtlasBitmapSource {
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

    assert!(frame.replaces_glyphon());
    assert_eq!(frame.atlas_layer_count(), 1);
    assert_eq!(
        frame.atlas_storage_format(),
        Some(GlyphAtlasStorageFormat::R8Unorm)
    );
    assert_eq!(frame.source_bytes()[0].bytes.len(), 64);
    assert_eq!(
        frame.prepare_report(),
        NativeBitmapAtlasPrepareReport {
            frame_index: TEST_BITMAP_ATLAS_FRAME_INDEX,
            visible_raster_glyph_count: 1,
            source_image_count: 1,
            missing_raster_image_count: 0,
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
            glyphon_fallback_reason: None,
            first_frame_degradation: None,
            replaces_glyphon: true,
            submission: frame.submission.submission_report(),
        }
    );

    let unsupported = NativeBitmapAtlasFrame {
        unsupported_glyph_count: 1,
        ..frame
    };
    assert!(!unsupported.replaces_glyphon());
}

#[test]
fn native_bitmap_atlas_frame_keeps_glyphon_when_raster_image_is_missing() {
    let source = GlyphAtlasBitmapSource {
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
fn native_bitmap_atlas_frame_reuses_approximate_bucket_while_exact_worker_is_pending() {
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
    let requested_key = text_area
        .buffer
        .layout_runs()
        .flat_map(|run| run.glyphs.iter())
        .next()
        .expect("test text should shape one glyph")
        .physical((text_area.left, text_area.top), text_area.scale)
        .cache_key;
    let face_index = font_system
        .db()
        .face(requested_key.font_id)
        .expect("test font face should be available")
        .index as usize;
    let mut approximate_key = requested_key;
    approximate_key.y_bin = if requested_key.y_bin == SubpixelBin::Zero {
        SubpixelBin::One
    } else {
        SubpixelBin::Zero
    };
    let bitmap_text_area = NativeBitmapAtlasTextArea::new(&text_area, None);
    let worker_pool = TextRasterWorkerPool::new_without_workers_for_test(
        TextRasterWorkerPoolOptions::new(1).with_queue_depth(4),
    );
    let mut source_cache = NativeBitmapAtlasSourceCache::with_capacity(4);
    source_cache.insert_test_image(approximate_key, test_cached_image(7));
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
    assert_eq!(
        work.request,
        SwashRasterRequest::glyphon_cache_key(
            face_index,
            CacheKey {
                x_bin: SubpixelBin::Zero,
                ..requested_key
            }
        )
    );
}

#[test]
fn native_bitmap_atlas_prepare_report_carries_frame_index() {
    let source = GlyphAtlasBitmapSource {
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

    assert_eq!(frame.source_bytes()[0].face_epoch, 0);
    assert_eq!(frame.face_validity(), GlyphAtlasBitmapFaceValidity::Valid);
    assert_eq!(
        frame.storage_submissions()[0].face_validity(),
        GlyphAtlasBitmapFaceValidity::Valid
    );

    frame.face_epoch = 1;

    assert_eq!(
        frame.face_validity(),
        GlyphAtlasBitmapFaceValidity::Invalidated
    );
    assert_eq!(
        frame.storage_submissions()[0].face_validity(),
        GlyphAtlasBitmapFaceValidity::Invalidated
    );
}

#[test]
fn native_bitmap_atlas_frame_marks_contiguous_mixed_storage_ready_for_renderer_handoff() {
    let alpha = GlyphAtlasBitmapSource {
        format: GlyphAtlasFormat::AlphaMask,
        content_size: UVec2::new(8, 8),
        screen_rect: GlyphAtlasScreenRect::new(4.0, 4.0, 8.0, 8.0),
        foreground_color: [1.0, 1.0, 1.0, 1.0],
        background_color: [0.0, 0.0, 0.0, 1.0],
        source_byte_len: 64,
    };
    let color = GlyphAtlasBitmapSource {
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
    let storage_submissions = frame.storage_submissions();

    assert!(!frame.replaces_glyphon());
    assert_eq!(frame.atlas_storage_format(), None);
    assert!(report.mixed_atlas_storage_format);
    assert_eq!(report.storage_submission_count, 2);
    assert_eq!(report.storage_submission_visible_glyph_count, 2);
    assert!(report.mixed_storage_replacement_ready);
    assert!(!report.requires_background_composite);
    assert!(!report.replaces_glyphon);
    assert_eq!(report.submission.visible_glyph_count, 2);
    assert_eq!(storage_submissions.len(), 2);
    assert_eq!(
        storage_submissions[0].storage_format,
        GlyphAtlasStorageFormat::R8Unorm
    );
    assert_eq!(
        storage_submissions[1].storage_format,
        GlyphAtlasStorageFormat::Rgba8Unorm
    );
    assert_eq!(storage_submissions[0].source_bytes()[0].source_index, 0);
    assert_eq!(storage_submissions[0].source_bytes()[0].bytes.len(), 64);
    assert_eq!(storage_submissions[1].source_bytes()[0].source_index, 0);
    assert_eq!(storage_submissions[1].source_bytes()[0].bytes.len(), 256);
    assert_eq!(storage_submissions[0].visible_glyph_count(), 1);
    assert_eq!(storage_submissions[1].visible_glyph_count(), 1);
    assert_eq!(storage_submissions[0].atlas_layer_count(), 1);
    assert_eq!(storage_submissions[1].atlas_layer_count(), 1);
}

#[test]
fn native_bitmap_atlas_storage_submissions_inherit_persistent_frame_atlas() {
    let alpha = GlyphAtlasBitmapSource {
        format: GlyphAtlasFormat::AlphaMask,
        content_size: UVec2::new(8, 8),
        screen_rect: GlyphAtlasScreenRect::new(4.0, 4.0, 8.0, 8.0),
        foreground_color: [1.0, 1.0, 1.0, 1.0],
        background_color: [0.0, 0.0, 0.0, 1.0],
        source_byte_len: 64,
    };
    let color = GlyphAtlasBitmapSource {
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

    let storage_submissions = frame.storage_submissions();

    assert_eq!(frame_alpha_generation, 12);
    assert_eq!(frame_color_generation, 24);
    assert_eq!(storage_submissions.len(), 2);
    assert_eq!(
        storage_submissions[0]
            .submission
            .run
            .atlas
            .page(GlyphAtlasFormat::AlphaMask, 0)
            .map(|page| page.generation),
        Some(frame_alpha_generation.saturating_add(1))
    );
    assert_eq!(
        storage_submissions[1]
            .submission
            .run
            .atlas
            .page(GlyphAtlasFormat::Color, 0)
            .map(|page| page.generation),
        Some(frame_color_generation.saturating_add(1))
    );
}

#[test]
fn native_bitmap_atlas_frame_preserves_repeated_storage_order_without_glyphon() {
    let first_alpha = GlyphAtlasBitmapSource {
        format: GlyphAtlasFormat::AlphaMask,
        content_size: UVec2::new(8, 8),
        screen_rect: GlyphAtlasScreenRect::new(4.0, 4.0, 8.0, 8.0),
        foreground_color: [1.0, 1.0, 1.0, 1.0],
        background_color: [0.0, 0.0, 0.0, 1.0],
        source_byte_len: 64,
    };
    let color = GlyphAtlasBitmapSource {
        format: GlyphAtlasFormat::Color,
        content_size: UVec2::new(8, 8),
        screen_rect: GlyphAtlasScreenRect::new(18.0, 4.0, 8.0, 8.0),
        foreground_color: [1.0, 1.0, 1.0, 1.0],
        background_color: [0.0, 0.0, 0.0, 1.0],
        source_byte_len: 256,
    };
    let second_alpha = GlyphAtlasBitmapSource {
        screen_rect: GlyphAtlasScreenRect::new(30.0, 4.0, 8.0, 8.0),
        ..first_alpha
    };
    let submission = test_submission([first_alpha, color, second_alpha]);
    let frame = test_frame(
        submission,
        vec![
            test_source_image(first_alpha, vec![255; 64]),
            test_source_image(color, vec![255; 256]),
            test_source_image(second_alpha, vec![127; 64]),
        ],
        3,
        0,
        0,
    );
    let report = frame.prepare_report();
    let storage_submissions = frame.storage_submissions();

    assert!(report.mixed_atlas_storage_format);
    assert_eq!(report.storage_submission_count, 3);
    assert_eq!(report.storage_submission_visible_glyph_count, 3);
    assert!(report.mixed_storage_replacement_ready);
    assert!(!report.replaces_glyphon);
    assert!(!frame.replaces_glyphon());
    assert_eq!(
        native_bitmap_atlas_handoff_for_report(&report),
        NativeBitmapAtlasHandoff::MixedStorageReplacement
    );
    assert_eq!(report.glyphon_fallback_reason, None);
    assert_eq!(storage_submissions.len(), 3);
    assert_eq!(
        storage_submissions
            .iter()
            .map(|submission| submission.storage_format)
            .collect::<Vec<_>>(),
        vec![
            GlyphAtlasStorageFormat::R8Unorm,
            GlyphAtlasStorageFormat::Rgba8Unorm,
            GlyphAtlasStorageFormat::R8Unorm,
        ]
    );
    assert!(storage_submissions
        .iter()
        .all(|submission| submission.visible_glyph_count() == 1));
    assert_eq!(storage_submissions[0].source_bytes()[0].bytes[0], 255);
    assert_eq!(storage_submissions[1].source_bytes()[0].bytes[0], 255);
    assert_eq!(storage_submissions[2].source_bytes()[0].bytes[0], 127);
}

#[test]
fn native_bitmap_atlas_frame_replaces_glyphon_for_single_color_storage_submission() {
    let color = GlyphAtlasBitmapSource {
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

    assert!(frame.replaces_glyphon());
    assert_eq!(
        frame.atlas_storage_format(),
        Some(GlyphAtlasStorageFormat::Rgba8Unorm)
    );
    assert!(!report.mixed_atlas_storage_format);
    assert_eq!(report.storage_submission_count, 1);
    assert_eq!(report.storage_submission_visible_glyph_count, 1);
    assert!(!report.mixed_storage_replacement_ready);
    assert!(!report.requires_background_composite);
    assert!(report.replaces_glyphon);
}

#[test]
fn native_bitmap_atlas_frame_keeps_glyphon_for_subpixel_background_composite() {
    let subpixel = GlyphAtlasBitmapSource {
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

    assert!(!frame.replaces_glyphon());
    assert_eq!(
        frame.atlas_storage_format(),
        Some(GlyphAtlasStorageFormat::Rgba8Unorm)
    );
    assert!(report.requires_background_composite);
    assert!(!report.background_composite_replacement_ready);
    assert_eq!(
        report.glyphon_fallback_reason,
        Some(NativeBitmapAtlasGlyphonFallbackReason::MissingBackgroundCompositeInput)
    );
    assert_eq!(report.background_composite_glyph_count, 1);
    assert_eq!(report.missing_background_composite_glyph_count, 1);
    assert!(!report.replaces_glyphon);
    assert_eq!(report.storage_submission_count, 1);
    assert_eq!(report.storage_submission_visible_glyph_count, 1);
    assert!(!report.mixed_storage_replacement_ready);
    assert_eq!(report.source_image_count, 1);
}

#[test]
fn native_bitmap_atlas_frame_replaces_glyphon_for_known_subpixel_background_composite() {
    let subpixel = GlyphAtlasBitmapSource {
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

    assert!(frame.replaces_glyphon());
    assert_eq!(
        frame.atlas_storage_format(),
        Some(GlyphAtlasStorageFormat::Rgba8Unorm)
    );
    assert!(report.requires_background_composite);
    assert!(report.background_composite_replacement_ready);
    assert_eq!(report.background_composite_glyph_count, 1);
    assert_eq!(report.missing_background_composite_glyph_count, 0);
    assert!(report.replaces_glyphon);
    assert_eq!(report.storage_submission_count, 1);
    assert_eq!(report.storage_submission_visible_glyph_count, 1);
}
