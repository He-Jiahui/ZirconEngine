use super::*;
use crate::text::atlas::{GlyphAtlasPageKey, GlyphAtlasPageSpec};

fn test_queued_glyph(
    source_index: usize,
    source: GlyphAtlasBitmapSource,
    retry_frame_index: u64,
) -> GlyphAtlasBitmapQueuedGlyph {
    GlyphAtlasBitmapQueuedGlyph {
        source_index,
        source,
        retry_frame_index,
    }
}

#[test]
fn native_bitmap_atlas_retry_frame_retries_visible_blocked_source_once() {
    let retry_source = GlyphAtlasBitmapSource {
        format: GlyphAtlasFormat::AlphaMask,
        content_size: UVec2::new(8, 8),
        screen_rect: GlyphAtlasScreenRect::new(8.0, 4.0, 8.0, 8.0),
        foreground_color: [0.8, 0.8, 0.8, 1.0],
        background_color: [0.0, 0.0, 0.0, 1.0],
        source_byte_len: 64,
    };
    let new_source = GlyphAtlasBitmapSource {
        screen_rect: GlyphAtlasScreenRect::new(24.0, 4.0, 8.0, 8.0),
        ..retry_source
    };
    let mut retry_state =
        GlyphAtlasBitmapRetryFrameState::with_blocked_glyphs([test_queued_glyph(
            37,
            retry_source,
            TEST_BITMAP_ATLAS_FRAME_INDEX,
        )]);

    let retry_frame = super::super::retry_frame::native_bitmap_atlas_retry_frame(
        &mut retry_state,
        GlyphAtlasSet::default(),
        vec![
            test_source_image(retry_source, vec![7; 64]),
            test_source_image(new_source, vec![11; 64]),
        ],
        TEST_BITMAP_ATLAS_FRAME_INDEX,
        test_viewport_size(),
        test_clip_rect(),
    );

    assert_eq!(retry_frame.retry_submission.retried_source_count, 1);
    assert_eq!(retry_frame.retry_submission.new_source_count, 1);
    assert_eq!(
        retry_frame.retry_submission.completed_retried_source_count,
        1
    );
    assert_eq!(retry_frame.retry_state.queued_blocked_glyph_count, 0);
    assert_eq!(retry_state.queued_blocked_glyphs(), &[]);
    assert_eq!(retry_frame.source_images[0].bytes, vec![7; 64]);
    assert_eq!(retry_frame.source_images[1].bytes, vec![11; 64]);
}

#[test]
fn native_bitmap_atlas_retry_frame_rebuilds_persistent_atlas_page() {
    let source = GlyphAtlasBitmapSource {
        format: GlyphAtlasFormat::AlphaMask,
        content_size: UVec2::new(8, 8),
        screen_rect: GlyphAtlasScreenRect::new(8.0, 4.0, 8.0, 8.0),
        foreground_color: [0.8, 0.8, 0.8, 1.0],
        background_color: [0.0, 0.0, 0.0, 1.0],
        source_byte_len: 64,
    };
    let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0);
    let atlas = GlyphAtlasSet::from_page(
        GlyphAtlasPageSpec::new(page_key, bitmap_atlas_page_size()).with_generation(7),
    );
    let mut retry_state = GlyphAtlasBitmapRetryFrameState::new();

    let retry_frame = super::super::retry_frame::native_bitmap_atlas_retry_frame(
        &mut retry_state,
        atlas,
        vec![test_source_image(source, vec![7; 64])],
        TEST_BITMAP_ATLAS_FRAME_INDEX,
        test_viewport_size(),
        test_clip_rect(),
    );

    assert_eq!(retry_frame.submission.run.rebuilt_pages, vec![page_key]);
    assert_eq!(retry_frame.submission.run.slot_invalidations.len(), 1);
    assert_eq!(
        retry_frame.submission.run.slot_invalidations[0].page_generation,
        8
    );
    assert_eq!(
        retry_frame
            .retry_submission
            .submission_report
            .slot_invalidation_count,
        1
    );
}

#[test]
fn native_bitmap_atlas_retry_frame_remaps_nonzero_visible_blocked_source() {
    let first_new_source = GlyphAtlasBitmapSource {
        format: GlyphAtlasFormat::AlphaMask,
        content_size: UVec2::new(8, 8),
        screen_rect: GlyphAtlasScreenRect::new(0.0, 4.0, 8.0, 8.0),
        foreground_color: [0.4, 0.4, 0.4, 1.0],
        background_color: [0.0, 0.0, 0.0, 0.0],
        source_byte_len: 64,
    };
    let retry_source = GlyphAtlasBitmapSource {
        format: GlyphAtlasFormat::AlphaMask,
        content_size: UVec2::new(8, 8),
        screen_rect: GlyphAtlasScreenRect::new(8.0, 4.0, 8.0, 8.0),
        foreground_color: [0.8, 0.8, 0.8, 1.0],
        background_color: [0.0, 0.0, 0.0, 0.0],
        source_byte_len: 64,
    };
    let last_new_source = GlyphAtlasBitmapSource {
        format: GlyphAtlasFormat::AlphaMask,
        content_size: UVec2::new(8, 8),
        screen_rect: GlyphAtlasScreenRect::new(16.0, 4.0, 8.0, 8.0),
        foreground_color: [0.9, 0.9, 0.9, 1.0],
        background_color: [0.0, 0.0, 0.0, 0.0],
        source_byte_len: 64,
    };
    let mut retry_state = GlyphAtlasBitmapRetryFrameState::new();
    retry_state.replace_blocked_glyphs([test_queued_glyph(
        0,
        retry_source,
        TEST_BITMAP_ATLAS_FRAME_INDEX,
    )]);

    let retry_frame = super::super::retry_frame::native_bitmap_atlas_retry_frame(
        &mut retry_state,
        GlyphAtlasSet::default(),
        vec![
            test_source_image(first_new_source, vec![3; 64]),
            test_source_image(retry_source, vec![7; 64]),
            test_source_image(last_new_source, vec![11; 64]),
        ],
        TEST_BITMAP_ATLAS_FRAME_INDEX,
        test_viewport_size(),
        test_clip_rect(),
    );

    assert_eq!(retry_frame.retry_submission.retried_source_count, 1);
    assert_eq!(retry_frame.retry_submission.new_source_count, 2);
    assert_eq!(retry_frame.source_images[0].bytes, vec![7; 64]);
    assert_eq!(retry_frame.source_images[1].bytes, vec![3; 64]);
    assert_eq!(retry_frame.source_images[2].bytes, vec![11; 64]);
}

#[test]
fn native_bitmap_atlas_retry_frame_discards_stale_blocked_source() {
    let stale_source = GlyphAtlasBitmapSource {
        format: GlyphAtlasFormat::AlphaMask,
        content_size: UVec2::new(8, 8),
        screen_rect: GlyphAtlasScreenRect::new(8.0, 4.0, 8.0, 8.0),
        foreground_color: [0.8, 0.8, 0.8, 1.0],
        background_color: [0.0, 0.0, 0.0, 1.0],
        source_byte_len: 64,
    };
    let visible_source = GlyphAtlasBitmapSource {
        screen_rect: GlyphAtlasScreenRect::new(48.0, 4.0, 8.0, 8.0),
        ..stale_source
    };
    let mut retry_state =
        GlyphAtlasBitmapRetryFrameState::with_blocked_glyphs([test_queued_glyph(
            13,
            stale_source,
            TEST_BITMAP_ATLAS_FRAME_INDEX,
        )]);

    let retry_frame = super::super::retry_frame::native_bitmap_atlas_retry_frame(
        &mut retry_state,
        GlyphAtlasSet::default(),
        vec![test_source_image(visible_source, vec![3; 64])],
        TEST_BITMAP_ATLAS_FRAME_INDEX,
        test_viewport_size(),
        test_clip_rect(),
    );

    assert_eq!(retry_frame.retry_submission.retried_source_count, 0);
    assert_eq!(retry_frame.retry_submission.new_source_count, 1);
    assert_eq!(retry_frame.retry_state.queued_blocked_glyph_count, 0);
    assert_eq!(retry_frame.discarded_stale_retry_glyph_count, 1);
    assert_eq!(retry_state.queued_blocked_glyphs(), &[]);
    assert_eq!(retry_frame.source_images[0].source, visible_source);
}

#[test]
fn native_bitmap_atlas_retry_frame_does_not_reuse_one_blocked_source_twice() {
    let retry_source = GlyphAtlasBitmapSource {
        format: GlyphAtlasFormat::AlphaMask,
        content_size: UVec2::new(8, 8),
        screen_rect: GlyphAtlasScreenRect::new(8.0, 4.0, 8.0, 8.0),
        foreground_color: [0.8, 0.8, 0.8, 1.0],
        background_color: [0.0, 0.0, 0.0, 1.0],
        source_byte_len: 64,
    };
    let mut retry_state =
        GlyphAtlasBitmapRetryFrameState::with_blocked_glyphs([test_queued_glyph(
            13,
            retry_source,
            TEST_BITMAP_ATLAS_FRAME_INDEX,
        )]);

    let retry_frame = super::super::retry_frame::native_bitmap_atlas_retry_frame(
        &mut retry_state,
        GlyphAtlasSet::default(),
        vec![
            test_source_image(retry_source, vec![3; 64]),
            test_source_image(retry_source, vec![7; 64]),
        ],
        TEST_BITMAP_ATLAS_FRAME_INDEX,
        test_viewport_size(),
        test_clip_rect(),
    );

    assert_eq!(retry_frame.retry_submission.retried_source_count, 1);
    assert_eq!(retry_frame.retry_submission.new_source_count, 1);
    assert_eq!(retry_frame.discarded_stale_retry_glyph_count, 0);
    assert_eq!(retry_frame.source_images[0].bytes, vec![3; 64]);
    assert_eq!(retry_frame.source_images[1].bytes, vec![7; 64]);
}

#[test]
fn native_bitmap_atlas_retry_frame_reports_face_invalidated_blocked_source() {
    let invalidated_source = GlyphAtlasBitmapSource {
        format: GlyphAtlasFormat::AlphaMask,
        content_size: UVec2::new(8, 8),
        screen_rect: GlyphAtlasScreenRect::new(8.0, 4.0, 8.0, 8.0),
        foreground_color: [0.8, 0.8, 0.8, 1.0],
        background_color: [0.0, 0.0, 0.0, 1.0],
        source_byte_len: 64,
    };
    let mut retry_state =
        GlyphAtlasBitmapRetryFrameState::with_blocked_glyphs([test_queued_glyph(
            13,
            invalidated_source,
            TEST_BITMAP_ATLAS_FRAME_INDEX,
        )]);
    retry_state.discard_all_for_face_invalidation();

    let retry_frame = super::super::retry_frame::native_bitmap_atlas_retry_frame(
        &mut retry_state,
        GlyphAtlasSet::default(),
        vec![test_source_image(invalidated_source, vec![3; 64])],
        TEST_BITMAP_ATLAS_FRAME_INDEX,
        test_viewport_size(),
        test_clip_rect(),
    );

    assert_eq!(retry_frame.retry_submission.retried_source_count, 0);
    assert_eq!(retry_frame.retry_submission.new_source_count, 1);
    assert_eq!(retry_frame.retry_state.invalidated_blocked_glyph_count, 1);
    assert_eq!(retry_frame.retry_state.queued_blocked_glyph_count, 0);
    assert_eq!(retry_state.report().invalidated_blocked_glyph_count, 0);
    assert_eq!(retry_frame.source_images[0].source, invalidated_source);
}
