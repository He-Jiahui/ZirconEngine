use super::*;
use crate::text::InstancedFaceId;
use crate::text::atlas::{
    GlyphAtlasBitmapUploadSourceBytes, GlyphAtlasRect, GlyphHintingMode, GlyphRasterKey,
    GlyphSmoothingMode, SyntheticGlyphStyle, glyph_atlas_bitmap_page_shadow_commit,
    glyph_atlas_bitmap_render_submission_plan,
    glyph_atlas_bitmap_render_submission_plan_with_atlas,
};

#[test]
fn native_bitmap_atlas_interleaved_storage_uses_one_canonical_frame_plan() {
    let [alpha, color, later_alpha] = interleaved_sources();
    let frame = interleaved_frame(alpha, color, later_alpha);

    let report = frame.prepare_report();

    assert_eq!(frame.canonical_frame_plan_count(), 1);
    assert_eq!(frame.storage_resource_count(), 2);
    assert_eq!(frame.ordered_draw_segment_count(), 3);
    assert_eq!(report.storage_submission_count, 1);
    assert_eq!(report.storage_submission_visible_glyph_count, 3);
}

#[test]
fn native_bitmap_atlas_canonical_frame_plan_prepares_interleaved_sources_once() {
    let [alpha, color, later_alpha] = interleaved_sources();
    let frame = interleaved_frame(alpha, color, later_alpha);
    let prepared_upload = frame.submission.prepared_upload(frame.source_bytes());

    assert!(!prepared_upload.has_failures());
    assert_eq!(frame.submission.run.upload_copies.len(), 3);
    assert!(
        prepared_upload
            .staged_uploads
            .uploads
            .iter()
            .any(|upload| upload.command.page_key.format == GlyphAtlasFormat::AlphaMask)
    );
    assert!(
        prepared_upload
            .staged_uploads
            .uploads
            .iter()
            .any(|upload| upload.command.page_key.format == GlyphAtlasFormat::Color)
    );

    let first_alpha = &frame.submission.run.glyphs[0];
    let second_alpha = &frame.submission.run.glyphs[2];
    assert_eq!(first_alpha.page_key, second_alpha.page_key);
    assert!(rectangles_are_disjoint(
        first_alpha.atlas_rect,
        second_alpha.atlas_rect
    ));
}

#[test]
fn native_bitmap_atlas_canonical_frame_plan_keeps_persistent_alpha_shadow_safe() {
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
    let prepared_upload = frame.submission.prepared_upload(frame.source_bytes());

    assert!(!prepared_upload.has_failures());
    assert_eq!(frame.canonical_frame_plan_count(), 1);
    assert!(
        prepared_upload
            .staged_uploads
            .uploads
            .iter()
            .any(|upload| upload.command.page_key.format == GlyphAtlasFormat::AlphaMask)
    );
}

fn interleaved_sources() -> [GlyphAtlasBitmapSource; 3] {
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
        format: GlyphAtlasFormat::Color,
        screen_rect: GlyphAtlasScreenRect::new(18.0, 4.0, 8.0, 8.0),
        source_byte_len: 256,
        ..alpha
    };
    let later_alpha = GlyphAtlasBitmapSource {
        screen_rect: GlyphAtlasScreenRect::new(30.0, 4.0, 8.0, 8.0),
        ..alpha
    };
    [alpha, color, later_alpha]
}

fn interleaved_frame(
    alpha: GlyphAtlasBitmapSource,
    color: GlyphAtlasBitmapSource,
    later_alpha: GlyphAtlasBitmapSource,
) -> NativeBitmapAtlasFrame {
    test_frame(
        test_submission([alpha, color, later_alpha]),
        vec![
            test_source_image(alpha, vec![255; 64]),
            test_source_image(color, vec![255; 256]),
            test_source_image(later_alpha, vec![127; 64]),
        ],
        3,
        0,
        0,
    )
}

fn rectangles_are_disjoint(left: GlyphAtlasRect, right: GlyphAtlasRect) -> bool {
    left.x.saturating_add(left.width) <= right.x
        || right.x.saturating_add(right.width) <= left.x
        || left.y.saturating_add(left.height) <= right.y
        || right.y.saturating_add(right.height) <= left.y
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
