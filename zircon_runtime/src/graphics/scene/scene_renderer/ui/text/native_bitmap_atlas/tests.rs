use super::*;

fn test_viewport_size() -> UVec2 {
    UVec2::new(128, 64)
}

fn test_clip_rect() -> GlyphAtlasScreenRect {
    GlyphAtlasScreenRect::new(0.0, 0.0, 128.0, 64.0)
}

fn test_submission<I>(sources: I) -> GlyphAtlasBitmapRenderSubmissionPlan
where
    I: IntoIterator<Item = GlyphAtlasBitmapSource>,
{
    glyph_atlas_bitmap_render_submission_plan(
        sources,
        UVec2::new(64, 64),
        BITMAP_ATLAS_FRAME_INDEX,
        GLYPH_ATLAS_DEFAULT_MAX_PAGES_PER_FORMAT,
        test_viewport_size(),
        test_clip_rect(),
    )
}

fn test_source_image(
    source: GlyphAtlasBitmapSource,
    bytes: Vec<u8>,
) -> NativeBitmapAtlasSourceImage {
    NativeBitmapAtlasSourceImage { source, bytes }
}

fn test_frame(
    submission: GlyphAtlasBitmapRenderSubmissionPlan,
    source_images: Vec<NativeBitmapAtlasSourceImage>,
    visible_raster_glyph_count: usize,
    unsupported_glyph_count: usize,
    clipped_glyph_count: usize,
) -> NativeBitmapAtlasFrame {
    NativeBitmapAtlasFrame {
        submission,
        source_images,
        viewport_size: test_viewport_size(),
        clip_rect: test_clip_rect(),
        visible_raster_glyph_count,
        unsupported_glyph_count,
        clipped_glyph_count,
    }
}

#[test]
fn native_bitmap_atlas_source_uses_glyphon_bitmap_pixel_rect() {
    let image = NativeBitmapGlyphImage {
        x: 20,
        y: 7,
        line_y: 14.2,
        top: 11,
        left: -2,
        width: 9,
        height: 13,
        format: GlyphAtlasFormat::AlphaMask,
        scale_factor: 1.0,
        source_byte_len: 117,
        foreground_color: [0.9, 0.8, 0.7, 1.0],
        background_color: [0.0, 0.0, 0.0, 1.0],
    };
    let clipped = native_bitmap_atlas_screen_rect(
        image.x,
        image.y,
        image.line_y,
        image.top,
        image.left,
        image.width,
        image.height,
        image.scale_factor,
    );
    let clipped_source = native_bitmap_atlas_source_from_image(image, clipped, vec![7; 117])
        .expect("alpha glyph image should produce an atlas source");
    let source = clipped_source.source;

    assert_eq!(source.format, GlyphAtlasFormat::AlphaMask);
    assert_eq!(source.content_size, UVec2::new(9, 13));
    assert_eq!(
        source.screen_rect,
        GlyphAtlasScreenRect::new(18.0, 10.0, 9.0, 13.0)
    );
    assert_eq!(source.source_byte_len, 117);
    assert_eq!(source.foreground_color, [0.9, 0.8, 0.7, 1.0]);
    assert_eq!(clipped_source.bytes.len(), 117);
    assert!(!clipped_source.was_clipped);
}

#[test]
fn native_bitmap_atlas_source_crops_alpha_rows_to_text_bounds() {
    let image = NativeBitmapGlyphImage {
        x: 20,
        y: 7,
        line_y: 14.2,
        top: 11,
        left: -2,
        width: 9,
        height: 4,
        format: GlyphAtlasFormat::AlphaMask,
        scale_factor: 1.0,
        source_byte_len: 36,
        foreground_color: [0.9, 0.8, 0.7, 1.0],
        background_color: [0.0, 0.0, 0.0, 1.0],
    };
    let source_bytes = (0..36).collect::<Vec<u8>>();
    let bounds = TextBounds {
        left: 20,
        top: 11,
        right: 25,
        bottom: 13,
    };
    let screen_rect = native_bitmap_atlas_screen_rect(
        image.x,
        image.y,
        image.line_y,
        image.top,
        image.left,
        image.width,
        image.height,
        image.scale_factor,
    );
    let clipped = text_bounds_clipped_screen_rect(bounds, screen_rect)
        .expect("text bounds should keep the visible glyph slice");

    let clipped_source = native_bitmap_atlas_source_from_image(image, clipped, source_bytes)
        .expect("text-bounds clipped alpha glyph should still produce an atlas source");

    assert_eq!(clipped_source.source.content_size, UVec2::new(5, 2));
    assert_eq!(
        clipped_source.source.screen_rect,
        GlyphAtlasScreenRect::new(20.0, 11.0, 5.0, 2.0)
    );
    assert_eq!(clipped_source.source.source_byte_len, 10);
    assert_eq!(
        clipped_source.bytes,
        vec![11, 12, 13, 14, 15, 20, 21, 22, 23, 24]
    );
    assert!(clipped_source.was_clipped);
}

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
            visible_raster_glyph_count: 1,
            source_image_count: 1,
            unsupported_glyph_count: 0,
            clipped_glyph_count: 0,
            atlas_storage_format: Some(GlyphAtlasStorageFormat::R8Unorm),
            mixed_atlas_storage_format: false,
            storage_submission_count: 1,
            storage_submission_visible_glyph_count: 1,
            mixed_storage_replacement_ready: false,
            requires_background_composite: false,
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
fn native_bitmap_atlas_frame_keeps_glyphon_when_mixed_storage_order_would_change() {
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

    assert!(report.mixed_atlas_storage_format);
    assert_eq!(report.storage_submission_count, 2);
    assert_eq!(report.storage_submission_visible_glyph_count, 3);
    assert!(!report.mixed_storage_replacement_ready);
    assert!(!report.replaces_glyphon);
    assert!(!frame.replaces_glyphon());
}

#[test]
fn native_bitmap_atlas_source_preserves_color_rgba_rows_and_untints_foreground() {
    let image = NativeBitmapGlyphImage {
        x: 10,
        y: 2,
        line_y: 8.0,
        top: 6,
        left: 1,
        width: 3,
        height: 2,
        format: GlyphAtlasFormat::Color,
        scale_factor: 1.0,
        source_byte_len: 24,
        foreground_color: native_bitmap_atlas_foreground_color(
            GlyphAtlasFormat::Color,
            [0.2, 0.4, 0.6, 0.8],
        ),
        background_color: [0.0, 0.0, 0.0, 1.0],
    };
    let source_bytes = (0..24).collect::<Vec<u8>>();
    let screen_rect = native_bitmap_atlas_screen_rect(
        image.x,
        image.y,
        image.line_y,
        image.top,
        image.left,
        image.width,
        image.height,
        image.scale_factor,
    );
    let clipped = GlyphAtlasScreenRect::new(screen_rect.x + 1.0, screen_rect.y, 2.0, 2.0);

    let clipped_source = native_bitmap_atlas_source_from_image(image, clipped, source_bytes)
        .expect("color glyph should produce RGBA atlas source bytes");

    assert_eq!(clipped_source.source.format, GlyphAtlasFormat::Color);
    assert_eq!(clipped_source.source.content_size, UVec2::new(2, 2));
    assert_eq!(clipped_source.source.source_byte_len, 16);
    assert_eq!(clipped_source.source.foreground_color, [1.0, 1.0, 1.0, 1.0]);
    assert_eq!(
        clipped_source.bytes,
        vec![4, 5, 6, 7, 8, 9, 10, 11, 16, 17, 18, 19, 20, 21, 22, 23]
    );
    assert!(clipped_source.was_clipped);
}

#[test]
fn native_bitmap_atlas_format_maps_rgba_contents_to_atlas_formats() {
    assert_eq!(
        native_bitmap_atlas_format(SwashContent::Mask),
        Some(GlyphAtlasFormat::AlphaMask)
    );
    assert_eq!(
        native_bitmap_atlas_format(SwashContent::Color),
        Some(GlyphAtlasFormat::Color)
    );
    assert_eq!(
        native_bitmap_atlas_format(SwashContent::SubpixelMask),
        Some(GlyphAtlasFormat::SubpixelMask)
    );
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
    let frame = test_frame(
        submission,
        vec![test_source_image(subpixel, vec![255; 256])],
        1,
        0,
        0,
    );
    let report = frame.prepare_report();

    assert!(!frame.replaces_glyphon());
    assert_eq!(
        frame.atlas_storage_format(),
        Some(GlyphAtlasStorageFormat::Rgba8Unorm)
    );
    assert!(report.requires_background_composite);
    assert!(!report.replaces_glyphon);
    assert_eq!(report.storage_submission_count, 1);
    assert_eq!(report.storage_submission_visible_glyph_count, 1);
    assert!(!report.mixed_storage_replacement_ready);
    assert_eq!(report.source_image_count, 1);
}

#[test]
fn native_bitmap_atlas_frame_replaces_glyphon_when_text_bounds_clip_alpha_source() {
    let inside = GlyphAtlasScreenRect::new(10.0, 8.0, 24.0, 12.0);
    let bounds = TextBounds {
        left: 8,
        top: 6,
        right: 40,
        bottom: 24,
    };

    assert_eq!(
        text_bounds_clipped_screen_rect(bounds, inside),
        Some(inside)
    );
    assert_eq!(
        text_bounds_clipped_screen_rect(bounds, GlyphAtlasScreenRect::new(10.0, 8.0, 40.0, 12.0)),
        Some(GlyphAtlasScreenRect::new(10.0, 8.0, 30.0, 12.0))
    );

    let clipped_source = GlyphAtlasBitmapSource {
        format: GlyphAtlasFormat::AlphaMask,
        content_size: UVec2::new(30, 12),
        screen_rect: GlyphAtlasScreenRect::new(10.0, 8.0, 30.0, 12.0),
        foreground_color: [1.0, 1.0, 1.0, 1.0],
        background_color: [0.0, 0.0, 0.0, 1.0],
        source_byte_len: 360,
    };
    let submission = test_submission([clipped_source]);
    let frame = test_frame(
        submission,
        vec![test_source_image(clipped_source, vec![255; 360])],
        1,
        0,
        1,
    );

    assert!(frame.replaces_glyphon());
}
