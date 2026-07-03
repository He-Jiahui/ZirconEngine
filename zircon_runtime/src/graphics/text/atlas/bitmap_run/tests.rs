use super::super::render_batch::glyph_atlas_draw_batch_plan;
use super::super::render_contract::GlyphAtlasBlendMode;
use super::super::render_plan::GlyphAtlasScreenRect;
use super::super::{
    GlyphAtlasFormat, GlyphAtlasPageKey, GlyphAtlasRect, GlyphAtlasSamplingSemantics,
    GlyphAtlasUploadMode,
};
use super::*;
use crate::core::math::UVec2;

#[test]
fn render_text_atlas_bitmap_run_allocates_bitmap_formats_to_distinct_pages() {
    let plan = glyph_atlas_bitmap_run_plan_with_padding(
        [
            source(GlyphAtlasFormat::AlphaMask, UVec2::new(8, 4), 8.0, 32),
            source(GlyphAtlasFormat::SubpixelMask, UVec2::new(6, 4), 22.0, 96),
            source(GlyphAtlasFormat::Color, UVec2::new(4, 4), 36.0, 64),
        ],
        UVec2::new(32, 32),
        7,
        1,
        2,
    );

    assert!(plan.allocation_failures.is_empty());
    assert_eq!(plan.atlas.page_count(), 3);
    assert_eq!(plan.glyphs.len(), 3);
    assert_eq!(plan.draw_glyphs.len(), 3);
    assert_eq!(plan.dirty_pages.len(), 3);
    assert_eq!(plan.upload_commands.len(), 3);
    assert!(plan.atlas.page(GlyphAtlasFormat::AlphaMask, 0).is_some());
    assert!(plan.atlas.page(GlyphAtlasFormat::SubpixelMask, 0).is_some());
    assert!(plan.atlas.page(GlyphAtlasFormat::Color, 0).is_some());

    assert_eq!(
        plan.glyphs[0].page_key,
        GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0)
    );
    assert_eq!(
        plan.glyphs[1].page_key,
        GlyphAtlasPageKey::new(GlyphAtlasFormat::SubpixelMask, 0)
    );
    assert_eq!(
        plan.glyphs[2].page_key,
        GlyphAtlasPageKey::new(GlyphAtlasFormat::Color, 0)
    );
    assert!(plan
        .glyphs
        .iter()
        .all(|glyph| glyph.atlas_rect.x == 0 && glyph.atlas_rect.y == 0));
}

#[test]
fn render_text_atlas_bitmap_run_reserves_new_page_on_shelf_overflow() {
    let plan = glyph_atlas_bitmap_run_plan_with_padding(
        [
            source(GlyphAtlasFormat::AlphaMask, UVec2::new(12, 12), 4.0, 144),
            source(GlyphAtlasFormat::AlphaMask, UVec2::new(12, 12), 20.0, 144),
        ],
        UVec2::new(16, 16),
        11,
        2,
        2,
    );

    assert!(plan.allocation_failures.is_empty());
    assert_eq!(plan.atlas.page_count(), 2);
    assert_eq!(
        plan.glyphs[0].page_key,
        GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0)
    );
    assert_eq!(
        plan.glyphs[1].page_key,
        GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 1)
    );
    assert_eq!(plan.dirty_pages.len(), 2);
    assert_eq!(
        plan.dirty_pages[0].merged_rect(),
        Some(atlas_rect(0, 0, 12, 12))
    );
    assert_eq!(
        plan.dirty_pages[1].merged_rect(),
        Some(atlas_rect(0, 0, 12, 12))
    );
}

#[test]
fn render_text_atlas_bitmap_run_emits_upload_commands_for_dirty_pages() {
    let plan = glyph_atlas_bitmap_run_plan_with_padding(
        [
            source(GlyphAtlasFormat::AlphaMask, UVec2::new(8, 4), 8.0, 32),
            source(GlyphAtlasFormat::SubpixelMask, UVec2::new(6, 4), 22.0, 96),
        ],
        UVec2::new(32, 32),
        19,
        1,
        2,
    );

    assert!(plan.allocation_failures.is_empty());
    assert_eq!(plan.upload_commands.len(), 2);
    assert_eq!(
        plan.upload_commands[0].mode,
        GlyphAtlasUploadMode::PartialRect
    );
    assert_eq!(
        plan.upload_commands[0].page_key,
        GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0)
    );
    assert_eq!(plan.upload_commands[0].rect, atlas_rect(0, 0, 8, 4));
    assert_eq!(plan.upload_commands[0].bytes_per_row, 32);
    assert_eq!(plan.upload_commands[0].upload_byte_len, 32);

    assert_eq!(
        plan.upload_commands[1].mode,
        GlyphAtlasUploadMode::PartialRect
    );
    assert_eq!(
        plan.upload_commands[1].sampling_semantics,
        GlyphAtlasSamplingSemantics::SubpixelCoverage
    );
    assert_eq!(plan.upload_commands[1].bytes_per_row, 32 * 4);
    assert_eq!(plan.upload_commands[1].upload_byte_len, 6 * 4 * 4);
}

#[test]
fn render_text_atlas_bitmap_run_promotes_full_page_dirty_to_full_upload() {
    let plan = glyph_atlas_bitmap_run_plan_with_padding(
        [source(
            GlyphAtlasFormat::AlphaMask,
            UVec2::new(16, 16),
            8.0,
            256,
        )],
        UVec2::new(16, 16),
        23,
        1,
        0,
    );

    assert!(plan.allocation_failures.is_empty());
    assert_eq!(plan.dirty_pages.len(), 1);
    assert_eq!(plan.upload_commands.len(), 1);
    assert_eq!(plan.upload_commands[0].mode, GlyphAtlasUploadMode::FullPage);
    assert_eq!(plan.upload_commands[0].rect, atlas_rect(0, 0, 16, 16));
    assert_eq!(plan.upload_commands[0].bytes_per_row, 16);
    assert_eq!(plan.upload_commands[0].rows_per_image, 16);
    assert_eq!(plan.upload_commands[0].upload_byte_len, 256);
}

#[test]
fn render_text_atlas_bitmap_run_records_failures_for_invalid_sources() {
    let plan = glyph_atlas_bitmap_run_plan_with_padding(
        [
            source(GlyphAtlasFormat::AlphaMask, UVec2::new(4, 4), 4.0, 16),
            source(GlyphAtlasFormat::Sdf, UVec2::new(4, 4), 12.0, 16),
            source(GlyphAtlasFormat::AlphaMask, UVec2::new(0, 4), 20.0, 0),
            source(GlyphAtlasFormat::Color, UVec2::new(2, 2), 28.0, 15),
            source(GlyphAtlasFormat::AlphaMask, UVec2::new(20, 20), 36.0, 400),
        ],
        UVec2::new(16, 16),
        13,
        0,
        2,
    );

    assert!(plan.glyphs.is_empty());
    assert_eq!(plan.atlas.page_count(), 0);
    assert_eq!(plan.blocked_glyphs.len(), 1);
    assert_eq!(plan.blocked_glyphs[0].source_index, 0);
    assert_eq!(
        plan.blocked_glyphs[0].source.format,
        GlyphAtlasFormat::AlphaMask
    );
    assert_eq!(plan.blocked_glyphs[0].retry_frame_index, 14);
    assert_eq!(plan.placeholder_glyphs.len(), 1);
    assert_eq!(plan.placeholder_glyphs[0].source_index, 0);
    assert_eq!(
        plan.placeholder_glyphs[0].mode,
        GlyphAtlasBitmapPlaceholderMode::TransparentQuad
    );
    assert_eq!(plan.placeholder_glyphs[0].retry_frame_index, 14);
    assert_eq!(
        plan.placeholder_glyphs[0].screen_rect,
        GlyphAtlasScreenRect::new(4.0, 6.0, 4.0, 4.0)
    );
    assert_eq!(
        plan.allocation_failures,
        vec![
            failure(
                0,
                GlyphAtlasFormat::AlphaMask,
                GlyphAtlasBitmapAllocationFailureReason::PageReservationBlocked
            ),
            failure(
                1,
                GlyphAtlasFormat::Sdf,
                GlyphAtlasBitmapAllocationFailureReason::UnsupportedFormat
            ),
            failure(
                2,
                GlyphAtlasFormat::AlphaMask,
                GlyphAtlasBitmapAllocationFailureReason::EmptyContent
            ),
            failure(
                3,
                GlyphAtlasFormat::Color,
                GlyphAtlasBitmapAllocationFailureReason::DataLengthMismatch {
                    expected: 16,
                    actual: 15,
                }
            ),
            failure(
                4,
                GlyphAtlasFormat::AlphaMask,
                GlyphAtlasBitmapAllocationFailureReason::OversizedGlyph
            ),
        ]
    );
}

#[test]
fn render_text_atlas_bitmap_run_feeds_draw_batches_without_rgba_semantic_merge() {
    let plan = glyph_atlas_bitmap_run_plan_with_padding(
        [
            source(GlyphAtlasFormat::SubpixelMask, UVec2::new(8, 8), 8.0, 256),
            source(GlyphAtlasFormat::Color, UVec2::new(8, 8), 22.0, 256),
        ],
        UVec2::new(32, 32),
        17,
        1,
        2,
    );
    let draw_plan = glyph_atlas_draw_batch_plan(
        plan.draw_glyphs,
        GlyphAtlasScreenRect::new(0.0, 0.0, 64.0, 32.0),
    );

    assert_eq!(draw_plan.visible_glyph_count, 2);
    assert_eq!(draw_plan.batches.len(), 2);
    assert!(draw_plan.requires_background_composite);
    assert_eq!(
        draw_plan.batches[0].key.render_contract.blend_mode,
        GlyphAtlasBlendMode::SubpixelBackgroundComposite
    );
    assert_eq!(
        draw_plan.batches[1].key.render_contract.blend_mode,
        GlyphAtlasBlendMode::SourceRgba
    );
}

fn source(
    format: GlyphAtlasFormat,
    content_size: UVec2,
    x: f32,
    source_byte_len: usize,
) -> GlyphAtlasBitmapSource {
    GlyphAtlasBitmapSource {
        format,
        content_size,
        screen_rect: GlyphAtlasScreenRect::new(
            x,
            6.0,
            content_size.x as f32,
            content_size.y as f32,
        ),
        foreground_color: [0.86, 0.88, 0.9, 1.0],
        background_color: [0.08, 0.09, 0.1, 1.0],
        source_byte_len,
    }
}

fn failure(
    source_index: usize,
    format: GlyphAtlasFormat,
    reason: GlyphAtlasBitmapAllocationFailureReason,
) -> GlyphAtlasBitmapAllocationFailure {
    GlyphAtlasBitmapAllocationFailure {
        source_index,
        format,
        reason,
    }
}

fn atlas_rect(x: u32, y: u32, width: u32, height: u32) -> GlyphAtlasRect {
    GlyphAtlasRect {
        x,
        y,
        width,
        height,
    }
}
