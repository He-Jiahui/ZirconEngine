use super::*;
use crate::text::InstancedFaceId;
use crate::text::atlas::{
    GlyphAtlasBitmapUploadSourceBytes, GlyphAtlasUploadMode, GlyphHintingMode, GlyphRasterKey,
    GlyphSmoothingMode, SyntheticGlyphStyle, glyph_atlas_bitmap_page_shadow_commit,
    glyph_atlas_bitmap_render_submission_plan,
    glyph_atlas_bitmap_render_submission_plan_with_atlas,
};

#[test]
fn native_bitmap_atlas_storage_split_does_not_promote_later_alpha_to_full_page() {
    let first_alpha = GlyphAtlasBitmapSource {
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
    let second_alpha = GlyphAtlasBitmapSource {
        raster_key: None,
        format: GlyphAtlasFormat::AlphaMask,
        content_size: UVec2::new(56, 64),
        screen_rect: GlyphAtlasScreenRect::new(30.0, 4.0, 56.0, 64.0),
        foreground_color: [1.0, 1.0, 1.0, 1.0],
        background_color: [0.0, 0.0, 0.0, 1.0],
        source_byte_len: 56 * 64,
    };
    let frame = test_frame(
        test_submission([first_alpha, color, second_alpha]),
        vec![
            test_source_image(first_alpha, vec![0xA5; 64]),
            test_source_image(color, vec![0xCC; 256]),
            test_source_image(second_alpha, vec![0x5A; 56 * 64]),
        ],
        3,
        0,
        0,
    );
    let storage_submissions = frame.storage_submissions();

    assert_eq!(storage_submissions.len(), 3);
    let first_alpha_submission = &storage_submissions[0].submission;
    let second_alpha_submission = &storage_submissions[2].submission;
    assert_eq!(
        first_alpha_submission.run.glyphs[0].page_key,
        second_alpha_submission.run.glyphs[0].page_key,
        "the split alpha runs must share one page for this replay regression",
    );
    assert_eq!(first_alpha_submission.run.upload_commands.len(), 1);
    assert_eq!(second_alpha_submission.run.upload_commands.len(), 1);
    assert_eq!(
        second_alpha_submission.run.upload_commands[0].mode,
        GlyphAtlasUploadMode::PartialRect,
    );
    assert_eq!(
        second_alpha_submission.run.upload_commands[0].rect,
        second_alpha_submission.run.upload_copies[0].atlas_rect,
    );

    let second_prepared =
        second_alpha_submission.prepared_upload(storage_submissions[2].source_bytes());
    assert!(!second_prepared.has_failures());
    assert_eq!(
        second_prepared.staging.pages[0].target_rect,
        second_alpha_submission.run.upload_copies[0].atlas_rect,
    );
    assert_eq!(second_prepared.staging.pages[0].bytes[0], 0x5A);
}

#[test]
fn native_bitmap_atlas_storage_split_does_not_replay_stale_shadow_over_new_alpha() {
    let cached_alpha = persistent_alpha_source(301, UVec2::new(8, 8), 4.0, 4.0);
    let first_submission = glyph_atlas_bitmap_render_submission_plan(
        [cached_alpha],
        UVec2::new(64, 64),
        70,
        1,
        test_viewport_size(),
        test_clip_rect(),
    );
    let first_prepared =
        first_submission.prepared_upload([GlyphAtlasBitmapUploadSourceBytes::new(0, &[0xD0; 64])]);
    let first_shadow_commit =
        glyph_atlas_bitmap_page_shadow_commit(&first_submission.run, first_prepared, true);
    let mut atlas = first_submission.run.atlas;
    atlas.commit_bitmap_page_shadow(first_shadow_commit);

    let new_alpha = persistent_alpha_source(302, UVec2::new(8, 8), 18.0, 4.0);
    let color = GlyphAtlasBitmapSource {
        raster_key: None,
        format: GlyphAtlasFormat::Color,
        content_size: UVec2::new(8, 8),
        screen_rect: GlyphAtlasScreenRect::new(30.0, 4.0, 8.0, 8.0),
        foreground_color: [1.0, 1.0, 1.0, 1.0],
        background_color: [0.0, 0.0, 0.0, 1.0],
        source_byte_len: 256,
    };
    let later_alpha = persistent_alpha_source(303, UVec2::new(64, 48), 42.0, 4.0);
    let submission = glyph_atlas_bitmap_render_submission_plan_with_atlas(
        atlas,
        [cached_alpha, new_alpha, color, later_alpha],
        UVec2::new(64, 64),
        71,
        1,
        test_viewport_size(),
        test_clip_rect(),
    );
    let frame = test_frame(
        submission,
        vec![
            test_source_image(cached_alpha, vec![0xD0; 64]),
            test_source_image(new_alpha, vec![0xA5; 64]),
            test_source_image(color, vec![0xCC; 256]),
            test_source_image(later_alpha, vec![0x5A; 64 * 48]),
        ],
        4,
        0,
        0,
    );
    let storage_submissions = frame.storage_submissions();

    assert_eq!(storage_submissions.len(), 3);
    let first_alpha_submission = &storage_submissions[0].submission;
    let later_alpha_submission = &storage_submissions[2].submission;
    assert_eq!(
        first_alpha_submission.run.glyphs[1].page_key,
        later_alpha_submission.run.glyphs[0].page_key,
        "the new alpha slots must share a page across separated storage splits",
    );
    assert_eq!(later_alpha_submission.run.upload_commands.len(), 1);
    assert_eq!(
        later_alpha_submission.run.upload_commands[0].mode,
        GlyphAtlasUploadMode::PartialRect,
    );
    assert_eq!(
        later_alpha_submission.run.upload_commands[0].rect,
        later_alpha_submission.run.upload_copies[0].atlas_rect,
    );

    let later_prepared =
        later_alpha_submission.prepared_upload(storage_submissions[2].source_bytes());
    assert!(!later_prepared.has_failures());
    assert_eq!(
        later_prepared.staging.pages[0].target_rect,
        later_alpha_submission.run.upload_copies[0].atlas_rect,
    );
    assert_eq!(later_prepared.staging.pages[0].bytes[0], 0x5A);
}

#[test]
fn native_bitmap_atlas_storage_submissions_keep_repeated_format_slots_disjoint() {
    let first_alpha = GlyphAtlasBitmapSource {
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
    let second_alpha = GlyphAtlasBitmapSource {
        screen_rect: GlyphAtlasScreenRect::new(30.0, 4.0, 8.0, 8.0),
        ..first_alpha
    };
    let frame = test_frame(
        test_submission([first_alpha, color, second_alpha]),
        vec![
            test_source_image(first_alpha, vec![255; 64]),
            test_source_image(color, vec![255; 256]),
            test_source_image(second_alpha, vec![127; 64]),
        ],
        3,
        0,
        0,
    );
    let storage_submissions = frame.storage_submissions();

    assert_eq!(storage_submissions.len(), 3);
    assert_eq!(
        storage_submissions
            .iter()
            .map(|submission| submission.atlas_format)
            .collect::<Vec<_>>(),
        vec![
            GlyphAtlasFormat::AlphaMask,
            GlyphAtlasFormat::Color,
            GlyphAtlasFormat::AlphaMask,
        ]
    );
    let first_alpha_glyph = &storage_submissions[0].submission.run.glyphs[0];
    let second_alpha_glyph = &storage_submissions[2].submission.run.glyphs[0];
    assert_eq!(
        first_alpha_glyph.atlas_rect,
        frame.submission.run.glyphs[0].atlas_rect
    );
    assert_eq!(
        second_alpha_glyph.atlas_rect,
        frame.submission.run.glyphs[2].atlas_rect
    );
    assert_eq!(first_alpha_glyph.page_key, second_alpha_glyph.page_key);
    assert!(
        first_alpha_glyph
            .atlas_rect
            .x
            .saturating_add(first_alpha_glyph.atlas_rect.width)
            <= second_alpha_glyph.atlas_rect.x
            || second_alpha_glyph
                .atlas_rect
                .x
                .saturating_add(second_alpha_glyph.atlas_rect.width)
                <= first_alpha_glyph.atlas_rect.x
            || first_alpha_glyph
                .atlas_rect
                .y
                .saturating_add(first_alpha_glyph.atlas_rect.height)
                <= second_alpha_glyph.atlas_rect.y
            || second_alpha_glyph
                .atlas_rect
                .y
                .saturating_add(second_alpha_glyph.atlas_rect.height)
                <= first_alpha_glyph.atlas_rect.y
    );
    assert_eq!(storage_submissions[0].source_bytes()[0].bytes[0], 255);
    assert_eq!(storage_submissions[1].source_bytes()[0].bytes[0], 255);
    assert_eq!(storage_submissions[2].source_bytes()[0].bytes[0], 127);
    assert_eq!(
        storage_submissions
            .iter()
            .map(|submission| submission.submission.run.upload_copies.len())
            .collect::<Vec<_>>(),
        vec![1, 1, 1]
    );

    let prepared_uploads = storage_submissions
        .iter()
        .map(|submission| {
            submission
                .submission
                .prepared_upload(submission.source_bytes())
        })
        .collect::<Vec<_>>();
    assert!(
        prepared_uploads
            .iter()
            .all(|prepared_upload| !prepared_upload.has_failures())
    );
    assert_eq!(
        prepared_uploads
            .iter()
            .map(|prepared_upload| prepared_upload.staging.pages.len())
            .collect::<Vec<_>>(),
        vec![1, 1, 1]
    );
    assert_eq!(
        prepared_uploads
            .iter()
            .map(|prepared_upload| prepared_upload.staged_uploads.uploads.len())
            .collect::<Vec<_>>(),
        vec![1, 1, 1]
    );

    for ((submission, prepared_upload), expected_byte) in storage_submissions
        .iter()
        .zip(&prepared_uploads)
        .zip([255, 255, 127])
    {
        let upload_copy = submission.submission.run.upload_copies[0];
        let staging_page = &prepared_upload.staging.pages[0];
        assert_eq!(staging_page.target_rect, upload_copy.atlas_rect);
        assert_eq!(staging_page.bytes[0], expected_byte);

        let staged_upload = prepared_upload.staged_uploads.uploads[0];
        assert_eq!(staged_upload.command.page_key, upload_copy.page_key);
        assert_eq!(staged_upload.command.rect, upload_copy.atlas_rect);
    }
}

fn persistent_alpha_source(
    glyph_id: u32,
    content_size: UVec2,
    x: f32,
    y: f32,
) -> GlyphAtlasBitmapSource {
    GlyphAtlasBitmapSource {
        raster_key: Some(GlyphRasterKey {
            face: InstancedFaceId(41),
            glyph_id,
            px_size_bucket: 16,
            subpixel_bin: 0,
            vertical_subpixel_bin: 0,
            format: GlyphAtlasFormat::AlphaMask,
            hinting: GlyphHintingMode::Full,
            smoothing: GlyphSmoothingMode::Grayscale,
            synthetic: SyntheticGlyphStyle::default(),
        }),
        format: GlyphAtlasFormat::AlphaMask,
        content_size,
        screen_rect: GlyphAtlasScreenRect::new(x, y, content_size.x as f32, content_size.y as f32),
        foreground_color: [1.0, 1.0, 1.0, 1.0],
        background_color: [0.0, 0.0, 0.0, 1.0],
        source_byte_len: content_size.x as usize * content_size.y as usize,
    }
}
